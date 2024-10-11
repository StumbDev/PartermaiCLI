use std::io::{self, Write, Read};
use std::fs::File;
use colored::*;

// Enum to represent different functionalities of Partermai
enum ToolMode {
    Shell,
}

// Function for starting the Partermai shell
fn main() {
    println!("{}", "Welcome to Partermai: The Multifunction Tool".green().bold());

    // Tool selection - right now, it starts with the shell by default.
    let mode = ToolMode::Shell;

    match mode {
        ToolMode::Shell => start_shell(),
    }
}

// Shell functionality
fn start_shell() {
    println!("{}", "Starting Partermai Shell. Type 'exit' to quit.".yellow());

    loop {
        // Display the prompt in blue
        print!("{}", "[partermai ~]$ ".blue().bold());
        io::stdout().flush().unwrap();

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        let input = input.trim(); // Remove any extra newlines or spaces

        if input == "exit" {
            println!("{}", "Exiting Partermai Shell...".red());
            break;
        }

        if input.is_empty() {
            continue; // Skip empty inputs
        }

        // Check for the built-in command `partermcli`
        if input.starts_with("partermcli") {
            handle_partermcli(input);
            continue;
        }

        // Other command handling (if any)
    }
}

// Function to handle the `partermcli` built-in command
fn handle_partermcli(input: &str) {
    let args: Vec<&str> = input.split_whitespace().collect();

    if args.len() > 1 {
        match args[1] {
            "+help" => {
                println!("{}", "Partermai CLI Help:".yellow().bold());
                println!("{}", "+help    - Displays help information".green());
                println!("{}", "+edit    - Opens the text editor".green());
                println!("{}", "+version - Shows modules versions".green());
            },
            "+edit" => {
                start_text_editor();
            },
            "+version" => {
                println!("{}", "Partermai Alpha version 0.1.0-1".bold());
            },
            _ => {
                println!("{}", "Unknown partermcli command. Use +help for available options.".red());
            }
        }
    } else {
        println!("{}", "partermcli: Missing argument. Use +help for available options.".red());
    }
}

// Simple text editor function
fn start_text_editor() {
    println!("{}", "Entering PartermEdit (CTRL + D to save and quit)...".yellow());

    let mut buffer = String::new();
    let stdin = io::stdin();

    loop {
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break, // Break on CTRL + D
            Ok(_) => buffer.push_str(&input),
            Err(error) => println!("Error reading input: {}", error),
        }
    }

    // Prompt the user to save the file
    println!("{}", "Save as: ".yellow().bold());
    let mut filename = String::new();
    io::stdin().read_line(&mut filename).expect("Failed to read filename");
    let filename = filename.trim();

    // Save the buffer to the specified file
    match File::create(filename) {
        Ok(mut file) => {
            file.write_all(buffer.as_bytes()).expect("Failed to write to file");
            println!("{}", "File saved successfully.".green().bold());
        },
        Err(e) => println!("{}", format!("Failed to create file: {}", e).red().bold()),
    }
}
