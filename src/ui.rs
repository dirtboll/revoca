use std::{io::{self, Write}};

use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
// use colored::Colorize;
use crossterm::style::Stylize;

use crate::model::{Card, ExamSession, Grade};

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
                println!("  {} {}", "Question".blue().bold(), card.prompt());
                match card {
                    Card::Qa(_) | Card::FillBlank(_) => {
                        print!("    {} ", "Answer".blue().bold());
                        
                        io::stdout().flush()
                            .with_context(|| anyhow!("Failed to flush stdout for card {}", card.id()))?;
                        io::stdin()
                            .read_line(&mut answer)
                            .with_context(|| anyhow!("Failed to read answer for card {}", card.id()))?;
                    }
                    Card::MultipleChoice { options, .. } => {
                        for opt_i in 0..options.len() {
                            println!("       [{}] {}", format!("{}", opt_i + 1).bold(), options[opt_i])
                        }
                        loop {
                            answer = String::new();
                            print!("    {} ", "Answer".blue().bold());

                            io::stdout().flush()
                                .with_context(|| anyhow!("Failed to flush stdout for card {}", card.id()))?;
                            io::stdin()
                                .read_line(&mut answer)
                                .with_context(|| anyhow!("Failed to read answer for card {}", card.id()))?;

                            match answer.trim().parse::<usize>() {
                                Ok(answer_i) => {
                                    if answer_i - 1 < options.len() {
                                        answer = options[answer_i - 1].to_string();
                                        break;
                                    }
                                }
                                Err(e) => {
                                    println!("    {} {}", "Error!".red().bold(), e)       
                                }
                            }
                        }
                    }
                }
            }
            UI::Ratatui(_) => {}
        }
        Ok(answer)
    }

    pub fn show_result(&self, card: &Card, answer: &str) -> Result<()> {
        if card.is_answer_correct(answer) {
            println!("   {}", "Correct".green().bold());
        } else {
            let card_answer = card.answer();
            match self {
                UI::Println(_) => {
                    println!("    {} {}", "Wrong!".red().bold(), card_answer);
                    
                    io::stdout().flush()
                        .with_context(|| anyhow!("Failed to flush stdout for card {}", card.id()))?;
                }
                UI::Ratatui(_) => {}
            }
        }

        Ok(())
    }

    pub fn ask_grade(&self) -> Result<Grade> {
        match self {
            UI::Println(_) => {
                loop {
                    println!();
                    println!("  {}", "Please grade 1 (Again), 2 (Hard), 3 (Good), or 4 (Easy)".grey());
                    print!("     {} ", "Grade".yellow().bold());
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

    pub fn print_exam_result(&self, exam_session: &ExamSession) -> Result<()> {
        match self {
            UI::Println(_) => {
                let score = exam_session.score();
                println!("-----------------");
                println!();
                println!("    {}", "Result".green().bold());
                println!("   {} {}/{}", "Correct".blue().bold(), exam_session.correct_cards, exam_session.total_cards);
                println!("     {} {}", "Score".blue().bold(), if score >= 90 {
                    format!("{}%", score).green().bold()
                } else if score >= 50 {
                    format!("{}%", score).yellow().bold()
                } else {
                    format!("{}%", score).red().bold()
                });
            }
            UI::Ratatui(_) => {}
        }

        Ok(())
    }
}

