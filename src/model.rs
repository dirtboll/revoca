use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf, absolute};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_yaml;
use smart_default::SmartDefault;


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BaseCard {
    pub id: String,
    pub prompt: String,
    pub answer: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Card {
    Qa(BaseCard),
    FillBlank(BaseCard),
    MultipleChoice {
        #[serde(flatten)]
        base: BaseCard,
        options: Vec<String>,
    },
}

impl Card {
    pub fn id(&self) -> &str {
        match self {
            Card::Qa(card) | Card::FillBlank(card) => &card.id,
            Card::MultipleChoice { base, .. } => &base.id,
        }
    }
    pub fn prompt(&self) -> &str {
        match self {
            Card::Qa(card) | Card::FillBlank(card) => &card.prompt,
            Card::MultipleChoice { base, .. } => &base.prompt,
        }
    }
    pub fn answer(&self) -> &str {
        match self {
            Card::Qa(card) | Card::FillBlank(card) => &card.answer,
            Card::MultipleChoice { base, .. } => &base.answer,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DeckFile {
    pub cards: Vec<Card>,
    
    #[serde(skip)]
    path: PathBuf,
}

impl DeckFile {
    #[cfg(test)]
    pub fn new<P: AsRef<Path>>(path: P, cards: Vec<Card>) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        DeckFile { cards, path: path_buf }
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let abs_path = absolute(path_ref)
            .with_context(|| anyhow!("Failed to retrieve absolute path of deck '{}'", path_ref.display()))?;
        let fd = File::open(&abs_path)
            .with_context(|| anyhow!("Failed to open file '{}'", abs_path.display()))?;
        let mut deck: Self = serde_yaml::from_reader(fd)
            .with_context(|| anyhow!("Failed to parse YAML file '{}'", abs_path.display()))?;
        deck.path = abs_path.to_path_buf();
        Ok(deck)
    }
}

#[derive(Debug)]
pub struct DeckManager {
    root: PathBuf,
    decks: HashMap<PathBuf, DeckFile>,
    relationships: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl DeckManager {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root_ref = root.as_ref();
        let root_absolute = absolute(root_ref)
            .with_context(|| anyhow!("Failed to change deck manager root path '{}' to absolute", root_ref.display()))?;
        Ok(DeckManager{ root: root_absolute, decks: HashMap::new(), relationships: HashMap::new() })
    }
    pub fn len(&self) -> usize {
        self.decks.len()
    }
    pub fn decks(&self) -> &HashMap<PathBuf, DeckFile> {
        &self.decks
    }
    pub fn root(&self) -> &PathBuf {
        &self.root
    } 
    pub fn add_deck(&mut self, deck: DeckFile) -> Result<()> {
        let deck_path = deck.path();

        if !deck_path.starts_with(&self.root) {
            return Err(anyhow!("Deck '{}' is not inside manager's root '{}'", deck.path().display(), self.root.display()))
        }

        let rel_path = deck_path
            .strip_prefix(&self.root)
            .map(|p| p.strip_prefix("/").unwrap_or(p))
            .map(|p| Path::new(".").join(p))
            .with_context(|| anyhow!("Failed to transform deck '{}' path to relative", deck.path().display()))?;

        self.decks.insert(rel_path.to_path_buf(), deck);

        let deck_ancestors: Vec<&Path> = rel_path.ancestors().collect();
        for folder_i in (1..deck_ancestors.len()).rev() {
            let parent_folder = deck_ancestors[folder_i];
            if parent_folder.eq(Path::new("")) { continue };
            
            let child_folder = deck_ancestors[folder_i-1];
            let folder_children = self.relationships
                .entry(parent_folder.to_path_buf())
                .or_default();
            folder_children.insert(child_folder.to_path_buf());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, SmartDefault)]
pub struct Schedule {
    #[default = 0]
    pub due_at: i64,
    #[default = 0]
    pub interval_seconds: i64,
    #[default = 2.5]
    pub ease: f64,
    #[default = 0]
    pub repetitions: i32,
}

impl Schedule {
    pub fn from(state: &SchedulerState) -> Self {
        Schedule{
            due_at: state.schedule.due_at,
            interval_seconds: state.schedule.interval_seconds,
            ease: state.schedule.ease,
            repetitions: state.schedule.repetitions
        }
    }
}



#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SchedulerState {
    pub deck_path: String,
    pub card_id: String,
    
    #[serde(flatten)]
    pub schedule: Schedule,
}

#[repr(u8)] 
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Grade {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct ReviewLog {
    pub id: i32,
    pub deck_path: String,
    pub card_id: String,
    pub grade: Grade,
    pub reviewed_at: i64,
    pub resulting_due_at: i64,
    pub resulting_interval_seconds: i64,
    pub resulting_ease: f64,
    pub resulting_repetitions: i32,
}

/**
 * TODO: Add unit tests for
 * - Card implementations
 * - YAML parsing
 * - undefined attributes
 */
#[cfg(test)]
mod tests {

}