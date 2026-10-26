# Task 10: Harden the Core and Add Sample Decks

**Description:** Finish core error handling, empty-selection behavior, user documentation, and sample decks. Resolve integration issues before declaring the core complete.

**Implementation steps:**

1. Add a small `decks/` tree containing all three card types in nested `.yaml` and `.yml` files. Use stable, readable ids and unambiguous answers following `AGENTS.md`.
2. Test and fix empty roots, empty directory selections, cards with no due entries, database-open failures, malformed YAML, and unexpected input during each study stage.
3. Confirm every application error passes through terminal cleanup. Remove any feature-specific terminal restoration that duplicates the guard.
4. Add project `README.md` usage for validation, study modes, controls, database location, stable ids, and scheduler limitations. Point deck authors to `AGENTS.md` and keep exam commands marked as upcoming until Task 11.
5. Manually exercise early exit after saved grades, full-deck grading of future cards, and restart behavior. Check that source YAML is never rewritten.

**Rust focus:** Error context should identify the operation and path while preserving the original error chain. Integration testing checks boundaries between modules rather than repeating every pure unit test. A small sample fixture makes manual checks repeatable.

**Acceptance criteria:**

- [ ] Sample decks validate and exercise every core card type and both study modes without runtime panics.
- [ ] Empty selections and filesystem, YAML, and SQLite failures produce useful errors and restore the terminal.
- [ ] Usage documentation matches actual core commands and explains restart persistence and stable id requirements.

**Verification:**

- [ ] Run `cargo run -- validate decks`, `cargo test`, and `cargo build`.
- [ ] Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`.
- [ ] Manual check: complete and exit sessions in both modes, restart with the same `--db`, and confirm due state and review counts persist. Trigger one malformed-deck error from the TUI and inspect the shell.

**Dependencies:** Task 9.

**Files likely touched:** `README.md`, `decks/rust/ownership.yaml`, `decks/rust/types.yml`, `src/app.rs`, `src/ui.rs`.

**Estimated scope:** Medium, 5 files, about 1.5 to 2 hours.

[Previous task](09-add-multiple-choice-study.md) | [Task index](../tasks.md) | [Next task](11-add-typed-exam-scoring.md)
