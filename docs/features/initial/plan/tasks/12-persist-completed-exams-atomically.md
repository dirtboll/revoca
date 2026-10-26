# Task 12: Persist Completed Exams Atomically

**Description:** Add exam schema migrations and save a completed session with all answer snapshots in one transaction. Preserve study tables and keep abandoned exams out of completed history.

**Implementation steps:**

1. Add migration 2 for `exam_sessions` and `exam_answers` using the plan's fields, foreign key, checks, and unique `(exam_session_id, position)` key. Keep migration 1 unchanged for existing databases.
2. Add storage models for completed sessions and answer snapshots. Include source deck path and card id as well as prompt, type, typed response, expected answer, correctness, position, and timestamps.
3. Implement one storage function that inserts the session and every answer in a transaction, then returns the new session id. Reject zero answers before insertion.
4. Call this function only after the final exam response and score calculation. Report save failure honestly rather than claiming history was saved.
5. Do not insert an incomplete session at exam start. Abandonment therefore leaves no completed-history row.
6. Test migration from version 1, snapshot round-trip, answer ordering, foreign keys, zero-card rejection, and rollback when an answer insert is forced to fail.

**Rust focus:** Database migrations should be ordered and idempotent through a recorded schema version. A transaction can insert a parent, obtain `last_insert_rowid`, and insert children that reference it. Integer booleans in SQLite need explicit conversion to Rust `bool`.

**Acceptance criteria:**

- [ ] Completed exams persist one session and every answer snapshot in original order, and reopening the database retains them.
- [ ] Failed multi-row inserts roll back completely; empty and abandoned exams create no completed session.
- [ ] Exam persistence leaves `scheduler_state` and `review_logs` unchanged.

**Verification:**

- [ ] Run `cargo test storage::tests` and `cargo test exam::tests`, including rollback and no-schedule-change checks.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Manual check: complete an exam with a temporary `--db`, abandon another, and rerun the app to confirm normal study still opens the same database without migration errors.

**Dependencies:** Task 11.

**Files likely touched:** `src/model.rs`, `src/storage.rs`, `src/exam.rs`.

**Estimated scope:** Medium, 3 files, about 1.5 to 2 hours.

[Previous task](11-add-typed-exam-scoring.md) | [Task index](../tasks.md) | [Next task](13-add-exam-history-views.md)
