use std::{io::{self, Write}};

use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;

use crate::model::{Card, Grade};

pub struct PrintlnUi;
pub struct RatatuiUi;

pub enum UI {
    Println(PrintlnUi),
    #[allow(dead_code)]
    Ratatui(RatatuiUi),
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiChoice {
    Println,
    Ratatui,
}

impl UI {
    pub fn ask_card(&self, card: &Card) -> Result<String> {
        let mut answer = String::new();
        match self {
            UI::Println(_) => {
                println!("-----------------");
                println!("Q: {}", card.prompt());
                print!("A => ");
                
                io::stdout().flush()
                    .with_context(|| anyhow!("Failed to flush stdout for card {}", card.id()))?;
                io::stdin()
                    .read_line(&mut answer)
                    .with_context(|| anyhow!("Failed to read answer for card {}", card.id()))?;
            }
            UI::Ratatui(_) => {}
        }

        Ok(answer)
    }

    pub fn show_answer(&self, card: &Card) -> Result<()> {
        let answer = card.answer();
        match self {
            UI::Println(_) => {
                println!("Correct answer: {}", answer);
                
                io::stdout().flush()
                    .with_context(|| anyhow!("Failed to flush stdout for card {}", card.id()))?;
            }
            UI::Ratatui(_) => {}
        }

        Ok(())
    }

    pub fn ask_grade(&self) -> Result<Grade> {
        match self {
            UI::Println(_) => {
                loop {
                    println!();
                    print!("Grade? [1 (Again) / 2 (Hard) / 3 (Good) / 4 (Easy) => ");
                    io::stdout().flush()
                        .with_context(|| anyhow!("Failed to flush stdout for card"))?;
    
                    let mut answer = String::new();
                    io::stdin()
                        .read_line(&mut answer)
                        .with_context(|| anyhow!("Failed to read answer for grade"))?;

                    if let Ok(grade) = answer.trim().parse::<Grade>() {
                        return Ok(grade)
                    }
                }
            }
            UI::Ratatui(_) => {
                Ok(Grade::Again)
            }
        }
    }
}

