use anyhow::{Context, Result, anyhow};
use std::collections::HashSet;
use std::fs::{File, metadata};
use std::path::Path;

use crate::model::{Card, DeckFile};

pub fn validate<P: AsRef<Path>>(root: P) -> Result<()> {
    let path = root.as_ref();
    metadata(path).with_context(|| anyhow!("Path '{}' is invalid", path.display()))?;
    Ok(())
}

pub fn validate_file<P: AsRef<Path>>(file: P) -> Result<()> {
    let _ = read_deck(file)?;
    Ok(())
}

fn read_deck<F: AsRef<Path>>(file: F) -> Result<DeckFile> {
    let path = file.as_ref();

    let file = File::open(path)
        .with_context(|| anyhow!("Failed to open deck file '{}'", path.display()))?;

    let deck: DeckFile = serde_yaml::from_reader(file)
        .with_context(|| anyhow!("Failed to parse deck file '{}'", path.display()))?;

    validate_deck(&deck)
        .with_context(|| anyhow!("Failed to validate deck file '{}'", path.display()))?;

    Ok(deck)
}

fn validate_deck(deck: &DeckFile) -> Result<()> {
    let mut card_set: HashSet<&str> = HashSet::new();
    for card in &deck.cards {
        let id = card.id().trim();
        if id.is_empty() {
            return Err(anyhow!("Card has empty id"));
        }
        if card.prompt().trim().is_empty() {
            return Err(anyhow!("Card '{}' has empty prompt", id));
        }
        let answer = card.answer().trim();
        if answer.is_empty() {
            return Err(anyhow!("Card '{}' has empty answer", id));
        }
        if card_set.contains(id) {
            return Err(anyhow!("Duplicate card found with id '{}'", id));
        }
        card_set.insert(id);
        if let Card::MultipleChoice { options, .. } = card {
            let options_set: HashSet<&str> = options
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if options_set.len() < 2 {
                return Err(anyhow!(
                    "Card '{}' has less than two exact-unique trimmed-nonempty options",
                    id
                ));
            }
            if !options_set.contains(answer) {
                return Err(anyhow!(
                    "Card '{}' has answer that does not equal to exactly one option",
                    id
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BaseCard;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_and_load(deck: &DeckFile) -> Result<DeckFile> {
        let deck_yaml =
            serde_yaml::to_string(&deck).expect("Failed to convert deck object to YAML string");
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(deck_yaml.as_bytes())
            .expect("Failed to write deck to temp file");
        let deck_read = read_deck(temp_file.path()).expect("Failed to read and validate deck");
        Ok(deck_read)
    }

    #[test]
    fn test_valid_deck() {
        let deck = DeckFile {
            cards: vec![
                Card::Qa(BaseCard {
                    id: String::from("immutable-binding-keyword"),
                    prompt: String::from(
                        "What keyword creates a binding that is immutable by default?",
                    ),
                    answer: String::from("let"),
                }),
                Card::FillBlank(BaseCard {
                    id: String::from("owner-blank"),
                    prompt: String::from("Each value in Rust has an _____."),
                    answer: String::from("owner"),
                }),
                Card::MultipleChoice {
                    base: BaseCard {
                        id: String::from("valid-owner-count"),
                        prompt: String::from(
                            "How many owners can a value have at one time under the basic ownership rule?",
                        ),
                        answer: String::from("One"),
                    },
                    options: vec![
                        String::from("Zero"),
                        String::from("One"),
                        String::from("unlimited"),
                    ],
                },
            ],
        };
        let deck_read = write_and_load(&deck).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    fn test_empty_deck() {
        let deck = DeckFile { cards: vec![] };
        let deck_read = write_and_load(&deck).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(expected = "Card has empty id")]
    fn test_empty_id() {
        let deck = DeckFile {
            cards: vec![Card::Qa(BaseCard {
                id: String::from("   "),
                prompt: String::from(
                    "What keyword creates a binding that is immutable by default?",
                ),
                answer: String::from("let"),
            })],
        };
        let deck_read = write_and_load(&deck).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(expected = "Card 'immutable-binding-keyword' has empty prompt")]
    fn test_empty_prompt() {
        let deck = DeckFile {
            cards: vec![Card::Qa(BaseCard {
                id: String::from("immutable-binding-keyword"),
                prompt: String::from("    "),
                answer: String::from("let"),
            })],
        };
        let deck_read = write_and_load(&deck).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(expected = "Card 'immutable-binding-keyword' has empty answer")]
    fn test_empty_answer() {
        let deck = DeckFile {
            cards: vec![Card::Qa(BaseCard {
                id: String::from("immutable-binding-keyword"),
                prompt: String::from(
                    "What keyword creates a binding that is immutable by default?",
                ),
                answer: String::from("    "),
            })],
        };
        let deck_read = write_and_load(&deck).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(expected = "Duplicate card found with id 'immutable-binding-keyword'")]
    fn test_duplicate_card() {
        let deck = DeckFile {
            cards: vec![
                Card::Qa(BaseCard {
                    id: String::from("immutable-binding-keyword"),
                    prompt: String::from(
                        "What keyword creates a binding that is immutable by default?",
                    ),
                    answer: String::from("let"),
                }),
                Card::Qa(BaseCard {
                    id: String::from("immutable-binding-keyword"),
                    prompt: String::from(
                        "What keyword creates a binding that is immutable by default?",
                    ),
                    answer: String::from("let"),
                }),
            ],
        };
        let deck_read = write_and_load(&deck).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(
        expected = "Card 'valid-owner-count' has less than two exact-unique trimmed-nonempty options"
    )]
    fn test_more_than_two_options() {
        let deck = DeckFile {
            cards: vec![Card::MultipleChoice {
                base: BaseCard {
                    id: String::from("valid-owner-count"),
                    prompt: String::from(
                        "How many owners can a value have at one time under the basic ownership rule?",
                    ),
                    answer: String::from("One"),
                },
                options: vec![
                    String::from("Zero"),
                    String::from("   "),
                    String::from("Zero"),
                ],
            }],
        };
        let deck_read = write_and_load(&deck).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(
        expected = "Card 'valid-owner-count' has answer that does not equal to exactly one option"
    )]
    fn test_answer_not_in_options() {
        let deck = DeckFile {
            cards: vec![Card::MultipleChoice {
                base: BaseCard {
                    id: String::from("valid-owner-count"),
                    prompt: String::from(
                        "How many owners can a value have at one time under the basic ownership rule?",
                    ),
                    answer: String::from("Two"),
                },
                options: vec![
                    String::from("Zero"),
                    String::from("One"),
                    String::from("unlimited"),
                ],
            }],
        };
        let deck_read = write_and_load(&deck).unwrap();
        assert_eq!(deck, deck_read);
    }
}
