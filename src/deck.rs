use anyhow::{Context, Result, Error, anyhow};
use walkdir::WalkDir;
use std::collections::{HashMap, HashSet};
use std::fs::{metadata};
use std::path::{Path, PathBuf};

use crate::model::{Card, DeckFile, DeckManager};

pub fn validate<P: AsRef<Path>>(root: P) -> Result<()> {
    let root_ref = root.as_ref();
    metadata(root_ref)
        .with_context(|| anyhow!("Path '{}' is invalid", root_ref.display()))?;

    let ( deck_manager, invalid_decks ) = collect_decks(root_ref)
        .with_context(|| anyhow!("Failed to collect decks in '{}'", root_ref.display()))?;

    // TODO: Move println! to main.rs
    if !invalid_decks.is_empty() {
        let mut err_message = String::from(format!("Got {} invalid decks:", invalid_decks.len()));
        for (deck_path, deck_error) in &invalid_decks {
            let mut deck_message = format!("
- path: {} 
  errors:", deck_path.display());
            for chain in deck_error.chain() {
                let chain_msg = format!("
  - {}", chain.to_string());
                deck_message.push_str(chain_msg.as_str());
            }
            err_message.push_str(deck_message.as_str());
        }
        return Err(anyhow!(err_message))
    }

    println!("Got {} valid deck(s):", deck_manager.len());
    for (path, deck) in deck_manager.decks() {
        println!("- path: {}
  cards: {}", path.display(), deck.cards.len())
    }
    println!("");

    Ok(())
}

pub fn collect_decks<P: AsRef<Path>>(root: P) -> Result<(DeckManager, HashMap<PathBuf, Error>)> {
    let root_ref = root.as_ref();
    
    let mut deck_manager = DeckManager::new(root_ref)
        .with_context(|| anyhow!("Failed to create deck manager for root '{}'", root_ref.display()))?;
    let mut invalid_decks: HashMap<PathBuf, Error> = HashMap::new();
    for deck_path in find_yaml_files(root_ref) {
        let deck = read_deck(&deck_path);
        match deck {
            Ok(d) => {
                deck_manager.add_deck(d)
                    .with_context(|| anyhow!("Failed to add deck '{}' to manager '{}'", deck_path.display(), deck_manager.root().display()))?;
            }
            Err(e) => {
                invalid_decks.insert(deck_path, e);
            }
        }
    }

    Ok((deck_manager, invalid_decks))
}



fn find_yaml_files<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    let path_ref = path.as_ref();
    WalkDir::new(path_ref)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
            && e.path().extension().map_or(false, |ext| {
                ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml")
            })
        })
        .map(|e| e.into_path())
        .collect()
}
 
fn read_deck<P: AsRef<Path>>(path: P) -> Result<DeckFile> {
    let file = path.as_ref();

    let deck: DeckFile = DeckFile::load_from_file(file)
        .with_context(|| anyhow!("Failed to parse deck file '{}'", file.display()))?;

    validate_deck(&deck)
        .with_context(|| anyhow!("Failed to validate deck file '{}'", file.display()))?;

    Ok(deck)
}

fn validate_deck(deck: &DeckFile) -> Result<()> {
    let mut card_set: HashSet<&str> = HashSet::new();
    let deck_path = deck.path();
    deck_path.to_str()
        .with_context(|| anyhow!("Deck path '{}' contains invalid unicode character", deck_path.display()))?;
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

    fn write_and_load(deck: &DeckFile, mut temp_file: &NamedTempFile) -> Result<DeckFile> {
        let deck_yaml =
            serde_yaml::to_string(&deck).expect("Failed to convert deck object to YAML string");
        temp_file
            .write_all(deck_yaml.as_bytes())
            .expect("Failed to write deck to temp file");
        let deck_read = read_deck(temp_file.path()).expect("Failed to read and validate deck");
        Ok(deck_read)
    }

    #[test]
    fn test_valid_deck() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let deck = DeckFile::new(
            temp_file.path().to_path_buf(),
            vec![
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
            );
        let deck_read = write_and_load(&deck, &temp_file).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    fn test_empty_deck() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let deck = DeckFile::new( 
            temp_file.path().to_path_buf(),
            vec![] 
        );
        let deck_read = write_and_load(&deck, &temp_file).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(expected = "Card has empty id")]
    fn test_empty_id() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let deck = DeckFile::new(
            temp_file.path().to_path_buf(),
            vec![Card::Qa(BaseCard {
                id: String::from("   "),
                prompt: String::from(
                    "What keyword creates a binding that is immutable by default?",
                ),
                answer: String::from("let"),
            })],
        );
        let deck_read = write_and_load(&deck, &temp_file).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(expected = "Card 'immutable-binding-keyword' has empty prompt")]
    fn test_empty_prompt() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let deck = DeckFile::new(
            temp_file.path().to_path_buf(),
            vec![Card::Qa(BaseCard {
                id: String::from("immutable-binding-keyword"),
                prompt: String::from("    "),
                answer: String::from("let"),
            })],
        );
        let deck_read = write_and_load(&deck, &temp_file).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(expected = "Card 'immutable-binding-keyword' has empty answer")]
    fn test_empty_answer() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let deck = DeckFile::new(
            temp_file.path().to_path_buf(),
            vec![Card::Qa(BaseCard {
                id: String::from("immutable-binding-keyword"),
                prompt: String::from(
                    "What keyword creates a binding that is immutable by default?",
                ),
                answer: String::from("    "),
            })],
        );
        let deck_read = write_and_load(&deck, &temp_file).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(expected = "Duplicate card found with id 'immutable-binding-keyword'")]
    fn test_duplicate_card() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let deck = DeckFile::new(
            temp_file.path().to_path_buf(),
            vec![
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
        );
        let deck_read = write_and_load(&deck, &temp_file).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(
        expected = "Card 'valid-owner-count' has less than two exact-unique trimmed-nonempty options"
    )]
    fn test_more_than_two_options() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let deck = DeckFile::new(
            temp_file.path().to_path_buf(),
            vec![Card::MultipleChoice {
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
        );
        let deck_read = write_and_load(&deck, &temp_file).unwrap();
        assert_eq!(deck, deck_read);
    }

    #[test]
    #[should_panic(
        expected = "Card 'valid-owner-count' has answer that does not equal to exactly one option"
    )]
    fn test_answer_not_in_options() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let deck = DeckFile::new(
            temp_file.path().to_path_buf(),
            vec![Card::MultipleChoice {
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
        );
        let deck_read = write_and_load(&deck, &temp_file).unwrap();
        assert_eq!(deck, deck_read);
    }
}
