# Task 8: Add Fill Blank Study

**Description:** Enable fill blank cards in the existing reveal and typed-attempt flow without changing scheduling or matching rules.

**Implementation steps:**

1. Extend the study card dispatch so `fill_blank` uses the same pre-reveal choices as Q&A: direct reveal or `t` for typed input.
2. Render the prompt as authored, preserving visible blank markers such as `_____`. Do not replace the blank with the answer or infer an answer from underscores.
3. Reuse the Task 7 editor, matcher, expected-answer display, and post-reveal grade handling. Do not create a second matching implementation.
4. Add application tests showing fill blank supports both direct reveal and typed submission and writes the same review types as Q&A.
5. Validate an example with an empty prompt or answer and confirm deck validation still rejects it.

**Rust focus:** Matching on enum variants keeps card-specific behavior explicit. Reusing functions over shared fields avoids duplicating state transitions. Borrowing card content for drawing avoids cloning strings on every frame.

**Acceptance criteria:**

- [ ] Fill blank cards support direct reveal and typed attempts with visible prompt blanks.
- [ ] Typed match status is informational and the user selects Again, Hard, Good, or Easy after reveal.
- [ ] Fill blank reviews persist and participate in due + unseen and full-deck selection like Q&A reviews.

**Verification:**

- [ ] Run `cargo test app::tests` and `cargo test deck::tests`.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Manual check: study a fill blank card using both reveal and typed attempts, restart, and verify its new due state.

**Dependencies:** Task 7.

**Files likely touched:** `src/app.rs`, `src/ui.rs`.

**Estimated scope:** Small, 2 files, about 1 hour.

[Previous task](07-add-typed-qa-exact-matching.md) | [Task index](../tasks.md) | [Next task](09-add-multiple-choice-study.md)
