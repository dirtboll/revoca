use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod storage;
mod model;
mod deck;

use crate::deck::{validate, collect_decks};
use crate::storage::Storage;

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
    #[command(about("Start session"), long_about = None)]
    Start {
        #[arg(help("The root path to load the decks from"))]
        root: PathBuf,
    }
}

fn start(db: &PathBuf, root: &PathBuf) -> Result<()> {
    let mut storage = Storage::new(db)?;
    storage.migrate()?;
    
    validate(root)?;
    let ( decks, _ ) = collect_decks(root)?;
    for (_, deck) in decks.decks() {
        for card in &deck.cards {
            let deck_path = deck.path().to_str().with_context(|| anyhow!("Failed to get deck path"))?;
            let schedule = storage.load_scheduler_state(deck_path, card.id())?;
            println!("{:#?}", schedule)
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let db = args.db;

    match &args.command {
        Commands::Validate { root } => {
            validate(root)?;
            println!("Validate success!");
        }
        Commands::Start { root } => {
            start(&db, root)?
        }
    }

    Ok(())
}
