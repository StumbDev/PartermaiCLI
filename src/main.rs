use std::io::{self, Write, Read};
use std::fs::File;
use std::collections::HashMap;
use colored::*;
use syntect::highlighting::{ThemeSet, Style};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

// Enum to represent different functionalities of Partermai
enum ToolMode {
    Shell,
}

// Struct to represent a session
struct Session {
    name: String,
    history: Vec<String>, // To store command history per session
}

// Session manager to handle multiple sessions
struct SessionManager {
    sessions: HashMap<String, Session>,
    active_session: Option<String>, // Name of the current active session
}

impl SessionManager {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session: None,
        }
    }

    fn create_session(&mut self, name: &str) {
        if self.sessions.contains_key(name) {
            println!("{}", "Session with this name already exists!".red());
        } else {
            let session = Session {
                name: name.to_string(),
                history: Vec::new(),
            };
            self.sessions.insert(name.to_string(), session);
            println!("{}: {}", "New session created".green(), name);
        }
    }

    fn list_sessions(&self) {
        println!("Active Sessions:");
        for name in self.sessions.keys() {
            println!("{}", name.yellow());
        }
    }

    fn switch_session(&mut self, name: &str) {
        if self.sessions.contains_key(name) {
            self.active_session = Some(name.to_string());
            println!("{}: {}", "Switched to session".green(), name);
        } else {
            println!("{}", "Session does not exist!".red());
        }
    }

    fn close_session(&mut self, name: &str) {
        if self.sessions.remove(name).is_some() {
            println!("{}: {}", "Closed session".green(), name);
        } else {
            println!("{}", "Session does not exist!".red());
        }
    }

    fn run_active_session(&mut self, ps: &SyntaxSet, ts: &ThemeSet) {
        if let Some(session_name) = &self.active_session {
            let session = self.sessions.get_mut(session_name).unwrap();
            println!("{}", format!("Running session: {}", session_name).yellow());

            let syntax = ps.find_syntax_by_extension("sh").unwrap(); // Shell syntax
            let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

            loop {
                print!("{}", format!("[partermai ~ {}]$ ", session_name).blue().bold());
                io::stdout().flush().unwrap();

                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read line");
                let input = input.trim();

                if input == "exit" {
                    println!("{}", "Exiting Partermai Shell...".red());
                    break;
                }

                if input == "history" {
                    session.history.iter().for_each(|cmd| println!("{}", cmd));
                    continue;
                }

                if input.starts_with("partermcli") {
                    handle_partermcli(input, session);
                    continue;
                }

                // Syntax highlight the input
                let ranges: Vec<(Style, &str)> = h.highlight(input, ps);
                let highlighted = as_24_bit_terminal_escaped(&ranges[..], true);
                println!("{}", highlighted);

                session.history.push(input.to_string());
            }
        } else {
            println!("{}", "No active session. Create one using 'partermcli +session new <name>'".red());
        }
    }
}

// Function to handle the `partermcli` built-in command
fn handle_partermcli(input: &str, session: &mut Session) {
    let args: Vec<&str> = input.split_whitespace().collect();

    if args.len() > 1 {
        match args[1] {
            "+help" => {
                println!("{}", "Partermai CLI Help:".yellow().bold());
                println!("{}", "+help    - Displays help information".green());
                println!("{}", "+edit    - Opens the text editor".green());
                println!("{}", "+version - Shows modules versions".green());
                println!("{}", "+session new <name>   - Create a new session".green());
                println!("{}", "+session switch <name> - Switch to an existing session".green());
                println!("{}", "+session list  - List all sessions".green());
                println!("{}", "+session close <name> - Close a session".green());
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

fn main() {
    println!("{}", "Welcome to Partermai: The Multifunction Tool".green().bold());

    // Load syntax sets and themes
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let mut session_manager = SessionManager::new();

    loop {
        print!("{}", "[partermai ~]$ ".blue().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        let input = input.trim();

        if input == "exit" {
            println!("{}", "Exiting Partermai...".red());
            break;
        }

        if input.starts_with("partermcli") {
            let args: Vec<&str> = input.split_whitespace().collect();
            if args.len() > 2 && args[1] == "+session" {
                match args[2] {
                    "new" => {
                        if let Some(name) = args.get(3) {
                            session_manager.create_session(name);
                        } else {
                            println!("{}", "Missing session name!".red());
                        }
                    },
                    "switch" => {
                        if let Some(name) = args.get(3) {
                            session_manager.switch_session(name);
                        } else {
                            println!("{}", "Missing session name!".red());
                        }
                    },
                    "list" => session_manager.list_sessions(),
                    "close" => {
                        if let Some(name) = args.get(3) {
                            session_manager.close_session(name);
                        } else {
                            println!("{}", "Missing session name!".red());
                        }
                    },
                    _ => println!("{}", "Invalid session command.".red()),
                }
            } else {
                println!("{}", "Unknown partermcli command. Use +help for available options.".red());
            }
        } else {
            println!("{}", "Unknown command. Use 'partermcli +help' for available options.".red());
        }

        // Run the active session (with syntax highlighting)
        session_manager.run_active_session(&ps, &ts);
    }
}
