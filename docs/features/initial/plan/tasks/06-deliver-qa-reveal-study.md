# Task 6: Deliver End-to-End Q&A Reveal Study

**Description:** Add the study command, terminal UI, deck and mode selection, due plus unseen filtering, and full-deck filtering. A user can now reveal a Q&A card, self-grade it, and persist the result. This is the first usable vertical slice.

**Implementation steps:**

1. Add the `study <root>` CLI variant. Build the card set after the user selects a directory group or file deck and chooses `d` for due + unseen or `f` for full deck.
2. Pass one session-start Unix timestamp into selection. Include cards with no state or `due_at <= now` in due + unseen. Include every card in full mode. Keep each selected card at most once.
3. Add `app.rs` for session state and `ui.rs` for Ratatui drawing and Crossterm input. For unsupported card variants in this slice, show a clear message rather than grading them as Q&A.
4. Implement Q&A reveal with `Space` or `Enter`. After reveal, map `1`, `2`, `3`, and `4` to Again, Hard, Good, and Easy. Do not accept a grade before reveal.
5. For each grade, call the pure scheduler with `SystemTime::now()` converted to Unix seconds, then save the state and log through the Task 4 transaction before advancing.
6. Add one terminal guard that enables raw mode and enters the alternate screen. Restore raw mode, screen, and cursor on success, `?` error returns, and panic. Route every TUI exit through this guard.
7. Test selection with unseen, due, and future-due cards. Manually test restart persistence in both modes.

**Rust focus:** `ratatui` draws frames while `crossterm` supplies terminal events. An event loop repeatedly draws, waits for input, and updates application state. A guard with `Drop` can restore the terminal during ordinary returns and unwinding. `SystemTime::now().duration_since(UNIX_EPOCH)` produces a duration to convert to seconds.

**Acceptance criteria:**

- [ ] A user can select a deck and either mode, reveal Q&A cards, apply all four grades, and reach a completion summary.
- [ ] Due + unseen excludes future-due cards, full deck includes them, and both modes persist identical scheduling and review-log behavior.
- [ ] Normal exit, input errors, and a forced test panic restore the terminal; completed grades survive restarting the app.

**Verification:**

- [ ] Run `cargo test app::tests`, `cargo test scheduler::tests`, and `cargo test storage::tests`.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Manual check: run `cargo run -- study decks`, grade a new Q&A card Again, restart, and verify it is absent from immediate due + unseen but present in full mode. Exit with `Esc` and `Ctrl+C` and check the shell.

**Dependencies:** Tasks 4 and 5.

**Files likely touched:** `src/main.rs`, `src/model.rs`, `src/app.rs`, `src/ui.rs`, `src/storage.rs`.

**Estimated scope:** Medium, 5 files, about 2 hours.

[Previous task](05-implement-test-pure-scheduler.md) | [Task index](../tasks.md) | [Next task](07-add-typed-qa-exact-matching.md)
