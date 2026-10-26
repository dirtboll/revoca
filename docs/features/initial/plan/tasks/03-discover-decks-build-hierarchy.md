# Task 3: Discover Decks Recursively and Build the Hierarchy

**Description:** Connect full file loading to `validate`, recurse through nested directories, and construct deterministic directory and file deck selections from relative paths.

**Implementation steps:**

1. Replace Task 1's directory-only validation with recursive discovery. Use `std::fs::read_dir`, process `.yaml` and `.yml` files, and ignore other extensions.
2. Do not follow directory symlinks. Add path context to directory iteration and metadata failures.
3. For every file, compute its path relative to the supplied root. Convert components to a forward-slash key that includes the extension, and reject paths that cannot be represented consistently as UTF-8.
4. Sort discovered decks by relative path. Preserve card order within each YAML file.
5. Build a small hierarchy structure in which directory groups contain descendant file decks and file decks contain their own cards. Selecting the `rust` directory group must include `rust.yaml`, direct files below `rust/`, and deeper descendants.
6. Make `validate` fail on any bad file and print valid file and card counts when all files pass. Test nested discovery and same-stem sibling, direct-child, and deeper-descendant selection with `tempfile` directories.

**Rust focus:** `read_dir` yields `Result<DirEntry>` items, so both opening and iterating a directory can fail. `Path::strip_prefix` removes the deck root. Iterators can collect into collections, but explicit loops are fine when error context is clearer. Borrowed `&str` values must not outlive the strings they reference.

**Acceptance criteria:**

- [ ] Nested `.yaml` and `.yml` files load in sorted relative-path order, and unrelated extensions are ignored.
- [ ] Selecting the `rust` directory group includes `rust.yaml`, direct files below `rust/`, and deeper descendants; selecting a file returns only that file's cards.
- [ ] `validate` reports aggregate counts on success and exits nonzero with path context on any bad deck.

**Verification:**

- [ ] Run `cargo test deck::tests` for parsing, recursion, and hierarchy tests.
- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Manual check: create `decks/rust/ownership.yaml` from the plan example, run `cargo run -- validate decks`, then introduce one invalid card and confirm the file appears in the error.

**Dependencies:** Task 2.

**Files likely touched:** `src/main.rs`, `src/deck.rs`, `src/model.rs`.

**Estimated scope:** Medium, 3 files, about 1.5 to 2 hours.

[Previous task](02-parse-validate-yaml-card-contract.md) | [Task index](../tasks.md) | [Next task](04-add-sqlite-scheduler-review-logs.md)
