# Task 2: Parse and Validate the YAML Card Contract

**Description:** Add typed card models and parse one deck file at a time. Enforce the top-level `cards` object, required fields, stable ids, and multiple choice rules before any recursive discovery work.

**Implementation steps:**

1. In `model.rs`, define a `DeckFile` with `cards: Vec<Card>`. Use Serde's internally tagged enum representation so `type: qa`, `type: fill_blank`, and `type: multiple_choice` select the right variant.
2. Give Q&A and fill blank variants `id`, `prompt`, and `answer`. Give multiple choice those fields plus `options: Vec<String>`. Keep the original answer and option strings for display.
3. In `deck.rs`, read a file with `std::fs::read_to_string` and decode it with `serde_yaml`. Use `anyhow::Context` to attach the file path to read and YAML errors.
4. Write one validation function that checks trimmed nonempty ids, prompts, and answers, duplicate ids within the file, and the multiple choice rules from the plan. Require at least two exact-unique, trimmed-nonempty options and an answer exactly equal to one option.
5. Do not add `deny_unknown_fields` to the top-level deck object. Future metadata must remain possible, but do not implement behavior for it.
6. Add inline unit tests using YAML strings. Cover all three valid types, a bare list, empty fields, duplicate ids, too few options, duplicate options, and an answer missing from options.

**Rust focus:** An `enum` can represent card variants with different fields. Serde's `#[serde(tag = "type", rename_all = "snake_case")]` maps YAML to that enum. `Vec<T>` owns a growable sequence. A `HashSet<&str>` or `HashSet<String>` helps detect duplicate ids and options.

**Acceptance criteria:**

- [ ] Valid examples of all three card types parse with their original content intact.
- [ ] Every specified card-level validation rule has at least one failing test.
- [ ] Read, YAML, and validation failures include the deck file path, with a card id when available.

**Verification:**

- [ ] Run `cargo test deck::tests` and `cargo test model::tests` for the focused suites.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Run `cargo test` to confirm the existing path checks and all new parser tests pass together.

**Dependencies:** Task 1.

**Files likely touched:** `src/main.rs`, `src/model.rs`, `src/deck.rs`.

**Estimated scope:** Medium, 3 files, about 1.5 to 2 hours.

[Previous task](01-create-cargo-project-cli-shell.md) | [Task index](../tasks.md) | [Next task](03-discover-decks-build-hierarchy.md)
