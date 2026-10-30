use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};

use crate::model::{DeckManager, SchedulerState};
use crate::Storage;
use crate::scheduler::schedule;
use crate::ui::UI;

pub struct AppContext<'a> {
    pub storage: &'a mut Storage,
}

// TODO: 
// - add ui to filter decks based on hierarchy

pub fn study(ctx: &mut AppContext, ui: &mut UI, deck_manager: &DeckManager) -> Result<()> {
    let storage = &mut *ctx.storage;

    for (_, deck) in deck_manager.decks() {
        let deck_path = deck.path().to_str()
            .with_context(|| anyhow!("Failed to retrieve path for deck '{}'", deck.path().display()))?;
            
        for card in &deck.cards {
            let card_id = card.id();


            let answer = ui.ask_card(card)
              .with_context(|| anyhow!("Failed to ask card '{}' deck '{}'", card_id, deck.path().display()))?;

            if !answer.trim().is_empty() && !answer.trim().eq_ignore_ascii_case(card.answer()) {
                ui.show_answer(card)
                    .with_context(|| anyhow!("Failed to show answer for card '{}' deck '{}'", card_id, deck.path().display()))?;
            }

            let grade = ui.ask_grade()
                .with_context(|| anyhow!("Failed to ask grade for card '{}' deck '{}'", card_id, deck.path().display()))?;
            
            
            let old_scheduler_state = storage.load_scheduler_state(&deck_path, card_id)
                .with_context(|| anyhow!("Failed to load scheduler state for card '{}' deck '{}'", card_id, deck.path().display()))?;

            let now = SystemTime::now();
            let now_epoch = now
                .duration_since(UNIX_EPOCH)
                .with_context(|| anyhow!("Failed to retrieve time epoch"))?;
            let now_ts = now_epoch.as_secs() as i64;

            let new_schedule = schedule(old_scheduler_state.as_ref(), grade, now_ts)
                .with_context(|| anyhow!("Failed to get new schedule for deck '{}'", deck.path().display()))?;

            let new_scheduler_state = SchedulerState{
                card_id: card_id.to_string(),
                deck_path: deck_path.to_string(),
                schedule: new_schedule,
            };
            storage.save_scheduler_state(&new_scheduler_state)
                .with_context(|| anyhow!("Failed to save new schedule for deck '{}'", deck.path().display()))?;
                
        }
    }

    Ok(())
}
