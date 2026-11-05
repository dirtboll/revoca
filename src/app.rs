use std::hash::{BuildHasher, Hasher, RandomState};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Ok, Result, anyhow};

use crate::model::{Card, DeckFile, DeckManager, ExamAnswer, ExamSession, Grade, SchedulerState};
use crate::{Storage};
use crate::scheduler::schedule;
use crate::ui::UI;


pub struct AppContext {
    pub storage: Storage,
    pub deck_manager: DeckManager,
    pub is_all_card: bool,
}

// TODO: 
// - add ui to filter decks based on hierarchy

pub fn study(ctx: &mut AppContext, ui: &mut UI) -> Result<()> {
    let storage = &mut ctx.storage;
    let deck_manager = &ctx.deck_manager;

    let decks: Vec<&DeckFile> = deck_manager.decks().values().collect();
    let cards = decks.iter().map(|d| &d.cards).flatten().collect();

    let mut filtered_cards = if ctx.is_all_card { cards } 
        else { filter_cards(storage, cards)? };
    shuffle(&mut filtered_cards);

    for card in filtered_cards {
        let card_id = card.id();

        let answer = ui.ask_card(card)
            .with_context(|| anyhow!("Failed to ask card '{}' deck '{}'", card_id, card.deck_path()))?;

        ui.show_result(card, &answer)
            .with_context(|| anyhow!("Failed to show answer for card '{}' deck '{}'", card_id, card.deck_path()))?;

        let grade = ui.ask_grade()
            .with_context(|| anyhow!("Failed to ask grade for card '{}' deck '{}'", card_id, card.deck_path()))?;
        
        update_card_schedule(storage, grade, card)
            .with_context(|| anyhow!("Failed to update card schedule for  card '{}' deck '{}'", card_id, card.deck_path()))?;
        
    }

    Ok(())
}

pub fn exam(ctx: &mut AppContext, ui: &mut UI) -> Result<()> {
    let storage = &mut ctx.storage;
    let deck_manager = &ctx.deck_manager;

    let decks: Vec<&DeckFile> = deck_manager.decks().values().collect();
    let mut cards: Vec<&Card> = decks.iter().map(|d| &d.cards).flatten().collect();
    shuffle(&mut cards);

    let mut now_ts = get_now_ts()?;

    let mut exam_session = ExamSession::new(
        deck_manager.root(), 
        now_ts, 
        -1,
        cards.len() as i64, 
        -1);
    storage.save_exam_session(&exam_session)
        .with_context(|| anyhow!("Failed to save exam session"))?;

    let mut exam_answers: Vec<ExamAnswer> = Vec::new();
    for card_i in 0..cards.len() {
        let card = cards[card_i];
        let card_id = card.id();

        let answer = ui.ask_card(card)
            .with_context(|| anyhow!("Failed to ask card '{}' deck '{}'", card_id, card.deck_path()))?;
        let is_answer_correct = card.is_answer_correct(&answer);
        
        ui.show_result(card, &answer)
            .with_context(|| anyhow!("Failed to show answer for card '{}' deck '{}'", card_id, card.deck_path()))?;

        now_ts = get_now_ts()?;
        let exam_answer = ExamAnswer::new(
            &exam_session, 
            card_i as i64, 
            card, 
            &answer, 
            is_answer_correct, 
            now_ts
        );
        storage.save_exam_answer(&exam_answer)
            .with_context(|| anyhow!("Failed to save exam answer for card '{}' session '{}'", card.id(), exam_session.id()))?;
        exam_answers.push(exam_answer);
    }

    now_ts = get_now_ts()?;
    exam_session.completed_at = now_ts;
    exam_session.correct_cards = exam_answers
        .iter()
        .filter_map(|a| if a.is_correct { Some(1) } else { None })
        .sum();
    storage.save_exam_session(&exam_session)
        .with_context(|| anyhow!("Failed to save exam session '{}'", exam_session.id()))?;
    
    ui.print_exam_result(&exam_session)
        .with_context(|| anyhow!("Failed to print exam result"))?;

    Ok(())
}

fn filter_cards<'a>(storage: &Storage, cards: Vec<&'a Card>) -> Result<Vec<&'a Card>> {
    let now = SystemTime::now();
    let now_epoch = now
        .duration_since(UNIX_EPOCH)
        .with_context(|| anyhow!("Failed to retrieve time epoch"))?;
    let now_ts = now_epoch.as_secs() as i64;

    let mut filtered_cards: Vec<&Card> = Vec::new();
    for c in cards {
        let schedule = storage.load_scheduler_state(c.deck_path(), c.id())
            .with_context(|| anyhow!("Failed to load scheduler state for card '{}' deck '{}'", c.id(), c.deck_path()))?;
        if let Some(s) = schedule {
            if s.schedule.due_at <= now_ts {
                filtered_cards.push(c);
            }
        } else {
            filtered_cards.push(c);
        }
    };

    if filtered_cards.len() == 0 {
        Err(anyhow!("All cards are not yet due, nothing to study!"))
    } else {
        Ok(filtered_cards)
    }
}

fn update_card_schedule(storage: &mut Storage, grade: Grade, card: &Card) -> Result<()> {
    let deck_path = card.deck_path();
    let card_id = card.id();

    let old_scheduler_state = storage.load_scheduler_state(&deck_path, card_id)
        .with_context(|| anyhow!("Failed to load scheduler state for card '{}' deck '{}'", card_id, deck_path))?;

    let now = SystemTime::now();
    let now_epoch = now
        .duration_since(UNIX_EPOCH)
        .with_context(|| anyhow!("Failed to retrieve time epoch"))?;
    let now_ts = now_epoch.as_secs() as i64;

    let new_schedule = schedule(old_scheduler_state.as_ref(), grade.clone(), now_ts)
        .with_context(|| anyhow!("Failed to get new schedule for deck '{}'", deck_path))?;

    let new_scheduler_state = SchedulerState{
        card_id: card_id.to_string(),
        deck_path: deck_path.to_string(),
        schedule: new_schedule,
    };
    storage.save_scheduler_state(&new_scheduler_state)
        .with_context(|| anyhow!("Failed to save new schedule for deck '{}'", deck_path))?;

    storage.insert_review_logs(&new_scheduler_state, grade.clone(), now_ts)
        .with_context(|| anyhow!("Failed to save review logs for deck '{}'", deck_path))?;

    Ok(())
}



fn shuffle<T>(vec: &mut [T]) {
    let n = vec.len();
    for i in 0..n {
        // Generate a pseudo-random index using RandomState
        let random_u64 = RandomState::new().build_hasher().finish();
        let j = i + (random_u64 as usize % (n - i));
        
        vec.swap(i, j);
    }
}

fn get_now_ts() -> Result<i64> {
    let now = SystemTime::now();
    let now_epoch = now
        .duration_since(UNIX_EPOCH)
        .with_context(|| anyhow!("Failed to retrieve time epoch"))?;
    Ok(now_epoch.as_secs() as i64)
}