use std::{
    fs, io::Write, path::Path,
};
use comrak::markdown_to_html;
use gray_matter::{Matter, engine::YAML};
use serde::Deserialize;

// Create a struct to hold the front matter
#[derive(Deserialize, Debug)]
pub struct FrontMatter {
    author: String,
    title: String,
    tags: Vec<String>,
    date: String,
    description: String,
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
        let _ext = match file_path.extension() {
            Some(ext) if ext == "md" => ext,
            _ => {
                println!("{} is not a markdown file.", &filename);
                continue;
            }
        };

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

        // Extract Front Matter from contents of file
        let matter = Matter::<YAML>::new();
        let parsed_matter = match matter.parse_with_struct::<FrontMatter>(&file_contents) {
            Some(matter) => matter,
            None => {
                println!("Something went wrong with the frontmatter in {display}");
                continue;
            }
        };
        let (_front_matter, content) = (parsed_matter.data, parsed_matter.content);
        // println!("The author is {}", front_matter.author);

        // parse the content into html
        let html = markdown_to_html(&content, &comrak::Options::default());

        let new_filename = file_stem.to_string() + ".html";
        let new_filepath = Path::new(&output_dir).join(&new_filename);
        let mut new_file = match fs::File::create(&new_filepath) {
            Ok(file) => file,
            Err(err) => {
                println!("Error creating file: {}: {}", &new_filename, err);
                continue;
            }
        };

        // Write HTML to file
        match new_file.write(html.as_bytes()) {
            Ok(bytes) => println!("{bytes} bytes written to {}", &new_filepath.display()),
            Err(err) => println!("Failed to write html to file: {}", err),
        }
    }

    Ok(())
}