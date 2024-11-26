pub mod page_builder;

use std::{
    fs, io::Write, path::Path,
};
use gray_matter::{engine::YAML, Matter, ParsedEntityStruct};
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

// Create a struct to hold the front matter
#[derive(Deserialize, Debug)]
pub struct PostData {
    author: String,
    title: String,
    tags: Vec<String>,
    date: String,
    description: String,
    // keywords: Vec<string>
}

#[derive(Clone)]
pub struct File {
    file_name: String,
    stem: String,
    ext: String,
    file_data: Vec<u8>,
}

impl File {
    fn new(file_name: String, stem: String, ext: String, file_data: Vec<u8>) -> File {
        File {
            file_name,
            stem,
            ext,
            file_data,
        }
    }
}

// Data structure that unites the file data and the post data (content)
pub struct Post {
    id: u32,
    file_data: File,
    parsed_post_data: ParsedEntityStruct<PostData>
}

impl Post {
    fn new(id: u32, file: File, parsed_post: ParsedEntityStruct<PostData>) -> Post {
        Post {
            id,
            file_data: file,
            parsed_post_data: parsed_post,
        }
    }

    // This constructs a lightweight preview of a post
    fn to_preview(&self) -> PostPreview {
        PostPreview {
            id: self.id,
            resource: self.file_data.file_name.clone(),
            title: self.parsed_post_data.data.title.clone(),
            tags: self.parsed_post_data.data.tags.clone(),
            date: self.parsed_post_data.data.date.clone(),
            description: self.parsed_post_data.data.description.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct PostPreview {
    id: u32,
    resource: String,
    title: String,
    description: String,
    tags: Vec<String>,
    date: String,
}

pub fn generate_site(content_dir: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {

    // Get Current Directory
    // let current_dir = env::current_dir()?;

    // Create Content directory
    if !Path::new(&content_dir).exists() {
        fs::create_dir(&content_dir)?
    }

    // Create Output directory
    if !Path::new(&output_dir).exists() {
        fs::create_dir(&output_dir)?
    }

    // Create container for read-in files
    let mut posts: Vec<Post> = Vec::new();
    let mut previews: Vec<PostPreview> = Vec::new();
    let mut current_id: u32 = 1;
    let mut post_files : Vec<File> = Vec::new();

    // Get files from directory
    let files = fs::read_dir(content_dir)?;
    
    // Filter files to find .md files
    // Perform manipulation on .md files
    for file in files {
        let file = match file {
            Ok(file) => file,
            Err(err) => {
                println!("Error reading file: {:?}", err);
                continue;
            }
        };
        let file_path = file.path();
        let display = file_path.display();

        // Validate filename is utf-8
        let filename = match file.file_name().to_str() {
            Some(name) => name.to_string(),
            None => {
                println!("Failed to get filename");
                continue;
            }
        };

        // Get extension, filter out non-markdown files
        let ext: String = match file_path.extension() {
            Some(ext) => ext.to_string_lossy().to_string(),
            None => {
                println!("Did not recognize extension");
                continue;
            }
        };

        if ext != "md" {
            println!("This is not a markdown file we don't handle those yet ;): {ext}");
            continue;
        }

        // Extract file stem: "sample.md" => "sample"
        let file_stem = match file_path.file_stem() {
            Some(stem) => match stem.to_str() {
                Some(valid_unicode) => valid_unicode,
                None => "Invalid Unicode found in file stem",
            },
            None => {
                println!("Empty file stem found");
                continue;
            }
        };

        // Extract contents of markdown file
        let file_contents = match fs::read_to_string(&file_path) {
            Ok(contents) => contents,
            Err(err) => {
                println!("Error reading file: {}", err);
                continue;
            }
        };

        // If in the future I upload non-markdown types I won't want to turn them
        // into blog posts, so I push all the files into a post_files collection
        let new_file = File::new(
            filename.clone(),
            file_stem.to_string(),
            ext,
            file_contents.clone().into_bytes()
        );
        post_files.push(new_file.clone());

        // Extract Front Matter from contents of file
        let matter = Matter::<YAML>::new();
        let parsed_matter = match matter.parse_with_struct::<PostData>(&file_contents) {
            Some(matter) => matter,
            None => {
                println!("Something went wrong with the PostData in {display}");
                continue;
            }
        };

        // Add to Post Collection
        let post: Post = Post::new(current_id, new_file, parsed_matter);
        let preview = post.to_preview();
        posts.push(post);
        previews.push(preview);
        // increment id
        current_id += 1;
    }

    // Generate tag set from all posts
    let mut tag_set: HashSet<String> = HashSet::new();
    for preview in &previews {
        for tag in &preview.tags {
            tag_set.insert(tag.clone());
        }
    }

    // Turn blog post => web page
    for post in posts {
        // Generate formatted blog post
        let blog_post = page_builder::generate_blog_post(
            &post,
            &previews,
            &tag_set
        );
        
        let file_name = post.file_data.stem.clone() + ".html";
        let file_path = Path::new(&output_dir).join(&file_name);
        
        let mut create_file = match fs::File::create(&file_path) {
            Ok(new_file) => new_file,
            Err(err) => {
                println!("Error creating file: {}: {}", &file_name, err);
                continue;
            }
        };
        
        // Write HTML to file
        // TODO: rename top-level file_data to avoid file_data.file_data 
        match create_file.write(blog_post.0.as_bytes()) {
            Ok(bytes) => println!("{bytes} bytes written to {}", &file_path.display()),
            Err(err) => println!("Failed to write html to file: {}", err),
        }
        
    }

    Ok(())
}