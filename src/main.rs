use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod storage;
mod model;
mod deck;
mod scheduler;
mod app;
mod ui;

use crate::deck::{validate, collect_decks};
use crate::storage::Storage;
use crate::app::{AppContext, study};
use crate::ui::{UI,UiChoice, PrintlnUi};

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
    #[command(about("Start study session"), long_about = None)]
    Study {
        #[arg(help("The root path to load the decks from"), default_value = "./decks")]
        root: PathBuf,

        #[arg(help("The UI to use"), default_value = "println", short, long, default_value = "println")]
        ui: UiChoice,
    }
}


fn main() -> Result<()> {
    let args = Args::parse();
    let db = args.db;

    match &args.command {
        Commands::Validate { root } => {
            validate(root)?;
            println!("Validate success!");
        }
        Commands::Study { root , ui} => {
            let mut storage = Storage::new(&db)
                .with_context(|| anyhow!("Failed to create storage"))?;
            storage.migrate()
                .with_context(|| anyhow!("Failed to migrate storage"))?;
            let mut ctx = AppContext{storage: &mut storage};
            let (deck_manager, _) = collect_decks(&root)?;
            match ui {
                UiChoice::Println => {
                    let mut ui_obj = UI::Println(PrintlnUi{});
                    study(&mut ctx, &mut ui_obj, &deck_manager)?
                }
                UiChoice::Ratatui => {}
            }
        }
    }

    Ok(())
}
