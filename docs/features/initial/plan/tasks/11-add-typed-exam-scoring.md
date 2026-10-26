# Task 11: Add Typed Exam Scoring for Every Card Type

**Description:** Add the exam command and an in-memory typed exam flow. Reuse deck selection and exact matching, exclude scheduling completely, and show a final score without saving history yet.

**Implementation steps:**

1. Add `exam.rs` and the `exam <root>` CLI command. Reuse the hierarchy selector, then take every card in deterministic order without loading scheduler state.
2. Reject an empty selection before creating exam state. Record `started_at` and the root-relative selected deck path.
3. Build one typed-answer screen for all card types. For multiple choice, show options for reference but require typing the full option text, never an index.
4. On `Enter`, score with the shared matcher, store the response and expected-answer snapshot in memory, and advance. Do not reveal correctness or the expected answer during the session.
5. On completion, calculate correct count and `correct / total * 100.0`, and display the percentage to one decimal place. Do not write review logs or scheduler rows.
6. Let `Esc` confirm abandonment. An abandoned exam returns without being reported as completed. Keep `Ctrl+C` cleanup behavior.
7. Test all card types, empty responses, empty exam rejection, deterministic totals, and percentage calculation. History persistence arrives in Task 12.

**Rust focus:** Reusing a pure function keeps study and exam matching identical. An iterator's `filter` and `count` can compute correctness, but a simple loop is also clear. Floating-point division requires converting integer counts before dividing.

**Acceptance criteria:**

- [ ] Exams include all selected cards, require typed answers for every type, reject zero-card selections, and show a correct final count and percentage.
- [ ] There is no early reveal, correctness feedback, recall grading, or multiple choice numeric selection.
- [ ] Exam execution does not read or modify scheduler state or review logs, and incomplete exams are not reported as complete.

**Verification:**

- [ ] Run `cargo test exam::tests` and `cargo test model::tests` for scoring and shared matching.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Manual check: run `cargo run -- exam decks`, type correct and incorrect answers for all three types, and verify the score. Start a study session afterward and confirm due selections have not changed.

**Dependencies:** Core app complete checkpoint after Task 10.

**Files likely touched:** `src/main.rs`, `src/model.rs`, `src/exam.rs`, `src/ui.rs`.

**Estimated scope:** Medium, 4 files, about 1.5 to 2 hours.

[Previous task](10-harden-core-add-sample-decks.md) | [Task index](../tasks.md) | [Next task](12-persist-completed-exams-atomically.md)
