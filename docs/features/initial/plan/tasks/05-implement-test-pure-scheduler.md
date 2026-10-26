# Task 5: Implement and Test the Pure Scheduler

**Description:** Encode the exact grade-transition table as a pure function. Use supplied timestamps so every transition and boundary can be tested without sleeping or reading the system clock.

**Implementation steps:**

1. Add `scheduler.rs`. Define a logical initial state with ease 2.5, interval zero, and repetitions zero for an unseen card.
2. Implement `schedule(previous: Option<CardState>, grade: Grade, reviewed_at: i64) -> ScheduledCard` or an equivalent pure signature.
3. Copy the formulas from the plan exactly: Again uses 600 seconds and resets repetitions; first Hard, Good, and Easy use 43,200, 86,400, and 345,600 seconds; learned intervals grow using the specified factors.
4. Clamp ease to at least 1.3 after each transition. Set due time to `reviewed_at + interval_seconds`. Round calculated intervals to the nearest second.
5. Keep this module free of SQLite, terminal, clock, and YAML calls.
6. Add table-driven tests for all first-pass grades, learned Hard/Good/Easy growth, Again after a learned card, ease-floor behavior, and due timestamps. Include at least one case that checks rounding.

**Rust focus:** Pure functions return the same output for the same inputs, which makes tests direct. `Option<CardState>` can distinguish unseen from reviewed cards. Integer and floating-point casts need deliberate types. Associated constants such as `SECONDS_PER_DAY` make formulas easier to audit.

**Acceptance criteria:**

- [ ] Every grade matches the plan's ease, repetition, interval, and due-time formulas for new and learned cards.
- [ ] Again resets repetitions and schedules exactly 10 minutes later; the ease floor is 1.3.
- [ ] Scheduler tests compile and run without a database, filesystem, terminal, or wall-clock dependency.

**Verification:**

- [ ] Run `cargo test scheduler::tests` and inspect the table cases against the plan.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Run `cargo test` to confirm storage and deck behavior remain intact.

**Dependencies:** Task 4.

**Files likely touched:** `src/main.rs`, `src/model.rs`, `src/scheduler.rs`.

**Estimated scope:** Small to medium, 3 files, about 1 to 1.5 hours.

[Previous task](04-add-sqlite-scheduler-review-logs.md) | [Task index](../tasks.md) | [Next task](06-deliver-qa-reveal-study.md)
