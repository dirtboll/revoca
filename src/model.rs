use serde::{Deserialize, Serialize};

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
}
