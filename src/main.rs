mod generator;
mod server;

use clap::{Parser, Subcommand};
use notify::{RecursiveMode, Watcher};
use std::{env, error::Error, sync::{mpsc, Arc, Mutex}, thread};
use generator::generate_site;
use server::server_create;

// Define the main CLI structure
#[derive(Parser)]
#[command(name = "blog")]
#[command(about = "A simple static site generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Define the subcommands
#[derive(Subcommand)]
enum Commands {
    /// Run the generator, building the html files
    Build,
    /// Serve the generated content over a local web server
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Watch the content directory for changes and regenerate files
    Watch {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build => {
            println!("Building files");
            if let Err(err) = build() {
                println!("Error Generating files: {}", err);
            }
        }
        Commands::Serve { port } => {
            println!("Starting server on port {port}");
            serve(*port)?;
        }
        Commands::Watch { port} => {
            println!("Watching for changes...");
            watch(*port)?;
        }
    }

    Ok(())
}

fn build() -> Result<(), Box<dyn Error>> {
    let cd = env::current_dir()?;
    // .to_string_lossy converts unknown unicode into �
    let content_dir = cd.join("content").to_string_lossy().to_string();
    let output_dir = cd.join("output").to_string_lossy().to_string();
    let _ = match generate_site(&content_dir, &output_dir) {
        Ok(_) => println!("Generation succeeded! Files built in {}", &output_dir),
        Err(err) => {
            println!("Generation failed: {}", err);
        }
    };
    Ok(())
}

fn serve(port: u16) -> Result<(), Box<dyn Error>> {
    server_create(port);

    println!("Server is running on port {port}");
    Ok(())
}

fn watch(port: u16) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();
    let server_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>> = Arc::new(Mutex::new(None));

    let mut watcher = notify::recommended_watcher(tx)?;
    
    let cd = env::current_dir()?;
    let content_dir = cd.join("content");

    // Begin serving content in new thread, save handle

    watcher.watch(&content_dir, RecursiveMode::Recursive)?;

    {
        // Start a new server
        let handle = {
            let server_handle = Arc::clone(&server_handle);
            thread::spawn(move || {
                if let Err(err) = serve(port) {
                    println!("Error running server: {}", err);
                }
                // Ensure the server handle is cleared when the server thread exits
                *server_handle.lock().unwrap() = None;
            })
        };
        *server_handle.lock().unwrap() = Some(handle);
    }

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                println!("File change detected: {:?}", event);

                match build() {
                    Ok(_) => println!("Built Successfully in {}", &content_dir.display()),
                    Err(err) => println!("Error Generating files: {}", err),
                }

                // Retrieve the server_handle and shut it down
                let mut guard = server_handle.lock().unwrap();
                if let Some(handle) = guard.take() {
                    handle.join().unwrap();
                }

                // Start a new server
                let handle = {
                    let server_handle = Arc::clone(&server_handle);
                    thread::spawn(move || {
                        if let Err(err) = serve(port) {
                            println!("Error running server: {}", err);
                        }
                        // Ensure the server handle is cleared when the server thread exits
                        *server_handle.lock().unwrap() = None;
                    })
                };
                *server_handle.lock().unwrap() = Some(handle);
            }
            Ok(Err(err)) => {
                println!("Watch error: {:?}", err);
            }
            Err(err) => {
                return Err(format!("Channel receive error: {:?}", err).into());
            }
        }
    }
}
