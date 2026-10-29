use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod storage;
mod model;
mod deck;
mod scheduler;

use crate::deck::{validate, collect_decks};
use crate::model::SchedulerState;
use crate::scheduler::schedule;
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
        #[arg(help("The root path to load the decks from"), default_value = "./decks")]
        root: PathBuf,
    }
}

fn start(db: &PathBuf, root: &PathBuf) -> Result<()> {
    let mut storage = Storage::new(db)
        .with_context(|| anyhow!("Failed to create sqlite Storage for '{}'", db.display()))?;
    storage.migrate()
        .with_context(|| anyhow!("Failed to migrate db '{}'", db.display()))?;
    
    let ( decks, _ ) = collect_decks(root)
        .with_context(|| anyhow!("Failed to collect decks in '{}'", root.display()))?;


    let now = SystemTime::now();
    let now_epoch = now
        .duration_since(UNIX_EPOCH)
        .with_context(|| anyhow!("Failed to retrieve time epoch"))?;
    let now_ts = now_epoch.as_secs() as i64;

    for (_, deck) in decks.decks() {
        for card in &deck.cards {
            let deck_path = deck.path().to_str()
                .with_context(|| anyhow!("Failed to get deck path"))?;
            let state = storage
                .load_scheduler_state(deck_path, card.id())?;
            let new_schedule = schedule(
                state.as_ref(), 
                model::Grade::Good, 
                now_ts
            ).with_context(|| anyhow!("Failed to create new schedule"))?;
            let new_state: SchedulerState = if let Some(mut st) = state {
                st.schedule = new_schedule; st
            } else {
                SchedulerState{
                    card_id: card.id().to_owned(),
                    deck_path: deck_path.to_owned(),
                    schedule: new_schedule
                }
            };
            storage.save_scheduler_state(&new_state)
                .with_context(|| anyhow!("Failed to save new scheduler state for deck '{}' card '{}'",
                    deck_path, card.id()))?;

            println!("{:#?}", new_state)
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
