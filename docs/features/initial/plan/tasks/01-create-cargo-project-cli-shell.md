# Task 1: Create the Cargo Project and CLI Shell

**Description:** Initialize the binary project, add the chosen dependencies, and create a small `validate <root>` command. At this stage it checks that the root exists and is a directory. Later tasks replace that limited check with full deck validation.

**Implementation steps:**

1. Confirm `rustc --version` and `cargo --version` work. Install Rust with the official rustup instructions if either command is missing.
2. Run `cargo init --bin .` in the project root. Keep the planning documents already present.
3. Add dependencies with:
   ```text
   cargo add ratatui crossterm serde_yaml anyhow
   cargo add serde --features derive
   cargo add clap --features derive
   cargo add rusqlite --features bundled
   cargo add tempfile --dev
   ```
4. In `main.rs`, use `clap::Parser` and a `Subcommand` enum for `Validate { root: PathBuf }`. Add the global `--db` option with `.flashcards.db` as its default even though later commands will use it.
5. Add `deck.rs` with a function that checks the root using `std::fs::metadata`. Return an `anyhow::Error` that names the invalid path.
6. Have `main` return `anyhow::Result<()>`. Print a clear success message for an existing directory. Add `.gitignore` entries for `/target` and `.flashcards.db`.

**Rust focus:** `cargo init` creates a package and `src/main.rs` is its binary entry point. `Result<T, E>` makes failure explicit. The `?` operator returns an error early instead of requiring nested `match` expressions. `PathBuf` owns a filesystem path, while `&Path` borrows one.

**Acceptance criteria:**

- [ ] `cargo run -- validate .` accepts an existing directory and prints a useful result.
- [ ] A missing path or a file used as the root produces an error containing that path.
- [ ] `cargo run -- --help` shows `validate` and the global `--db` option.

**Verification:**

- [ ] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo build`.
- [ ] Run `cargo test` even though the first suite may contain only simple path tests.
- [ ] Manual check: run `cargo run -- validate .` and `cargo run -- validate /definitely/missing` and inspect both outputs.

**Dependencies:** None.

**Files likely touched:** `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/deck.rs`, `.gitignore`.

**Estimated scope:** Medium, 5 files, about 1 to 2 hours including Rust installation checks.

[Task index](../tasks.md) | [Next task](02-parse-validate-yaml-card-contract.md)
