use std::io::{self, Write};
use std::fs::File;
use std::collections::HashMap;
use colored::*;
use syntect::highlighting::{ThemeSet, Style};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
use chrono::{DateTime, Local};
use rustyline::Editor;
use rustyline::history::FileHistory;
use dirs::home_dir;
use rand::seq::SliceRandom;
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::process::Command;
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use reqwest;

// Enum to represent different functionalities of Partermai
enum ToolMode {
    Shell,
}

#[derive(Default)]
struct Environment {
    current_dir: PathBuf,
    home_dir: PathBuf,
}

impl Environment {
    fn new() -> Self {
        Self {
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            home_dir: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
        }
    }

    fn get_current_dir_display(&self) -> String {
        self.current_dir.display().to_string()
    }

    fn change_directory(&mut self, path: &str) -> io::Result<()> {
        let new_path = if path == "~" {
            self.home_dir.clone()
        } else if path.starts_with("~/") {
            self.home_dir.join(&path[2..])
        } else {
            Path::new(path).to_path_buf()
        };

        if new_path.is_dir() {
            self.current_dir = new_path;
            env::set_current_dir(&self.current_dir)?;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Directory not found"))
        }
    }
}

// Struct to represent a session
struct Session {
    name: String,
    history: Vec<String>, // To store command history per session
    env: Environment,
    voia: Option<Voia>,
}

impl Session {
    fn new(name: String) -> Self {
        Self {
            name,
            history: Vec::new(),
            env: Environment::new(),
            voia: None,
        }
    }

    fn get_random_tip() -> &'static str {
        let tips = [
            "💡 Use 'clear' to clean the terminal",
            "💡 Try 'help' to see all available commands",
            "💡 You can switch sessions with 'switch <name>'",
            "💡 Use 'edit' to open the text editor",
            "💡 Press Ctrl+R to search through command history",
            "💡 Use Tab for command completion",
        ];
        tips.choose(&mut rand::thread_rng()).unwrap()
    }

    fn print_goodbye(&self) {
        println!("{}", "\nThanks for using Partermai! See you next time! 👋\n".bright_cyan());
    }
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
            let session = Session::new(name.to_string());
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
            println!("{}", "Session not found!".red());
        }
    }

    fn close_session(&mut self, name: &str) {
        if let Some(session) = self.sessions.remove(name) {
            println!("{}: {}", "Closed session".green(), name);
            session.print_goodbye();
        }
    }

    async fn run_active_session(&mut self, ps: &SyntaxSet, ts: &ThemeSet) {
        if let Some(session_name) = &self.active_session {
            if let Some(session) = self.sessions.get_mut(session_name) {
                let mut editor = Editor::<(), FileHistory>::new().unwrap();
                if let Some(home) = home_dir() {
                    let history_path = home.join(".partermai_history");
                    editor.load_history(&history_path).unwrap_or_default();
                }
                
                let syntax = ps.find_syntax_by_extension("sh").unwrap();
                let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

                // Show a random tip at start
                println!("{}", Session::get_random_tip().bright_yellow());

                loop {
                    let prompt = format!("{}@{}:{} ❯ ", 
                        "partermai".bright_purple(),
                        session_name.bright_blue(),
                        session.env.get_current_dir_display().bright_green());
                    
                    match editor.readline(&prompt) {
                        Ok(input) => {
                            let input = input.trim();
                            if input.is_empty() { continue; }
                            
                            let _ = editor.add_history_entry(input.to_string());
                            session.history.push(input.to_string());

                            let parts: Vec<&str> = input.split_whitespace().collect();
                            if let Some(cmd) = parts.first() {
                                match *cmd {
                                    "exit" | "quit" => {
                                        session.print_goodbye();
                                        break;
                                    },
                                    "clear" => print!("\x1B[2J\x1B[1;1H"),
                                    "tip" => println!("{}", Session::get_random_tip().bright_yellow()),
                                    "history" => {
                                        for cmd in &session.history {
                                            println!("{}", cmd);
                                        }
                                    },
                                    "ls" => {
                                        let mut show_hidden = false;
                                        let mut path = None;

                                        for arg in &parts[1..] {
                                            match *arg {
                                                "-a" => show_hidden = true,
                                                _ if !arg.starts_with('-') => path = Some(*arg),
                                                _ => {}
                                            }
                                        }

                                        if let Err(e) = execute_ls(path, &session.env, show_hidden) {
                                            println!("{}: {}", "Error".red(), e);
                                        }
                                    },
                                    "cd" => {
                                        let path = parts.get(1).map_or("~", |s| *s);
                                        if let Err(e) = session.env.change_directory(path) {
                                            println!("{}: {}", "Error".red(), e);
                                        }
                                    },
                                    "pwd" => execute_pwd(&session.env),
                                    "cat" => {
                                        if let Some(path) = parts.get(1) {
                                            if let Err(e) = execute_cat(path, &session.env) {
                                                println!("{}: {}", "Error".red(), e);
                                            }
                                        } else {
                                            println!("{}", "Usage: cat <file>".red());
                                        }
                                    },
                                    "voia" => {
                                        if session.voia.is_none() {
                                            match std::env::var("OPENAI_API_KEY") {
                                                Ok(_) => {
                                                    session.voia = Some(Voia::new());
                                                    println!("{}", "Voia AI Assistant is now active! Ask me anything...".bright_green());
                                                },
                                                Err(_) => {
                                                    println!("{}", "Error: OPENAI_API_KEY not found in environment".red());
                                                    println!("Please set your OpenAI API key in the .env file:");
                                                    println!("OPENAI_API_KEY=your_api_key_here");
                                                    continue;
                                                }
                                            }
                                        }
                                        
                                        if parts.len() > 1 {
                                            let question = parts[1..].join(" ");
                                            match session.voia.as_mut().unwrap().ask(&question).await {
                                                Ok(response) => println!("{}: {}", "Voia".bright_cyan(), response),
                                                Err(e) => println!("{}: {}", "Error".red(), e),
                                            }
                                        } else {
                                            println!("Usage: voia <your question>");
                                            println!("Example: voia what is the meaning of life?");
                                        }
                                    },
                                    "voia-clear" => {
                                        if let Some(voia) = &mut session.voia {
                                            voia.clear_history();
                                            println!("{}", "Conversation history cleared.".green());
                                        } else {
                                            println!("{}", "Voia is not active. Use 'voia' command first.".yellow());
                                        }
                                    },
                                    "voia-setkey" => {
                                        if let Some(key) = parts.get(1) {
                                            if let Some(voia) = &mut session.voia {
                                                match voia.set_api_key(key) {
                                                    Ok(_) => {
                                                        println!("{}", "API key updated successfully.".green());
                                                        println!("The key has been saved to your .env file.");
                                                    },
                                                    Err(e) => println!("{}: {}", "Error saving API key".red(), e),
                                                }
                                            } else {
                                                session.voia = Some(Voia::new());
                                                if let Some(voia) = &mut session.voia {
                                                    if let Err(e) = voia.set_api_key(key) {
                                                        println!("{}: {}", "Error saving API key".red(), e);
                                                    } else {
                                                        println!("{}", "API key set successfully.".green());
                                                    }
                                                }
                                            }
                                        } else {
                                            println!("Usage: voia-setkey <your-api-key>");
                                        }
                                    },
                                    "voia-model" => {
                                        if parts.len() > 1 {
                                            let model = parts[1];
                                            if let Some(voia) = &mut session.voia {
                                                voia.set_model(model);
                                                println!("Current model: {}", voia.get_model().bright_cyan());
                                            } else {
                                                println!("{}", "Voia is not active. Use 'voia' command first.".yellow());
                                            }
                                        } else {
                                            println!("Available models:");
                                            println!("  {} - Fast, good for most tasks", "gpt-3.5-turbo".bright_cyan());
                                            println!("  {} - Most capable model, but slower", "gpt-4".bright_cyan());
                                            println!("  {} - Legacy model", "text-davinci-003".bright_cyan());
                                            if let Some(voia) = &session.voia {
                                                println!("\nCurrent model: {}", voia.get_model().bright_cyan());
                                            }
                                        }
                                    },
                                    cmd if cmd.starts_with("partermai") => {
                                        handle_partermcli(input, session);
                                    },
                                    _ => {
                                        // Try to execute as system command
                                        if let Ok(mut child) = Command::new(cmd)
                                            .args(&parts[1..])
                                            .current_dir(&session.env.current_dir)
                                            .spawn()
                                        {
                                            let _ = child.wait();
                                        } else {
                                            println!("{}: command not found", cmd.red());
                                        }
                                    }
                                }
                            }

                            if let Some(home) = home_dir() {
                                let history_path = home.join(".partermai_history");
                                editor.save_history(&history_path).unwrap_or_default();
                            }
                        },
                        Err(_) => {
                            println!("{}", "Error reading input".red());
                            break;
                        }
                    }
                }
            }
        } else {
            println!("{}", "No active session. Create one using 'partermai new <name>'".red());
        }
    }
}

// Function to handle the `partermai` built-in command
fn handle_partermcli(input: &str, _session: &mut Session) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    match parts.get(1).map(|s| *s) {
        Some("help") => {
            println!("\n🌟 Welcome to Partermai CLI Help 🌟\n");
            
            println!("Session Management:");
            println!("  {} - Create a new session", "partermai new <name>".yellow());
            println!("  {} - Switch to a session", "partermai switch <name>".yellow());
            println!("  {} - List all sessions", "partermai list".yellow());
            println!("  {} - Close a session", "partermai close <name>".yellow());
            
            println!("\nFile Operations:");
            println!("  {} - List directory contents", "ls [-a] [path]".yellow());
            println!("    -a: Show hidden files");
            println!("  {} - Change directory", "cd [path]".yellow());
            println!("    ~: Home directory");
            println!("    ..: Parent directory");
            println!("  {} - Print working directory", "pwd".yellow());
            println!("  {} - Display file contents", "cat <file>".yellow());
            
            println!("\nUtilities:");
            println!("  {} - Show command history", "history".yellow());
            println!("  {} - Clear the screen", "clear".yellow());
            println!("  {} - Show a random tip", "tip".yellow());
            println!("  {} - Show this help message", "partermai help".yellow());
            println!("  {} - Exit the current session", "exit/quit".yellow());
            
            println!("\nVoia AI Assistant:");
            println!("  {} - Start Voia and ask a question", "voia <question>".yellow());
            println!("  {} - Set your OpenAI API key", "voia-setkey <key>".yellow());
            println!("  {} - Change AI model", "voia-model [model]".yellow());
            println!("  {} - Clear conversation history", "voia-clear".yellow());
            println!("Available models: gpt-3.5-turbo, gpt-4, text-davinci-003");
            println!("Note: Requires OpenAI API key in .env file");
            
            println!("\nTips:");
            println!("- Use Tab for command completion");
            println!("- Press Ctrl+R to search through command history");
            println!("- Commands are case-sensitive");
            println!("- Use ~ to refer to your home directory");
            println!("- Colors indicate file types in ls output");
        },
        _ => println!("{}", "Unknown command. Try 'partermai help'".red()),
    }
}

#[derive(Serialize)]
struct VoiaRequest {
    prompt: String,
    max_tokens: u32,
    model: String,
}

#[derive(Deserialize)]
struct VoiaResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    text: String,
}

struct Voia {
    client: reqwest::Client,
    api_key: String,
    model: String,
    conversation_history: Vec<String>,
}

impl Voia {
    fn new() -> Self {
        dotenv().ok();
        let api_key = std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY must be set in environment");
        
        Self {
            client: reqwest::Client::new(),
            api_key,
            model: String::from("gpt-3.5-turbo"),
            conversation_history: Vec::new(),
        }
    }

    fn set_api_key(&mut self, key: &str) -> io::Result<()> {
        let env_path = std::env::current_dir()?.join(".env");
        let env_content = format!("OPENAI_API_KEY={}", key);
        fs::write(&env_path, env_content)?;
        self.api_key = key.to_string();
        Ok(())
    }

    fn set_model(&mut self, model: &str) {
        let valid_models = ["gpt-3.5-turbo", "gpt-4", "text-davinci-003"];
        if valid_models.contains(&model) {
            self.model = model.to_string();
        }
    }

    async fn ask(&mut self, question: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.conversation_history.push(format!("User: {}", question));
        
        let context = self.conversation_history.join("\n");
        let request = VoiaRequest {
            prompt: format!("{}\nVoia:", context),
            max_tokens: 150,
            model: self.model.clone(),
        };

        let response = self.client
            .post("https://api.openai.com/v1/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<VoiaResponse>()
            .await?;

        let reply = response.choices.first()
            .map(|c| c.text.trim().to_string())
            .unwrap_or_else(|| "I'm sorry, I couldn't generate a response.".to_string());

        self.conversation_history.push(format!("Voia: {}", reply));
        Ok(reply)
    }

    fn clear_history(&mut self) {
        self.conversation_history.clear();
    }

    fn get_model(&self) -> &str {
        &self.model
    }
}

fn execute_ls(path: Option<&str>, env: &Environment, show_hidden: bool) -> io::Result<()> {
    let target_path = path.map_or_else(|| env.current_dir.clone(), |p| {
        if p == "~" {
            env.home_dir.clone()
        } else if p.starts_with("~/") {
            env.home_dir.join(&p[2..])
        } else {
            Path::new(p).to_path_buf()
        }
    });

    let entries = fs::read_dir(&target_path)?;
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            show_hidden || !name_str.starts_with('.')
        })
        .collect();

    files.sort_by_key(|entry| entry.file_name());

    for entry in files {
        let metadata = entry.metadata()?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if metadata.is_dir() {
            print!("{} ", name_str.blue().bold());
        } else {
            print!("{} ", name_str);
        }
    }
    println!();
    Ok(())
}

fn execute_pwd(env: &Environment) {
    println!("{}", env.get_current_dir_display());
}

fn execute_cat(path: &str, env: &Environment) -> io::Result<()> {
    let target_path = if path == "~" {
        env.home_dir.clone()
    } else if path.starts_with("~/") {
        env.home_dir.join(&path[2..])
    } else {
        env.current_dir.join(path)
    };

    let content = fs::read_to_string(&target_path)?;
    print!("{}", content);
    Ok(())
}

// Simple text editor function
fn start_text_editor() -> io::Result<()> {
    let mut content = String::new();
    println!("Simple text editor (Press Ctrl+D or type 'exit' to save and exit)");
    
    loop {
        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 || line.trim() == "exit" {
            break;
        }
        content.push_str(&line);
    }
    
    println!("Enter filename to save:");
    let mut filename = String::new();
    io::stdin().read_line(&mut filename)?;
    let filename = filename.trim();
    
    let mut file = File::create(filename)?;
    file.write_all(content.as_bytes())?;
    println!("{}: {}", "File saved".green(), filename);
    Ok(())
}

fn print_welcome_banner() {
    let banner = r#"
    ╔═══════════════════════════════════════╗
    ║           Welcome to Partermai         ║
    ║      Your Cozy Command-Line Home      ║
    ╚═══════════════════════════════════════╝
    "#;
    println!("{}", banner.bright_cyan());
    
    let now: DateTime<Local> = Local::now();
    println!("{} Started at: {}\n", "🕒".bright_yellow(), now.format("%Y-%m-%d %H:%M:%S").to_string().bright_green());
}

#[tokio::main]
async fn main() {
    print_welcome_banner();

    // Load syntax sets and themes
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let mut session_manager = SessionManager::new();
    
    // Create a default session
    session_manager.create_session("main");
    session_manager.switch_session("main");
    
    session_manager.run_active_session(&ps, &ts).await;
}
