use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

fn main() {
    println!("Welcome to PartermEdit!");
    println!("Start typing your code. Press CTRL + D (EOF) when finished.\n");

    let mut code = String::new();

    // Reading input line by line until EOF (CTRL + D)
    let stdin = io::stdin();
    let handle = stdin.lock();

    for line in handle.lines() {
        match line {
            Ok(content) => {
                code.push_str(&content);
                code.push('\n'); // Add newline after each input
            }
            Err(_) => break,
        }
    }

    println!("\nInput finished.");

    // Request filename to save the input
    let mut filename = String::new();
    println!("Enter filename to save: ");
    io::stdin()
        .read_line(&mut filename)
        .expect("Failed to read filename");

    // Trim any trailing newline characters
    let filename = filename.trim();

    if !filename.is_empty() {
        match save_to_file(filename, &code) {
            Ok(_) => println!("Code saved to {}", filename),
            Err(e) => eprintln!("Failed to save the file: {}", e),
        }
    } else {
        println!("No filename provided. Code not saved.");
    }
}

fn save_to_file(filename: &str, content: &str) -> io::Result<()> {
    let path = Path::new(filename);
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
