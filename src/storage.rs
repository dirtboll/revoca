use anyhow::{Context, Error, Result, anyhow};
use serde_rusqlite::{columns_from_statement, from_row_with_columns, to_params_named};
use std::path::Path;
use std::time::Duration;
use rusqlite::{Connection, params};

use crate::model::{ExamAnswer, ExamSession, Grade, ReviewLog, SchedulerState};

pub struct Storage {
    connection: Connection,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let connection = Connection::open(path_ref)
            .with_context(|| anyhow!("Failed to open sqlite db '{}'", path_ref.display()))?;

        connection.pragma_update(None, "foreign_keys", "ON")
            .with_context(|| anyhow!("Failed to enable foreign keys"))?;

        connection.pragma_update(None, "journal_mode", "WAL")
            .with_context(|| anyhow!("Failed to enable WAL mode"))?;

        connection.busy_timeout(Duration::from_secs(5))
            .with_context(|| anyhow!("Failed to set database timeout"))?;
        Ok(Storage{ connection })
    }

    pub fn migrate(&mut self) -> Result<()> {
        let conn = &mut self.connection;
        let current_version: i32 = conn.query_row(
            "PRAGMA user_version", [], 
            |r| r.get(0)
        ).with_context(|| anyhow!("Failed to get user_version"))?;

        let tx = conn.transaction()
            .with_context(|| anyhow!("Failed to create transaction"))?;

        if current_version < 1 {
            tx.execute_batch(r#"
                CREATE TABLE scheduler_state (
                    deck_path TEXT NOT NULL,
                    card_id TEXT NOT NULL,
                    due_at INTEGER NOT NULL,
                    interval_seconds INTEGER NOT NULL,
                    ease REAL NOT NULL,
                    repetitions INTEGER NOT NULL,
                    PRIMARY KEY (deck_path, card_id)
                );

                CREATE TABLE review_logs (
                    id TEXT PRIMARY KEY,
                    deck_path TEXT NOT NULL,
                    card_id TEXT NOT NULL,
                    grade INTEGER NOT NULL,
                    reviewed_at INTEGER NOT NULL,
                    resulting_due_at INTEGER NOT NULL,
                    resulting_interval_seconds INTEGER NOT NULL,
                    resulting_ease REAL NOT NULL,
                    resulting_repetitions INTEGER NOT NULL
                );

                PRAGMA user_version = 1;
            "#).with_context(|| anyhow!("Failed to execute batch migration 1"))?;
        }

        if current_version < 2 {
            tx.execute_batch(r#"
                CREATE TABLE exam_sessions (
                    id TEXT PRIMARY KEY,
                    selected_deck_path TEXT NOT NULL,
                    started_at INTEGER NOT NULL,
                    completed_at INTEGER NOT NULL,
                    total_cards INTEGER NOT NULL,
                    correct_cards INTEGER NOT NULL
                );

                CREATE TABLE exam_answers (
                    id TEXT PRIMARY KEY,
                    exam_session_id TEXT NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
                    position INTEGER NOT NULL,
                    deck_path TEXT NOT NULL,
                    card_id TEXT NOT NULL,
                    card_type TEXT NOT NULL,
                    prompt TEXT NOT NULL,
                    typed_response TEXT NOT NULL,
                    expected_answer TEXT NOT NULL,
                    is_correct INTEGER NOT NULL,
                    answered_at INTEGER NOT NULL,
                    UNIQUE (exam_session_id, position)
                );

                PRAGMA user_version = 2;
            "#).with_context(|| anyhow!("Failed to execute batch migration 2"))?;
        }

        tx.commit()
            .with_context(|| anyhow!("Failed to commit migration for version {}", current_version))?;

        Ok(())
    }

    pub fn load_scheduler_state(&self, deck_path: &str, card_id: &str) -> Result<Option<SchedulerState>> {
        let conn = &self.connection;

        let mut stmt = conn.prepare(r#"
            SELECT * FROM scheduler_state 
                WHERE deck_path = ? 
                  AND card_id = ?"#
        ).with_context(|| anyhow!("Failed to prepare statement for deck '{}' card '{}'", deck_path, card_id))?;
        let columns = columns_from_statement(&stmt);

        let mut rows = stmt.query_and_then(
            params![deck_path, card_id], 
            |r| from_row_with_columns::<SchedulerState>(r, &columns)
        ).with_context(|| anyhow!("Failed to query scheduler state for deck '{}' card '{}'", deck_path, card_id))?;
        
        rows.next().transpose().map_err(Error::new)
    }

    pub fn save_scheduler_state(&mut self, state: &SchedulerState) -> Result<()> {
        let conn = &mut self.connection;
        let tx = conn.transaction()
            .with_context(|| anyhow!("Failed to create transaction for deck '{}' card '{}'",
                state.deck_path, state.card_id))?;

        {
            let mut stmt = tx.prepare(r#"
                INSERT INTO scheduler_state (deck_path, card_id, due_at, interval_seconds, ease, repetitions)
                VALUES (:deck_path, :card_id, :due_at, :interval_seconds, :ease, :repetitions)
                ON CONFLICT (deck_path, card_id) DO UPDATE SET
                    due_at           = excluded.due_at, 
                    interval_seconds = excluded.interval_seconds, 
                    ease             = excluded.ease, 
                    repetitions      = excluded.repetitions;
            "#).with_context(|| anyhow!("Failed to prepare statement for deck '{}' card '{}')", 
                state.deck_path, state.card_id))?;
                
            let state_params = to_params_named(state)
                .with_context(|| anyhow!("Failed to create named params for for deck '{}' card '{}'",
                    state.deck_path, state.card_id))?;
            
            stmt.execute(state_params.to_slice().as_slice())
                .with_context(|| anyhow!("Failed to execute upsert for deck '{}' card '{}'",
                state.deck_path, state.card_id))?;
        }

        tx.commit().with_context(|| anyhow!("Failed to commit scheduler state for deck '{}' card '{}'",
            state.deck_path, state.card_id))?;

        Ok(())
    }

    pub fn insert_review_logs(&mut self, state: &SchedulerState, grade: Grade, reviewed_at_ts: i64) -> Result<()> {
        let conn = &mut self.connection;
        let tx = conn.transaction()
            .with_context(|| anyhow!("Failed to create transaction for deck '{}' card '{}'",
                state.deck_path, state.card_id))?;
        let review_log = ReviewLog::new(state, grade, reviewed_at_ts);

        {
            let mut stmt = tx.prepare(r#"
                INSERT INTO review_logs (
                    id, deck_path, 
                    card_id, grade, 
                    reviewed_at, 
                    resulting_due_at,
                    resulting_interval_seconds,
                    resulting_ease,
                    resulting_repetitions
                ) VALUES (
                    :id, :deck_path, 
                    :card_id, :grade, 
                    :reviewed_at, 
                    :resulting_due_at, 
                    :resulting_interval_seconds, 
                    :resulting_ease, 
                    :resulting_repetitions
                );
            "#).with_context(|| anyhow!("Failed to prepare statement for deck '{}' card '{}')", 
                state.deck_path, state.card_id))?;

            let state_params = to_params_named(review_log)
                .with_context(|| anyhow!("Failed to create named params for for review log for deck '{}' card '{}'",
                    state.deck_path, state.card_id))?;

            stmt.execute(state_params.to_slice().as_slice())
                .with_context(|| anyhow!("Failed to insert review log for deck '{}' card '{}'",
                state.deck_path, state.card_id))?;
        }

        tx.commit().with_context(|| anyhow!("Failed to commit review logs for deck '{}' card '{}'",
            state.deck_path, state.card_id))?;

        Ok(())
    }

    pub fn save_exam_session(&mut self, exam_session: &ExamSession) -> Result<()> {
        let conn = &mut self.connection;
        let tx = conn.transaction()
            .with_context(|| anyhow!("Failed to create transaction for exam session '{}'",
                exam_session.id()))?;

        {
            let mut stmt = tx.prepare(r#"
                INSERT INTO exam_sessions (
                    id, selected_deck_path,
                    started_at, completed_at,
                    total_cards, correct_cards
                ) VALUES (
                    :id, :selected_deck_path,
                    :started_at, :completed_at,
                    :total_cards, :correct_cards
                ) ON CONFLICT (id) DO UPDATE SET
                    completed_at  = excluded.completed_at, 
                    correct_cards = excluded.correct_cards;
            "#).with_context(|| anyhow!("Failed to prepare statement for exam session '{}')", 
                exam_session.id()))?;

            let state_params = to_params_named(&exam_session)
                .with_context(|| anyhow!("Failed to create named params for exam session '{}'",
                    exam_session.id()))?;

            stmt.execute(state_params.to_slice().as_slice())
                .with_context(|| anyhow!("Failed to insert exam session '{}'",
                exam_session.id()))?;
        }

        tx.commit().with_context(|| anyhow!("Failed to commit exam session '{}'",
                exam_session.id()))?;

        Ok(())
    }

    pub fn save_exam_answer(&mut self, exam_answer: &ExamAnswer) -> Result<()> {
        let conn = &mut self.connection;
        let tx = conn.transaction()
            .with_context(|| anyhow!("Failed to create transaction for exam asnwer '{}'",
                exam_answer.id()))?;

        {
            let mut stmt = tx.prepare(r#"
                INSERT INTO exam_answers (
                    id, exam_session_id,
                    position, deck_path,
                    card_id, card_type,
                    prompt, typed_response,
                    expected_answer,
                    is_correct, answered_at
                ) VALUES (
                    :id, :exam_session_id,
                    :position, :deck_path,
                    :card_id, :card_type,
                    :prompt, :typed_response,
                    :expected_answer,
                    :is_correct, :answered_at
                );
            "#).with_context(|| anyhow!("Failed to prepare statement for exam answer '{}')", 
                exam_answer.id()))?;

            let state_params = to_params_named(&exam_answer)
                .with_context(|| anyhow!("Failed to create named params for exam answer '{}'",
                    exam_answer.id()))?;

            stmt.execute(state_params.to_slice().as_slice())
                .with_context(|| anyhow!("Failed to insert exam answer '{}'",
                exam_answer.id()))?;
        }

        Ok(())
    }

    
}
