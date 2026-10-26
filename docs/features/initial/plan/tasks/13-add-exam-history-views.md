# Task 13: Add Exam History List and Detail Views

**Description:** Add `exam-history` with an optional session id. List completed exams newest first and render stored answer snapshots without depending on current YAML.

**Implementation steps:**

1. Add a CLI variant for `exam-history [session_id]`. Reuse the global `--db` path and open SQLite without loading the deck root.
2. Add a storage query for session summaries ordered by `completed_at` descending, then id descending for deterministic ties. Calculate percentages from stored counts.
3. With no id, print session id, selected deck path, completion timestamp, correct count, total, and one-decimal percentage. Print a friendly message when there is no history.
4. With an id, load the session and answers ordered by `position`. Print prompt, card type, typed response, expected answer, and correctness for each item.
5. Return a useful error for an unknown id. Use stored snapshots only, so renamed or removed YAML cannot change old history.
6. Test ordering, empty history, unknown ids, percentages, and detail after the original YAML values are absent.

**Rust focus:** Query rows can be collected into `Vec<T>` before printing, which keeps database code separate from presentation. `Option<i64>` in the CLI can represent an optional session id. Deterministic secondary ordering avoids unstable output when timestamps match.

**Acceptance criteria:**

- [ ] `exam-history` lists completed sessions newest first with accurate counts and percentages and handles empty history.
- [ ] `exam-history <id>` shows every stored response and expected answer in original order; an unknown id gives a clear error.
- [ ] History works without deck files and never modifies study or exam records.

**Verification:**

- [ ] Run `cargo test storage::tests` and `cargo test exam::tests` for query and presentation data.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Manual check: complete two exams, restart, run `cargo run -- exam-history`, inspect both detail views, and temporarily rename the deck root before running history again.

**Dependencies:** Task 12.

**Files likely touched:** `src/main.rs`, `src/storage.rs`, `src/exam.rs`.

**Estimated scope:** Medium, 3 files, about 1 to 1.5 hours.

[Previous task](12-persist-completed-exams-atomically.md) | [Task index](../tasks.md) | [Next task](14-final-verification-user-documentation.md)
