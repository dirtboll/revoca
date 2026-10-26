# Task 9: Add Multiple Choice Study

**Description:** Enable normal-study multiple choice with keyboard option navigation, one selection, answer reveal, and manual grading.

**Implementation steps:**

1. Add a multiple choice session state containing the selected option index and whether the answer has been revealed.
2. Draw all options in YAML order. Support `Up`/`Down` and `j`/`k`, with selection bounded to valid indices.
3. On `Enter`, record the selected option and reveal the expected answer. Show whether the selected option was correct, but do not derive a scheduler grade from it.
4. Reuse the post-reveal `1` to `4` grade path and existing persistence transaction.
5. Add tests for navigation boundaries, exactly one selected option, correct and incorrect selections, and the requirement to reveal before grading.

**Rust focus:** An index into a `Vec` must remain in bounds. Saturating subtraction is useful for moving up at index zero. An enum for prompt, revealed, and graded stages makes invalid input transitions easier to prevent.

**Acceptance criteria:**

- [ ] Multiple choice displays at least two options, permits one keyboard selection, and reveals the expected answer on submission.
- [ ] Correctness never assigns a grade; all four post-reveal grades persist normally.
- [ ] Navigation cannot move outside the option list and pre-reveal grade keys do not record a review.

**Verification:**

- [ ] Run `cargo test app::tests` for option and session-state tests.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Manual check: select correct and incorrect options on separate cards, grade each, and verify schedules after restart.

**Dependencies:** Task 8.

**Files likely touched:** `src/app.rs`, `src/ui.rs`.

**Estimated scope:** Medium, 2 files, about 1 to 1.5 hours.

[Previous task](08-add-fill-blank-study.md) | [Task index](../tasks.md) | [Next task](10-harden-core-add-sample-decks.md)
