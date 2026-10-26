# Terminal Flashcards Task Index

This index implements the [implementation plan](README.md). Work in order. Each task should take one focused session, touch about five files or fewer, and leave a buildable app. Do not start exam tasks until the core app complete checkpoint passes.

Run commands from the project root after Task 1 creates the Cargo project. Focused test filters assume unit tests live beside their modules. Adjust a filter if a test module has a different name.

## Foundation, Tasks 1-3

1. [Create the Cargo Project and CLI Shell](tasks/01-create-cargo-project-cli-shell.md)
2. [Parse and Validate the YAML Card Contract](tasks/02-parse-validate-yaml-card-contract.md)
3. [Discover Decks Recursively and Build the Hierarchy](tasks/03-discover-decks-build-hierarchy.md)

### Checkpoint: Foundation Complete

- [ ] All YAML contract and recursive-discovery tests pass.
- [ ] `cargo run -- validate decks` validates the sample hierarchy without silently skipping errors.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo build` all pass.

## First Usable Slice, Tasks 4-6

4. [Add SQLite Migrations, Scheduler State, and Review Logs](tasks/04-add-sqlite-scheduler-review-logs.md)
5. [Implement and Test the Pure Scheduler](tasks/05-implement-test-pure-scheduler.md)
6. [Deliver End-to-End Q&A Reveal Study](tasks/06-deliver-qa-reveal-study.md)

### Checkpoint: First Usable Study Slice

- [ ] Q&A reveal, self-grade, schedule persistence, and review logging work end to end.
- [ ] Due + unseen and full-deck selection have passing tests and manual restart checks.
- [ ] Terminal restoration works on normal, error, and panic paths; all automated quality commands pass.

## Core Completion, Tasks 7-10

7. [Add Typed Q&A Attempts and Shared Exact Matching](tasks/07-add-typed-qa-exact-matching.md)
8. [Add Fill Blank Study](tasks/08-add-fill-blank-study.md)
9. [Add Multiple Choice Study](tasks/09-add-multiple-choice-study.md)

### Checkpoint: All Core Card Types Work

- [ ] Q&A, fill blank, and multiple choice all reach reveal and manual grading.
- [ ] Typed Q&A and fill blank matching has no automatic grading path.
- [ ] All card types persist schedules and logs; focused tests and full quality commands pass.

10. [Harden the Core and Add Sample Decks](tasks/10-harden-core-add-sample-decks.md)

### Core App Complete Checkpoint: Required Before Exams

- [ ] Every core acceptance criterion in Tasks 1 through 10 passes, including all card types, typed attempts, both selection modes, and all four grades.
- [ ] Scheduler and review data survive restart; full-deck study updates schedules; no content is duplicated into SQLite.
- [ ] Validation errors are actionable, terminal state restores on exit and failure, and sample decks validate.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo build` are clean.
- [ ] Do not begin Task 11 until every box above is checked.

## Exams, Tasks 11-13

11. [Add Typed Exam Scoring for Every Card Type](tasks/11-add-typed-exam-scoring.md)
12. [Persist Completed Exams Atomically](tasks/12-persist-completed-exams-atomically.md)

### Checkpoint: Scored and Persisted Exams

- [ ] All card types are typed and scored with the shared matcher.
- [ ] Completed exams and snapshots survive database reopen; abandoned or failed exams are not partially saved.
- [ ] Study schedules and review logs are unchanged by exams, and all quality commands pass.

13. [Add Exam History List and Detail Views](tasks/13-add-exam-history-views.md)

## Final Verification, Task 14

14. [Final Verification and User Documentation](tasks/14-final-verification-user-documentation.md)

### Final Checkpoint: Application Complete

- [ ] All acceptance criteria and verification boxes in Tasks 1 through 14 are complete.
- [ ] The final quality gate passes: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo build`.
- [ ] Sample decks validate, all core study flows work, and scheduler and review data survive restart.
- [ ] Typed exams score correctly, completed history survives restart, abandoned exams are absent, and exams never alter study schedules.
- [ ] Terminal state restores on normal exit, error, and panic paths, and final documentation matches observed behavior.
