use colored::*;
use std::io::{self, Write};
use std::process::{Command, Stdio};

// Enum to represent different functionalities of Partermai
enum ToolMode {
    Shell,
    // Add more modes here in the future
}

fn main() {
    println!("{}", "Welcome to Partermai v1 ⚡".green().bold());

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

        // Execute the input as a system command
        match execute_command(input) {
            Ok(output) => println!("{}", output.green()), // Successful output in green
            Err(e) => eprintln!("{}", e.red()), // Errors in red
        }
    }
}

// Helper function to execute shell commands
fn execute_command(command: &str) -> Result<String, String> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
