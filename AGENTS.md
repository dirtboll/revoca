# Deck Authoring Instructions

These instructions are for AI agents designing and writing flashcard decks for the terminal flashcard app. They are not implementation instructions for the app.

Keep card content in YAML. Scheduling state, review grades, and exam results live in SQLite and must never be added to deck files.

## Directory and File Layout

- Put decks below the deck root supplied to the app, commonly `decks/`.
- Use UTF-8 files with lowercase `.yaml` or `.yml` extensions.
- The app loads deck files recursively. Other file extensions are ignored.
- Relative paths define the hierarchy. For example, `decks/rust/ownership.yaml` is conceptually **Rust > Ownership**.
- Selecting a directory includes cards from its descendant deck files. Selecting a file deck includes that file's cards.
- Moving or renaming a file changes its scheduler identity. Preserve paths when possible.
- Keep each file focused on one topic. Use subdirectories for broader subjects.

## Required Document Shape

Every deck file has a top-level object with a `cards` sequence. Do not use a bare top-level list.

The top-level object allows metadata in a future app version. Only `cards` has behavior now. Tags, descriptions, difficulty fields, and other metadata do not currently affect study or exams.

## Stable Card IDs

Every card requires an explicit string `id` that is unique within its YAML file.

- Use short, stable, content-specific ids such as `owner-blank` or `valid-owner-count`.
- Do not change an id after the card may have been studied.
- Do not reuse an old id for a different question.
- Reordering cards is safe because position is not their identity.
- The same id may appear in a different file. Scheduler identity combines the relative file path and card id.
- Ids are internal identifiers. The app does not show them as answers.

When editing an existing deck, preserve existing ids unless the user explicitly wants learning state detached from a card.

## Card Types

### Q&A

Required fields: `id`, `type`, `prompt`, and `answer`.

```yaml
id: immutable-binding-keyword
type: qa
prompt: What keyword creates a binding that is immutable by default?
answer: let
```

Use `qa` for a direct question with one concise expected answer.

### Fill Blank

Required fields: `id`, `type`, `prompt`, and `answer`.

```yaml
id: owner-blank
type: fill_blank
prompt: Each value in Rust has an _____.
answer: owner
```

Make every blank visible in the prompt. Use a consistent marker such as `_____`. The prompt must make the missing text clear without showing the answer elsewhere.

### Multiple Choice

Required fields: `id`, `type`, `prompt`, `options`, and `answer`.

```yaml
id: valid-owner-count
type: multiple_choice
prompt: How many owners can a value have at one time under the basic ownership rule?
options:
  - Zero
  - One
  - Unlimited
answer: One
```

Provide at least two unique, nonempty options. Set `answer` to the full correct option string, not its position. The answer must exactly equal one option, including capitalization, punctuation, and surrounding whitespace.

Use only one correct option. Do not author multiple-select questions.

## Complete Example

This is one valid file with all three card types:

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

## Validation Constraints

Before delivering a new or edited deck, check all of these rules:

- The file is valid UTF-8 and valid YAML.
- The top level is an object containing a `cards` sequence.
- Every card has a supported `type`: `qa`, `fill_blank`, or `multiple_choice`.
- Every required field is present.
- `id`, `prompt`, and `answer` are strings and are nonempty after surrounding whitespace is trimmed.
- No id is duplicated within the same file.
- Multiple choice has at least two options.
- Every option is a string, nonempty after trimming, and unique by exact string comparison.
- A multiple choice answer exactly equals one option.

The app fails loading when it finds an invalid deck. It does not silently skip malformed files. Include enough context in your own editing checks to find the file and card quickly.

Run this command from the project root once the app is implemented:

```text
cargo run -- validate decks
```

Replace `decks` if the user has chosen another deck root. Fix every reported error rather than excluding the file or weakening its content schema.

## Authoring for Exact Typed Matching

Typed responses use the same matching rule in study and exams:

1. Surrounding whitespace is trimmed.
2. Both strings are converted with Unicode-aware lowercase conversion.
3. The results must match exactly.

Internal whitespace, punctuation, accents, and spelling still matter. There is no fuzzy matching, semantic matching, alternative-answer list, or paraphrase detection. Unicode lowercase conversion is not full case folding.

Write cards so one short response is clearly expected:

- Ask for one fact, term, name, or value at a time.
- Prefer concise answers that are practical to type.
- Specify the requested form when needed, such as singular versus plural or a particular symbol.
- Avoid prompts that allow several reasonable phrasings unless any exact wording is intentionally required.
- If paraphrases are likely, narrow the prompt or rewrite the card. Do not assume the app will recognize equivalent meaning.
- Do not add scheduling or accepted-answer syntax to YAML to work around matching.
- Avoid decorative leading or trailing whitespace in answers and options.
- Use capitalization naturally, but do not rely on capitalization to distinguish two acceptable answers because typed matching ignores case.

For multiple choice, normal study lets the user select an option. Exams require typing the full correct option text. Keep options distinct, concise, and unambiguous when read as typed answers.

## Normal Study and Exam Design

Normal study is for learning:

- Q&A and fill blank support direct reveal or a typed attempt.
- Multiple choice presents options for one selection.
- Typed match status and option correctness are informational.
- After reveal, the user always chooses Again, Hard, Good, or Easy.
- Study grades update the scheduler.

Exams are for typed assessment:

- Every card type requires a typed response, including multiple choice.
- Multiple choice options may be visible for reference, but the user types the full answer rather than selecting a number.
- There is no early reveal, recall grade, or correctness feedback during the exam.
- The app scores exact normalized matches and reports a final percentage.
- Exams do not read or change study schedules.

Design each prompt so it works in a typed exam as well as normal study. A card that depends on vague prose answers may be frustrating even when its content is accurate.

## Editing Checklist

- Preserve the file's hierarchy, existing stable ids, and top-level `cards` object.
- Keep content accurate, concise, and limited to one correct expected answer.
- Check YAML quoting when text contains colons, leading special characters, or other YAML-sensitive content.
- Keep fill blank prompt blanks visibly marked.
- Keep scheduling, grades, scores, and exam history out of YAML.
- Do not claim tags or other top-level metadata currently work.
- Run `cargo run -- validate decks` and report the validation result with the deck changes.
