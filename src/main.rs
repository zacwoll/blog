use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    if let Err(err) = blog::run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
    Ok(())
}
