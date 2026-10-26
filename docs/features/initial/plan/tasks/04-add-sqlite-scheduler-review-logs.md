# Task 4: Add SQLite Migrations, Scheduler State, and Review Logs

**Description:** Open the configured database, create versioned study tables, and provide concrete functions for loading state and atomically saving a review. Keep these functions independent of terminal input.

**Implementation steps:**

1. Add `storage.rs` with a `Storage` struct that owns a `rusqlite::Connection`. Open the path supplied by `--db` and execute `PRAGMA foreign_keys = ON` for every new connection.
2. Read `PRAGMA user_version`. Write migration 1 for `scheduler_state` and `review_logs` using the fields and constraints in the plan, then update `user_version` in the same transaction.
3. Add model structs for persisted scheduler state and a review record. Use `i64` for timestamps and intervals, `f64` for ease, and an enum or validated integer representation for grades.
4. Implement `load_state(deck_path, card_id)`, where no row means unseen. Implement a review-save function that upserts state and inserts its log in one transaction.
5. Use named or positional parameters for every value. Do not build SQL by interpolating paths, ids, or answers.
6. Test migration on a temporary file, missing state, insert then update, and review-log contents. Reopen the temporary database to prove data was committed.

**Rust focus:** `rusqlite::Connection` owns the database handle. A transaction borrows the connection mutably and commits explicitly. `Option<T>` represents a missing scheduler row without a sentinel value. `ON CONFLICT ... DO UPDATE` performs an upsert while preserving the composite key.

**Acceptance criteria:**

- [ ] A new database migrates to version 1 and reopening it does not duplicate or rerun schema changes.
- [ ] Missing state returns `None`; saved state round-trips all scheduler fields by `(deck_path, card_id)`.
- [ ] Saving a review commits the state update and one log together, with grades constrained to 1 through 4.

**Verification:**

- [ ] Run `cargo test storage::tests` for migration, round-trip, and transaction tests.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Manual check: run the existing `validate` command and confirm YAML validation still works without creating a database.

**Dependencies:** Task 3.

**Files likely touched:** `src/main.rs`, `src/model.rs`, `src/storage.rs`.

**Estimated scope:** Medium, 3 files, about 1.5 to 2 hours.

[Previous task](03-discover-decks-build-hierarchy.md) | [Task index](../tasks.md) | [Next task](05-implement-test-pure-scheduler.md)
