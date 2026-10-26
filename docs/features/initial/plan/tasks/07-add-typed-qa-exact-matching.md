# Task 7: Add Typed Q&A Attempts and Shared Exact Matching

**Description:** Implement the shared normalized exact matcher and the Q&A typed-answer editor. Show match status after submission, but keep grading under user control.

**Implementation steps:**

1. Add a pure answer-matching function in `model.rs`. Apply `trim().to_lowercase()` to both expected and typed strings and compare the results exactly.
2. Test surrounding whitespace, ASCII case, Unicode lowercase behavior, internal whitespace, punctuation, accents, and `ß` versus `ss`. Reuse this function later in exams.
3. Add a Q&A input state to `app.rs` and a text editor to `ui.rs`. `t` opens it, printable keys edit, `Backspace` deletes, and `Enter` submits.
4. After submission, reveal the expected answer and show match or no match. Keep the original typed response visible so the user can compare it.
5. Accept only `1` to `4` for the grade after reveal. `Esc` cancels an unsubmitted attempt, and `Ctrl+C` still exits through cleanup.

**Rust focus:** `str::trim` removes surrounding Unicode whitespace and `str::to_lowercase` returns an owned `String`. This is Unicode lowercase conversion, not full case folding. Keeping input as a `String` and moving a cursor by character boundaries avoids slicing UTF-8 at invalid byte offsets.

**Acceptance criteria:**

- [ ] Typed Q&A attempts reveal the expected answer and accurate match status using the exact shared algorithm.
- [ ] Match status never assigns a grade; all four manual grades still write scheduler state and logs.
- [ ] Cancelling an unsubmitted attempt records nothing, and typed Unicode input does not panic.

**Verification:**

- [ ] Run `cargo test model::tests` and `cargo test app::tests` for matcher and input-state behavior.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Manual check: try an answer with surrounding spaces and changed case, then a punctuation mismatch. Confirm both reveal correctly and both still require a manual grade.

**Dependencies:** Task 6.

**Files likely touched:** `src/model.rs`, `src/app.rs`, `src/ui.rs`.

**Estimated scope:** Medium, 3 files, about 1.5 hours.

[Previous task](06-deliver-qa-reveal-study.md) | [Task index](../tasks.md) | [Next task](08-add-fill-blank-study.md)
