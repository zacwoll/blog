pub mod threadpool;

use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
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
                    let peer_addr = match stream.peer_addr() {
                        Ok(addr) => addr,
                        Err(err) => {
                            println!("unknown address: {}", err);
                            continue;
                        }
                    };
                    println!("New connection from {peer_addr}");
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

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "output/sample.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "output/sample.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "output/404.html"),
    };

    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(err) => {
            return Err(format!("Could not read file {filename}: {err}").into())
        }
    };

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes())?;
    Ok(())
}