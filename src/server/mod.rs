pub mod threadpool;

use std::{
    fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, path::{Path, PathBuf},
};
use threadpool::ThreadPool;

pub fn server_create(port: u16) {
    // Bind to the specified port
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap_or_else(|err| {
        eprintln!("Failed to bind to address: {}", err);
        std::process::exit(1);
    });

    // Create a thread pool with 4 threads
    let pool = ThreadPool::new(4);

    // Handle incoming connections
    loop {
        for stream in listener.incoming() {
            // Check stream validity
            let stream = match stream {
                Ok(stream) => {
                    let _peer_addr = match stream.peer_addr() {
                        Ok(addr) => addr,
                        Err(err) => {
                            println!("unknown address: {}", err);
                            continue;
                        }
                    };
                    // println!("New connection from {peer_addr}");
                    stream
                }
                Err(err) => {
                    eprintln!("Failed to establish a connection: {}", err);
                    continue;
                }
            };

            // Add job to pool
            pool.execute(move || {
                if let Err(err) = handle_connection(stream) {
                    eprintln!("Error handling connection: {}", err);
                }
            });
        }
    }

    // println!("Shutting down.");
}

fn get_content_type(ext: &str) -> &str {
    match ext {
        "" => "text/html",
        "html" => "text/html",
        "css" => "text/css",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "js" => "application/javascript",
        _ => "application/octet-stream", // Default content type for files with no extension or unknown extension
    }
}

// Encodes a response in response to a get request
// Path => "/" "/assets/styles.css" "/blog_posts/sample"
fn handle_get(mut stream: TcpStream, path: PathBuf) {

    // Point / => home blog post
    let path = if path.to_str() == Some("/") {
        PathBuf::from("/home")
    } else {
        path
    };


    // TODO: change hardcoded output to env
    let output_dir = "output";
    let path = Path::new("output").join(path);

    let ext = match path.extension() {
        Some(ext) => ext.to_str().unwrap(),
        None => "html"
    };

    // Determine the type of content to send back
    let content_type = get_content_type(ext);


    // Construct the file path based on the extension
    let full_path = {
        format!("{}{}", output_dir, path.to_string_lossy())
    };
    
    // Debug statement for full path
    println!("Reading {full_path}");

    let contents = match fs::read_to_string(&full_path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Error reading contents of {}: {}", &full_path, err);
            "<h1>404 Not Found</h1>".to_string()
        }
    };

    let status_line = if contents == "<h1>404 Not Found</h1>" {
        "HTTP/1.1 404 NOT FOUND"
    } else {
        "HTTP/1.1 200 OK"
    };

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {length}\r\n\r\n{contents}");

    if let Err(err) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to write response to stream: {}", err);
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let buf_reader = BufReader::new(&mut stream);
    // let request_line = buf_reader.lines().next().unwrap().unwrap();

    // TODO: print out entire request, handle them appropriately
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // when debugging, print each request
    // for line in &http_request {
    //     println!("{line}");
    // }

    // Extract the request line
    let request_line: Vec<&str> = match http_request.get(0) {
        Some(line) => {
            println!("{}: {line}", stream.peer_addr()?);
            line.split_whitespace().collect()
        },
        None => {
            eprintln!("Failed to read request line");
            return Err("Failed to read request line".into());
        }
    };

    if request_line.len() != 3 {
        eprintln!("Invalid request line format");
        return Err("Invalid request line format".into());
    }

    let (status, path, _version) = (request_line[0], request_line[1], request_line[2]);

    let path = Path::new("/output").join(path);

    let res = match status {
        "GET" => handle_get(stream, path),
        _ => {
            println!("Invalid request was received: {:?}", request_line)
        },
    };

    println!("{:?}", res);

    Ok(())
}