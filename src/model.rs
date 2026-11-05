use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, absolute};
use std::str::FromStr;

use anyhow::{Context, Result, Error, anyhow};
use serde::{Deserialize, Serialize, Serializer};
use serde_yaml;
use smart_default::SmartDefault;
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BaseCard {
    pub id: String,
    pub prompt: String,
    pub answer: String,

    #[serde(skip)]
    pub deck_file: String,
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
    pub fn deck_path(&self) -> &str {
        match self {
            Card::Qa(card) | Card::FillBlank(card) => &card.deck_file,
            Card::MultipleChoice { base, .. } => &base.deck_file,
        }
    }
    pub fn is_answer_correct(&self, answer: &str) -> bool {
        answer.trim().eq_ignore_ascii_case(self.answer())
    }
    pub fn type_string(&self) -> String {
        let type_str = match self {
            Card::Qa(..) => { "Qa" },
            Card::FillBlank(..) => { "FillBlank" },
            Card::MultipleChoice {..} => { "MultipleChoice" }
        };
        type_str.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DeckFile {
    pub cards: Vec<Card>,
    
    #[serde(skip)]
    path: String,
}

impl DeckFile {
    #[cfg(test)]
    pub fn new<P: AsRef<Path>>(path: P, cards: Vec<Card>) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        let path_string = path_buf.into_string().unwrap();
        DeckFile { cards, path: path_string }
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let abs_path = absolute(path_ref)
            .with_context(|| anyhow!("Failed to retrieve absolute path of deck '{}'", path_ref.display()))?;
        let abs_path_str = abs_path.clone().into_string()
            .map_err(|_| anyhow!("Failed to convert deck '{}' path to string", abs_path.display()))?;
        let fd = File::open(&abs_path)
            .with_context(|| anyhow!("Failed to open file '{}'", abs_path.display()))?;
        let mut deck: Self = serde_yaml::from_reader(fd)
            .with_context(|| anyhow!("Failed to parse YAML file '{}'", abs_path.display()))?;
        for card in &mut deck.cards {
            match card {
                Card::Qa(base) | Card::FillBlank(base) => { base.deck_file = abs_path_str.clone()}
                Card::MultipleChoice { base, .. } => { base.deck_file = abs_path_str.clone()}
            }
        }
        deck.path = abs_path_str;
        Ok(deck)
    }
}

#[derive(Debug)]
pub struct DeckManager {
    root: String,
    decks: HashMap<String, DeckFile>,
    relationships: HashMap<String, HashSet<String>>,
}

impl DeckManager {
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root_ref = root.as_ref();
        let root_absolute = absolute(root_ref)
            .with_context(|| anyhow!("Failed to change deck manager root path '{}' to absolute", root_ref.display()))?;
        let root_absolute_string = root_absolute.into_string()
            .map_err(|_| anyhow!("Failed to parse root path '{}' to string", root_ref.display()))?;
        Ok(DeckManager{ root: root_absolute_string.clone(), decks: HashMap::new(), relationships: HashMap::new() })
    }
    pub fn len(&self) -> usize {
        self.decks.len()
    }
    pub fn decks(&self) -> &HashMap<String, DeckFile> {
        &self.decks
    }
    pub fn root(&self) -> &String {
        &self.root
    } 
    pub fn add_deck(&mut self, deck: DeckFile) -> Result<()> {
        let deck_path = deck.path();

        if !deck_path.starts_with(&self.root) {
            return Err(anyhow!("Deck '{}' is not inside manager's root '{}'", deck.path(), self.root))
        }

        let rel_path = deck_path
            .strip_prefix(&self.root)
            .map(|p| p.strip_prefix("/").unwrap_or(p))
            .map(|p| Path::new(".").join(p))
            .with_context(|| anyhow!("Failed to transform deck '{}' path to relative", deck.path()))?;
        let rel_path_string = rel_path.to_str()
            .with_context(|| anyhow!("Failed to get string representation of root path '{}'", rel_path.display()))?
            .to_string();

        self.decks.insert(rel_path_string, deck);

        let deck_ancestors: Vec<&str> = rel_path.ancestors().filter_map(|a| a.to_str()).collect();
        for folder_i in (1..deck_ancestors.len()).rev() {
            let parent_folder = deck_ancestors[folder_i];
            if Path::new(parent_folder).eq(Path::new("")) { continue };
            
            let child_folder = deck_ancestors[folder_i-1];
            let folder_children = self.relationships
                .entry(parent_folder.to_string())
                .or_default();
            folder_children.insert(child_folder.to_string());
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Grade {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4
}

fn serialize_grade<S>(grade: &Grade, serializer: S) -> Result<S::Ok, S::Error>
where 
    S: Serializer 
{
    let grade_int = grade.clone() as u8;
    serializer.serialize_u8(grade_int)
}

impl TryFrom<u8> for Grade {
    type Error = anyhow::Error;

    fn try_from(v: u8) -> std::prelude::v1::Result<Self, Self::Error> {
        match v {
            v if v == Grade::Again as u8 => Ok(Grade::Again),
            v if v == Grade::Hard as u8 => Ok(Grade::Hard),
            v if v == Grade::Good as u8 => Ok(Grade::Good),
            v if v == Grade::Easy as u8 => Ok(Grade::Easy),
            _ => Err(anyhow!("Int {} can't be parsed to grade", v)),
        }
    }
}

impl FromStr for Grade {
    type Err = Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        let parse_int = s.parse::<u8>()
            .map_err(|_| anyhow!("String '{}' can't be parsed to u8", s))?;

        Grade::try_from(parse_int)
            .map_err(|_| anyhow!("String '{}' can't be parsed to grade", s))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ReviewLog {
    pub id: String,
    pub deck_path: String,
    pub card_id: String,
    #[serde(serialize_with = "serialize_grade")]
    pub grade: Grade,
    pub reviewed_at: i64,
    pub resulting_due_at: i64,
    pub resulting_interval_seconds: i64,
    pub resulting_ease: f64,
    pub resulting_repetitions: i32,
}

impl ReviewLog {
    pub fn new(state: &SchedulerState, grade: Grade, reviewed_at_ts: i64) -> ReviewLog {
        ReviewLog { 
            id: Uuid::now_v7().to_string(), 
            deck_path: state.deck_path.to_string(), 
            card_id: state.card_id.to_string(), 
            grade: grade, 
            reviewed_at: reviewed_at_ts, 
            resulting_due_at: state.schedule.due_at, 
            resulting_interval_seconds: state.schedule.interval_seconds, 
            resulting_ease: state.schedule.ease, 
            resulting_repetitions: state.schedule.repetitions
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ExamSession {
    id: String,
    pub selected_deck_path: String,
    pub started_at: i64,
    pub completed_at: i64,
    pub total_cards: i64,
    pub correct_cards: i64
}

impl ExamSession {
    pub fn new(selected_deck_path: &str, started_at: i64, completed_at: i64, total_cards: i64, correct_cards: i64) -> ExamSession {
        ExamSession {
            id: Uuid::now_v7().to_string(),
            selected_deck_path: selected_deck_path.to_string(),
            started_at: started_at,
            completed_at: completed_at,
            total_cards: total_cards,
            correct_cards: correct_cards,
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn score(&self) -> i64 {
        ((self.correct_cards as f64 / self.total_cards as f64) * 100_f64) as i64
    }
}

fn serialize_bool<S>(boolean: &bool, serializer: S) -> Result<S::Ok, S::Error>
where 
    S: Serializer 
{
    let boolean_int: u8 = if *boolean { 1 } else { 0 };
    serializer.serialize_u8(boolean_int)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ExamAnswer {
    id: String,
    pub exam_session_id: String,
    pub position: i64, // Numerical order of the cards when shown in exam mode
    pub deck_path: String,
    pub card_id: String,
    pub card_type: String,
    pub prompt: String,
    pub typed_response: String,
    pub expected_answer: String,
    #[serde(serialize_with = "serialize_bool")]
    pub is_correct: bool,
    pub answered_at: i64,
}

impl ExamAnswer {
    pub fn new(exam_session: &ExamSession, position: i64, card: &Card, answer: &str, is_correct: bool, answered_at: i64) -> ExamAnswer {
        ExamAnswer { 
            id: Uuid::now_v7().to_string(), 
            exam_session_id: exam_session.id.to_string(), 
            position,
            deck_path: card.deck_path().to_string(), 
            card_id: card.id().to_string(),
            card_type: card.type_string(), 
            prompt: card.prompt().to_string(), 
            typed_response: answer.to_string(), 
            expected_answer: card.answer().to_string(), 
            is_correct, answered_at
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
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