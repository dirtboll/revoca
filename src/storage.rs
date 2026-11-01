use anyhow::{Context, Error, Result, anyhow};
use serde_rusqlite::{columns_from_statement, from_row_with_columns, to_params_named};
use std::path::Path;
use std::time::Duration;
use rusqlite::{Connection, params};
use uuid::Uuid;

use crate::model::{Grade, SchedulerState};

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

        {
            let mut stmt = tx.prepare(r#"
                INSERT INTO review_logs (
                    id, 
                    deck_path, 
                    card_id, 
                    grade, 
                    reviewed_at, 
                    resulting_due_at,
                    resulting_interval_seconds,
                    resulting_ease,
                    resulting_repetitions
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#).with_context(|| anyhow!("Failed to prepare statement for deck '{}' card '{}')", 
                state.deck_path, state.card_id))?;

            stmt.execute(params![
                Uuid::now_v7().to_string(),
                state.deck_path,
                state.card_id,
                grade as u8,
                reviewed_at_ts,
                state.schedule.due_at,
                state.schedule.interval_seconds,
                state.schedule.ease,
                state.schedule.repetitions
            ])
                .with_context(|| anyhow!("Failed to execute upsert for deck '{}' card '{}'",
                state.deck_path, state.card_id))?;
        }

        tx.commit().with_context(|| anyhow!("Failed to commit review logs for deck '{}' card '{}'",
            state.deck_path, state.card_id))?;

        Ok(())
    }
}
