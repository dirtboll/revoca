# Task 14: Final Verification and User Documentation

**Description:** Audit the implementation against the full plan, finish command and authoring documentation, and perform clean automated and manual checks. This task fixes integration defects rather than adding features.

**Implementation steps:**

1. Compare every plan section and acceptance criterion with the implementation. Check scheduler formulas against test fixtures and confirm no automatic study grade exists.
2. Update project `README.md` with final validation, study, exam, and history commands, controls, database behavior, scheduler limitations, exact matching, and exam separation.
3. Review `AGENTS.md` against the parser and sample decks. Correct any schema, validation-command, or matching guidance that drifted during implementation.
4. Run the full quality gate from a clean build state if practical. Investigate every warning or failure instead of suppressing it.
5. Perform final manual restart checks for scheduler state, review logs, exam sessions, and answer history. Verify an exam before and after a study review leaves schedule rows untouched except for the explicit study review.
6. Confirm only YAML owns live card content and SQLite contains scheduler data, review logs, and attempted exam snapshots.

**Rust focus:** A final audit checks behavior at module boundaries and user-visible outputs. Formatting, Clippy, tests, and a normal build answer different questions, so all are required. Manual persistence tests catch wiring mistakes that pure unit tests cannot see.

**Acceptance criteria:**

- [ ] All core and exam acceptance criteria pass, including validation, all study interactions, scheduling, persistence, scoring, and history.
- [ ] Formatting, Clippy with warnings denied, all tests, and the application build are clean.
- [ ] Documentation matches the final app, and manual restart checks prove study and exam persistence with no exam-driven schedule changes.

**Verification:**

- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo build` without failures.
- [ ] Run `cargo run -- validate decks`, then manually exercise both study modes, every card type, typed matching, every grade, and all exit paths.
- [ ] Complete an exam, browse list and detail history after restart, confirm schedules are unchanged by the exam, and verify terminal restoration after an induced error.

**Dependencies:** Task 13.

**Files likely touched:** `README.md`, `AGENTS.md`, and up to three focused source or test files needed for audit fixes.

**Estimated scope:** Medium, at most 5 files, about 1.5 to 2 hours.

[Previous task](13-add-exam-history-views.md) | [Task index](../tasks.md)
