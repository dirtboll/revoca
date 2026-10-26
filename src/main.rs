use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::deck::validate;

mod deck;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(global(true), short, long, default_value = ".flashcards.db")]
    db: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about("Validate deck(s) under specified root path"), long_about = None)]
    Validate {
        #[arg(help("The root path to validate"))]
        root: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        Commands::Validate { root } => {
            let path = root
                .to_str()
                .ok_or_else(|| anyhow!("Path '{}' contains invalid UTF-8", root.display()))?;
            validate(path)?;
            println!("Validate success!");
        }
    }

    Ok(())
}
