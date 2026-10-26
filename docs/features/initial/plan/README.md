# Terminal Flashcards Implementation Plan

## Overview

Build a small Rust terminal application for studying flashcards stored in YAML files. Directory and file paths define the deck hierarchy. Normal study supports reveal-based recall, optional typed attempts, and self-assigned grades. A simple scheduler stores due dates and review history in SQLite.

Exam mode is a later phase. It uses typed answers for every card type, scores responses automatically, and saves completed exam history. It is separate from scheduling.

The card files are the source of truth for card content. SQLite owns learning state and activity records, not a copy of the card collection.

## Goals

- Recursively load UTF-8 `.yaml` and `.yml` files from a deck root.
- Turn relative paths into selectable deck groups, including all descendant cards.
- Support `qa`, `fill_blank`, and `multiple_choice` cards.
- Offer due plus unseen study and full-deck study.
- Support reveal or typed attempts for Q&A and fill blank cards.
- Present multiple choice options for one selection during normal study.
- Let the user choose Again, Hard, Good, or Easy after every normal-study attempt.
- Persist scheduler state and review logs in SQLite.
- Add typed, automatically scored exams only after the core study app works.
- Preserve enough exam data to show list and detail history after card files change.
- Keep the code small and understandable for a first Rust project.

## Non-goals

- Exact Anki, SM-2, or FSRS compatibility.
- Anki imports, other imports, sync, or remote storage.
- Multiple-select questions.
- Fuzzy, phonetic, or semantic answer matching.
- Rich session customization, plugins, async processing, or repository abstractions.
- Media, shuffling, tags, and other extras. These may be considered later, but are not part of this plan.

## Command-Line Interface

The binary uses these commands:

```text
cargo run -- validate decks
cargo run -- study decks
cargo run -- exam decks
cargo run -- exam-history
cargo run -- exam-history 12
```

`decks` is the deck root. The database defaults to `.flashcards.db` in the current directory. A global `--db` option selects another path:

```text
cargo run -- --db /tmp/review.db study decks
```

`validate` only reads YAML files. The other commands open SQLite as needed. `exam` and `exam-history` are added in the exam phase.

## User Flows

### Validate decks

1. Run `cargo run -- validate decks`.
2. The app discovers deck files recursively and parses every one.
3. It reports the number of valid deck files and cards.
4. Any discovery, UTF-8, YAML, or card validation error fails the command with the relevant file path.

### Normal study

1. Run `cargo run -- study decks`.
2. Select a directory group or an individual file deck. A directory group includes cards from all descendant files.
3. Choose **due + unseen** or **full deck**.
4. Answer each selected card using the controls for its type.
5. Reveal the expected answer, then choose Again, Hard, Good, or Easy.
6. The app saves the grade, review log, and new schedule before advancing.
7. At the end, the app shows a small completion summary. Exiting early keeps reviews already saved.

### Exam

Exam work starts only after the core app complete checkpoint.

1. Run `cargo run -- exam decks`.
2. Select a directory group or file deck. The exam includes all cards in that selection, regardless of study schedules.
3. Type a response for every card. Multiple choice also requires typing the full expected option, not a number.
4. Submit each response once. There is no early answer reveal, correctness feedback, or recall grade.
5. Finish the exam. The app saves the completed session and its answer snapshots in one transaction.
6. The final screen shows correct count, total count, and percentage. Abandoned exams are not listed as completed history.

An empty card selection cannot start an exam.

### Exam history

1. Run `cargo run -- exam-history` to list completed sessions newest first.
2. Show session id, selected deck path, completion time, correct count, total count, and percentage.
3. Run `cargo run -- exam-history 12` to show that session and its answers in original order.
4. Detail output includes the card prompt, card type, typed response, expected answer, and correctness. Stored snapshots keep history readable if YAML later changes.

## YAML Contract

Each file contains a top-level object with a `cards` sequence. A bare top-level list is invalid. The top-level object permits metadata in a later version, but only `cards` has behavior now.

A complete example:

```yaml
cards:
  - id: immutable-binding-keyword
    type: qa
    prompt: What keyword creates a binding that is immutable by default?
    answer: let

  - id: owner-blank
    type: fill_blank
    prompt: Each value in Rust has an _____.
    answer: owner

  - id: valid-owner-count
    type: multiple_choice
    prompt: How many owners can a value have at one time under the basic ownership rule?
    options:
      - Zero
      - One
      - Unlimited
    answer: One
```

### Card shapes

Q&A and fill blank use the same fields:

```yaml
id: stable-string-id
type: qa # or fill_blank
prompt: Question text
answer: Expected answer text
```

Multiple choice uses:

```yaml
id: stable-string-id
type: multiple_choice
prompt: Question text
options:
  - First option
  - Second option
answer: First option
```

The `answer` for multiple choice is the correct option text. It is not an index.

### Identity and deck hierarchy

- Every card has an explicit, nonempty string `id`.
- An id is unique within its YAML file. Different files may use the same id.
- Do not change an id after studying the card. Do not reuse an old id for different content.
- Scheduler identity is the pair `(relative_deck_path, card_id)`, not the card's position in `cards`.
- `relative_deck_path` includes the filename and extension and uses forward slashes in persisted keys. This distinguishes `topic.yaml` from `topic.yml`.
- Reordering cards does not affect scheduler state. Moving or renaming a file changes its identity.
- Card ids are internal identity fields. The UI does not show them as answers.

For `decks/rust/ownership.yaml`, the relative path is `rust/ownership.yaml`. Its conceptual hierarchy is **Rust > Ownership**. Directory names form parent groups and the filename stem forms the file deck label. The original relative path remains the persistence key.

Both directory groups and file decks are selectable. Selecting `rust` includes `rust.yaml`, if present, and every valid deck below `rust/`. Discovery sorts relative paths so sessions have deterministic card order. YAML card order is preserved within each file. Shuffling is out of scope.

## Validation Rules

Loading fails on the first error with path context. The app does not silently skip bad files.

- The deck root must exist and be a readable directory.
- Recursion includes files with lowercase `.yaml` or `.yml` extensions. Other files are ignored.
- Directory symlinks are not followed, which prevents recursive loops. Filesystem errors include the affected path.
- Deck files must contain valid UTF-8 and valid YAML.
- The top level must be an object containing a `cards` sequence.
- Unknown card types and missing required fields are errors.
- `id`, `prompt`, and `answer` must be strings that are nonempty after trimming surrounding whitespace.
- Card ids must be unique within each file.
- Multiple choice `options` must contain at least two strings.
- Every option must be nonempty after trimming and unique by exact string comparison.
- A multiple choice `answer` must exactly equal one untrimmed option string. Authors should not add decorative surrounding whitespace.
- Filesystem, YAML, and validation errors identify the file. Card-specific errors also identify the card id when available.

A deck with zero cards is valid YAML, but cannot start a study session or exam. Study and exam selection must reject an empty resulting card set with a useful message.

## Study Behavior

### Deck selection

**Due + unseen** selects:

- Cards with no scheduler row, which are unseen.
- Reviewed cards whose `due_at` is less than or equal to the session start time.

It excludes reviewed cards due in the future.

**Full deck** selects every card in the chosen directory group or file deck, including new and future-due cards. Grades in this mode update schedules and write review logs exactly as in due + unseen mode.

The card set is captured when the session starts. Each selected card appears once in that session. An Again grade schedules it 10 minutes later but does not insert it again into the current session.

### Card interaction

- `qa`: reveal directly or make a typed attempt before reveal.
- `fill_blank`: reveal directly or make a typed attempt before reveal. The prompt should visibly show the blank.
- `multiple_choice`: move through the options and select one. Selection reveals the correct answer.

After reveal, every card type shows the expected answer and accepts grades `1` through `4`. Typed Q&A and fill blank attempts may also show **match** or **no match**. This status is informational. The user always chooses the normal-study grade.

Full-deck study and typed attempts never bypass scheduler writes. There is no automatic normal-study grade.

## Exact Typed Matching

Study and exams use one shared pure matching function:

1. Trim surrounding Unicode whitespace with Rust `str::trim`.
2. Convert both strings with Rust Unicode-aware `str::to_lowercase`.
3. Compare the resulting strings exactly.

Everything else remains significant, including internal whitespace, punctuation, accents, and spelling. There is no fuzzy or semantic matching. Unicode lowercase conversion is not full case folding or Unicode normalization. For example, `Straße` matches `straße`, but `ß` does not match `ss`.

Typed multiple choice exams compare the typed response with the expected option string using the same function. Authors must write prompts with one unambiguous expected response. The matcher does not accept paraphrases.

## Scheduler Behavior

The scheduler is Anki-inspired only. It does not claim exact SM-2, Anki, or FSRS compatibility.

All times are integer Unix timestamps in seconds. Intervals are integer seconds. Storage does not require a date crate.

### State

A missing scheduler row represents an unseen card with this logical state:

| Field | Initial value |
| --- | ---: |
| `ease` | 2.5 |
| `interval_seconds` | 0 |
| `repetitions` | 0 |
| `due_at` | none |

The first review creates the persisted row. Every review uses the supplied `reviewed_at` timestamp and sets `due_at = reviewed_at + new_interval_seconds`.

### Grade transitions

Let `I` be the old interval in seconds, `E` the old ease, and `R` the old repetitions. Round calculated floating-point intervals to the nearest whole second. Apply the minimum ease of 1.3 after every transition.

| Grade | Value | New ease | New repetitions | New interval |
| --- | ---: | --- | --- | ---: |
| Again | 1 | `max(1.3, E - 0.20)` | 0 | 600 seconds |
| Hard | 2 | `max(1.3, E - 0.15)` | `R + 1` | 43,200 seconds if `R = 0`; otherwise `max(86,400, round(I * 1.2))` |
| Good | 3 | `E` | `R + 1` | 86,400 seconds if `R = 0`; otherwise `max(86,400, round(I * E))` |
| Easy | 4 | `E + 0.15` | `R + 1` | 345,600 seconds if `R = 0`; otherwise `max(345,600, round(I * E * 1.3))` |

This gives first-pass Hard, Good, and Easy intervals of 12 hours, 1 day, and 4 days. For learned cards, Good and Easy grow using ease while Hard grows more slowly. Again always uses the fixed 10-minute relearning delay.

The scheduler is a pure function over old state, grade, and review time. It does not read the clock, terminal, YAML, or SQLite. Tests cover each initial grade, subsequent growth, Again reset, rounding, due timestamps, and the ease floor.

### Persistence semantics

For each normal-study grade, one SQLite transaction:

1. Inserts or updates the scheduler row keyed by source deck path and card id.
2. Inserts a review log with grade, review time, and resulting schedule.
3. Commits before the UI advances.

If content is removed from YAML, its historical rows may remain. They are not shown in study because discovery starts from current YAML. Recreating a path and id can reconnect old state, so ids must not be reused for unrelated cards.

## SQLite Data Ownership and Schema

YAML owns card ids, types, prompts, options, answers, and deck paths. SQLite owns scheduler state, review logs, and exam history. There is no `cards` table and no bulk duplication of YAML content.

Use migration versions through SQLite `user_version`. Enable foreign keys whenever a connection opens.

### `scheduler_state`

- `deck_path TEXT NOT NULL`
- `card_id TEXT NOT NULL`
- `due_at INTEGER NOT NULL`
- `interval_seconds INTEGER NOT NULL`
- `ease REAL NOT NULL`
- `repetitions INTEGER NOT NULL`
- Primary key: `(deck_path, card_id)`

### `review_logs`

- `id INTEGER PRIMARY KEY`
- `deck_path TEXT NOT NULL`
- `card_id TEXT NOT NULL`
- `grade INTEGER NOT NULL`, constrained to 1 through 4
- `reviewed_at INTEGER NOT NULL`
- `resulting_due_at INTEGER NOT NULL`
- `resulting_interval_seconds INTEGER NOT NULL`
- `resulting_ease REAL NOT NULL`
- `resulting_repetitions INTEGER NOT NULL`

### `exam_sessions`

- `id INTEGER PRIMARY KEY`
- `selected_deck_path TEXT NOT NULL`
- `started_at INTEGER NOT NULL`
- `completed_at INTEGER NOT NULL`
- `total_cards INTEGER NOT NULL`
- `correct_cards INTEGER NOT NULL`

`total_cards` must be positive and `correct_cards` must be between zero and the total.

### `exam_answers`

- `id INTEGER PRIMARY KEY`
- `exam_session_id INTEGER NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE`
- `position INTEGER NOT NULL`
- `deck_path TEXT NOT NULL`
- `card_id TEXT NOT NULL`
- `card_type TEXT NOT NULL`
- `prompt TEXT NOT NULL`
- `typed_response TEXT NOT NULL`
- `expected_answer TEXT NOT NULL`
- `is_correct INTEGER NOT NULL` as a checked boolean
- `answered_at INTEGER NOT NULL`
- Unique key: `(exam_session_id, position)`

These rows are snapshots of attempted cards, not a second live card store. The prompt, typed response, expected answer, type, and result are sufficient for the planned detail view.

The final percentage is `correct_cards / total_cards * 100`, calculated as floating point. Display it to one decimal place. Zero-card exams are rejected before session creation.

## TUI Controls and Terminal Safety

### Deck and mode selection

- `Up`/`Down` or `j`/`k`: move.
- `Enter`: choose.
- `d`: due + unseen when choosing a study mode.
- `f`: full deck when choosing a study mode.
- `Esc`: go back or exit.
- `Ctrl+C`: exit immediately through normal cleanup.

### Normal study

For Q&A and fill blank before reveal:

- `Space` or `Enter`: reveal without typing.
- `t`: open the typed-answer editor.
- `Esc`: exit the session.

In a typed-answer editor:

- Printable characters edit the response; `Backspace` deletes.
- `Enter`: submit, show match status, and reveal.
- `Esc`: cancel the attempt and return to the prompt without grading.
- `Ctrl+C`: exit through normal cleanup.

For multiple choice before reveal:

- `Up`/`Down` or `j`/`k`: move the selected option.
- `Enter`: select and reveal.
- `Esc`: exit the session.

After reveal for every type:

- `1`: Again.
- `2`: Hard.
- `3`: Good.
- `4`: Easy.
- `Esc`: exit without grading the current card.

### Exam

- Type the response and use `Backspace` to edit before submission.
- `Enter`: submit the current response and advance.
- Multiple choice shows its options for reference, but requires typing the full option text.
- `Esc`: ask for confirmation to abandon the incomplete exam.
- `Ctrl+C`: exit through normal cleanup.
- Before the final screen, save the completed session and all answers. The final screen shows score and percentage, then returns to the command prompt.

The TUI owns a terminal guard. Startup enables raw mode and enters the alternate screen. Every normal return, error return, or panic disables raw mode, leaves the alternate screen, and restores the cursor. Keep terminal setup and restoration in one place so no feature implements its own cleanup.

## Exam Mode and History

Exam mode reuses deck discovery, hierarchy selection, and the exact matching function. It does not query scheduler state and does not write scheduler state or review logs.

All cards in the selected directory group or file deck are included in deterministic discovery order. Due dates, ease, repetitions, and prior grades have no effect. An empty response is allowed as an attempt. Since expected answers are nonempty, an empty response is incorrect.

Each response is scored once on submission. The app does not reveal correctness until the final summary. Completed exams are persisted atomically with every answer. A failure to save is reported as an error, and the session is not presented as successfully saved.

History uses stored snapshots rather than reloading current YAML. This preserves the original questions and expected answers. Study scheduling remains unchanged by starting, completing, abandoning, or browsing an exam.

## Module and Dependency Decisions

Use these modules and no additional abstraction without a current caller:

| Module | Responsibility |
| --- | --- |
| `main.rs` | CLI parsing, module declarations, command dispatch, top-level errors |
| `model.rs` | Card and deck types, grades, scheduler state, exam records, shared answer matching |
| `deck.rs` | Recursive discovery, YAML decoding, validation, hierarchy construction |
| `scheduler.rs` | Pure grade transitions and due calculations |
| `storage.rs` | SQLite migrations, state writes, review logs, exam persistence and queries |
| `app.rs` | Normal-study selection and session orchestration |
| `ui.rs` | Ratatui screens, input handling, terminal guard |
| `exam.rs` | Exam orchestration, scoring flow, and history presentation, added only in the exam phase |

Avoid trait-based repositories, async, plugin systems, and service layers. Functions and concrete structs are enough.

Dependencies:

- `ratatui` and `crossterm` for the TUI and terminal events.
- `serde` with `derive` and `serde_yaml` for YAML.
- `rusqlite` with `bundled` for SQLite without a system library requirement.
- `clap` with `derive` for commands.
- `anyhow` for application-level error context.
- `tempfile` as a dev dependency only if useful for filesystem and SQLite tests.

Use `std` for paths, time, recursion, and string processing. Do not add a date crate for Unix timestamps or a matching crate for lowercase comparison.

## Dependency Graph and Vertical Slices

```text
CLI and terminal startup
    |
    v
YAML model and validation -> recursive deck hierarchy
    |
    v
SQLite state + pure scheduler
    |
    v
Q&A reveal -> self-grade -> persisted schedule
    |
    +-> typed Q&A -> fill blank -> multiple choice
    |
    v
Core app complete checkpoint
    |
    v
Typed exam scoring -> atomic exam persistence -> history
```

Foundations come first because every flow needs valid decks and reliable persistence. The first study slice then connects discovery, selection, UI, scheduling, and SQLite for one card type. This produces a usable app before adding typed attempts and other card types. It also exposes integration problems early without building exam features on an unstable core.

## Implementation Phases

1. **Foundation, Tasks 1-3:** Cargo setup, CLI, YAML contract, recursive discovery, and validation.
2. **First usable slice, Tasks 4-6:** SQLite, pure scheduler, deck selection, and persisted Q&A reveal study.
3. **Core completion, Tasks 7-10:** Typed Q&A, fill blank, multiple choice, hardening, and sample decks.
4. **Core app complete checkpoint:** Verify every normal-study flow, persistence across restart, terminal restoration, formatting, lints, tests, and build. Exam work cannot begin before this passes.
5. **Exams, Tasks 11-13:** Typed scoring, completed-exam persistence, and history.
6. **Final verification, Task 14:** Full automated and manual verification plus user documentation.

## Risks and Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Changed or reused card ids attach history to the wrong content | Incorrect schedules | Document stable ids, key by path plus id, and never infer identity from order |
| Moving deck files loses the connection to state | Cards appear unseen | Document path identity and retain historical rows |
| One malformed file hides otherwise usable decks | Confusing missing content | Fail loading with file and card context; provide `validate` |
| Typed feedback is mistaken for automatic grading | Incorrect learning state | Keep match display separate and require `1` to `4` in normal study |
| Exam work accidentally changes schedules | Unwanted due-date changes | Keep exam orchestration separate and test that scheduler and review tables are unchanged |
| Terminal remains raw after an error or panic | Unusable shell | Centralize cleanup in a guard, install panic cleanup, and test error paths manually |
| SQLite writes partially complete | Inconsistent history or scheduling | Use transactions for grade writes and completed exams; enable foreign keys |
| Unicode matching expectations exceed lowercase behavior | Surprising mismatches | Document the exact algorithm and test non-ASCII examples |
| YAML edits make old exam details unreadable | Lost context | Store answer snapshots for attempted exam cards |
| Scope grows into a general spaced-repetition platform | Delayed usable app | Keep the listed non-goals and add abstractions only for current callers |

## Verification Strategy

Use focused tests while developing each task, then run the full suite at checkpoints.

- **Deck unit tests:** all card shapes, top-level object requirement, empty fields, duplicate ids, invalid options, wrong multiple choice answer, malformed YAML, and path-rich errors.
- **Discovery tests:** nested `.yaml` and `.yml` files, ignored extensions, deterministic order, directory descendants, and symlink-safe recursion.
- **Matcher tests:** surrounding whitespace, ASCII and Unicode case changes, punctuation, internal whitespace, accents, and examples that distinguish lowercase from full case folding.
- **Scheduler tests:** table-driven initial and learned transitions for all grades, Again reset, ease floor, rounding, and exact due timestamps.
- **Storage tests:** migrations on a temporary database, insert/update behavior, transaction writes, review logs, exam snapshots, ordering, and foreign keys.
- **Selection tests:** unseen, due, future-due, full-deck behavior, descendant inclusion, and empty selections.
- **Exam tests:** all card types use typed matching, score and percentage calculations, zero-card rejection, history snapshots, and no study-table changes.
- **Manual TUI checks:** reveal, typed attempts, option selection, every grade, exit paths, terminal restoration, and restart persistence.

The final quality gate is:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
```

Also validate the sample decks, complete normal-study sessions in both modes, restart to confirm schedules and logs, complete and browse an exam, and confirm the exam leaves study schedules unchanged.

## Task List

The ordered implementation checklist is in [tasks.md](tasks.md). It contains 14 small tasks, dependencies, beginner-focused Rust notes, verification commands, and checkpoints. This plan defines the behavior those tasks implement.
