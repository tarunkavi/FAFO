use clap::{Parser, Subcommand};
use std::process::exit;

#[derive(Parser)]
#[command(name = "kvs", version, about = "A key-value store")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Get the value of a key
    Get { key: String },
    /// Set the value of a key
    Set { key: String, value: String },
    /// Remove a key
    Rm { key: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Get { .. } => {
            eprintln!("unimplemented");
            exit(1);
        }
        Command::Set { .. } => {
            print!("this is set");
            eprintln!("unimplemented");
            exit(1);
        }
        Command::Rm { .. } => {
            eprintln!("unimplemented");
            exit(1);
        }
    }
}