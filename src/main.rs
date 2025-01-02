use clap::{ArgAction, Parser, Subcommand};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(version = "0.0.1", about = "A Rust version of tips CLI")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    List {
        name: Option<String>,
        #[arg(short = 'q', long = "query")]
        query: Option<String>,
    },
    Edit {
        name: String,
        #[arg(long = "init", action = ArgAction::SetTrue)]
        init: bool,
    },
}

fn tips_home() -> PathBuf {
    if let Ok(path) = env::var("TIPS_HOME") {
        return PathBuf::from(path);
    }

    let mut home_path = PathBuf::from(env::var("HOME").unwrap());
    home_path.push(".config/himkt/dotfiles/tips");
    home_path
}

fn list_tips_names(home: PathBuf, query: Option<String>) {
    let entries = match fs::read_dir(&home) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("No tips.d found on {:?}", home);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext != "tips" {
                continue;
            }
            if let Some(file_stem) = path.file_stem() {
                let file_stem_str = file_stem.to_string_lossy();
                if let Some(ref q) = query {
                    if file_stem_str.contains(q) {
                        println!("{}", file_stem_str);
                    }
                } else {
                    println!("{}", file_stem_str);
                }
            }
        }
    }
}

fn list_tips_for(home: PathBuf, name: String, query: Option<String>) {
    let tips_file = home.join(format!("{}.tips", name));

    if !tips_file.exists() {
        println!("No tips available for {}", name);
        return;
    }

    let file = match File::open(&tips_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Cannot open file: {}", e);
            return;
        }
    };

    let reader = BufReader::new(file);
    for line_res in reader.lines() {
        if let Ok(mut line) = line_res {
            line = line.trim_end().to_string();
            if let Some(ref q) = query {
                if line.contains(q) {
                    println!("{}", line);
                }
            } else {
                println!("{}", line);
            }
        }
    }
}

fn list_tips(name: Option<String>, query: Option<String>) {
    let home = tips_home();
    match name {
        Some(name) => list_tips_for(home, name, query),
        _ => list_tips_names(home, query),
    }
}

fn edit_tips(name: String, init: bool) {
    let home = tips_home();
    let tips_file = home.join(format!("{}.tips", name));

    if init {
        if let Err(e) = fs::create_dir_all(&home) {
            eprintln!("Cannot create directory {:?}: {}", home, e);
            return;
        }
        if !tips_file.exists() {
            if let Err(e) = File::create(&tips_file) {
                eprintln!("Cannot create file {:?}: {}", tips_file, e);
                return;
            }
        }
    }

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let status = Command::new(editor)
        .arg(&tips_file)
        .status()
        .expect("Failed to start editor process");

    if !status.success() {
        println!("Cancelled.");
        return;
    }

    println!("Tips for {} updated.", name);
}

fn main() {
    match Args::parse().command {
        Some(Commands::List { name, query }) => list_tips(name, query),
        Some(Commands::Edit { name, init }) => edit_tips(name, init),
        None => println!("Available commands: list, edit"),
    };
}
