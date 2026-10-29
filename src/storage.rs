use anyhow::{Context, Error, Result, anyhow};
use serde_rusqlite::{columns_from_statement, from_row_with_columns};
use std::path::Path;
use std::time::Duration;
use rusqlite::{Connection, params};

use crate::model::SchedulerState;

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
                    id INTEGER PRIMARY KEY,
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

    pub fn load_scheduler_state(&mut self, deck_path: &str, card_id: &str) -> Result<SchedulerState> {
        let conn = &mut self.connection;

        let mut stmt = conn.prepare("SELECT * FROM scheduler_state WHERE deck_path = ? AND deck_path = ?")?;
        let columns = columns_from_statement(&stmt);
        let mut rows = stmt.query_and_then(params![deck_path, card_id], |r| from_row_with_columns::<SchedulerState>(r, &columns))?;
        if let Some(state) = rows.next() {
            return state.map_err(|err| Error::new(err));
        } else {
            let new_state = SchedulerState{
                deck_path: deck_path.to_string(),
                card_id: card_id.to_string(),
                ease: 2.5,
                interval_seconds: 0,
                repetitions: 0,
                due_at: 0,
            };

            Ok(new_state)
        }
    }
}