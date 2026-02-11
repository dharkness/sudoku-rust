# AGENTS.md

Guidance for agentic coding assistants working in this repo.

## Repository Summary

- CLI Sudoku generator/solver/player written in Rust (edition 2021).
- Core modules: `layout/`, `puzzle/`, `solve/`, `io/`, `commands/`, `build/`.
- Heavy use of bitset-style types (`CellSet`, `DigitSet`, `Bit`).

## Build, Test, Lint

- Build (release): `bin/build.sh`
- Build (release, direct): `cargo build --release`
- Build (debug): `cargo build`
- Run app: `./sudoku-rust`
- Show CLI help: `./sudoku-rust help`
- Run tests: `cargo test`
- Run a single test: `cargo test test_name`
- Run tests in a module: `cargo test module_name::`
- Run a specific solver test: `cargo test solve::algorithms::test_name` (example pattern)
- Lint with Clippy: `cargo clippy`

## Formatting

- Use `rustfmt` defaults; no `rustfmt.toml` is present.
- Keep formatting consistent with existing code.
- Use `#[rustfmt::skip]` only for dense arrays/bitsets where alignment matters.
- Prefer one item per line in long arrays to keep diffs readable.
- Keep doc comment paragraphs wrapped like neighboring files.

## Imports and Module Layout

- Import order: standard library, external crates, then `crate::` or `super::`.
- Separate import groups with a blank line.
- Use explicit imports for frequently used types (e.g., `Cell`, `Digit`).
- Use `super::*` inside solver algorithm submodules when consistent with neighbors.
- Re-export public APIs in module root files (see `src/puzzle.rs`, `src/io.rs`).
- Prefer `pub use` in top-level module files rather than deep re-exports.

## Naming and Conventions

- Types/traits/enums use `CamelCase` (`Board`, `DigitSet`, `Strategy`).
- Functions, methods, and modules use `snake_case`.
- Constants use `SCREAMING_SNAKE_CASE`.
- Prefer descriptive names; avoid one-letter names unless in tiny iterators.
- Use domain types (`Cell`, `Digit`, `House`) over raw integers.
- Convert domain types with helpers like `.usize()` when indexing arrays.
- Keep functions small and focused; solver functions usually return `Option<Effects>`.
- Avoid public fields unless required by other modules.

## Types and Data Structures

- Use bitset operations for speed: `|` (union), `&` (intersection), `-` (difference).
- Favor `CellSet`/`DigitSet` operations over manual loops.
- Prefer `const fn` where values are compile-time friendly.
- Use `Value::none()` and `DigitSet::full()` for initialization.
- Avoid heap allocations in hot solver paths.
- Prefer iterators that borrow from the board rather than collecting.

## Error Handling

- Avoid `panic!` in library code; use `puzzle::Error` or return `Option`/`Result`.
- Solving algorithms typically return `Option<Effects>` when deductions exist.
- Use `Effects` to collect `Action`s and errors rather than side effects.
- CLI commands may call `exit(1)` for invalid user input or fatal failures.
- Implement `fmt::Display` for new error enums.
- Prefer early returns on invalid state instead of deeply nested matches.

## Actions, Effects, and Strategies

- Create actions with `Action::new`/`Action::new_set` and attach to `Effects`.
- Use `Strategy` enum to label all deductions and solver steps.
- Prefer `effects.add_action(action)` and early-return when `single` is set.
- Use `Verdict::Related`/`Verdict::Primary` for clue highlighting.
- Keep strategy implementations pure; avoid mutating the board directly.

## Solver Algorithm Conventions

- Name deduction entry points `find_*`.
- Signature uses `board: &Board, single: bool`.
- Initialize with `let mut effects = Effects::new();`.
- Early-return `Some(effects)` when `single` and an action was added.
- Return `None` when no actions exist.
- Prefer `super::*` imports like other algorithms.
- Use `Action::new_set`/`Action::new` for solver deductions.
- Avoid allocating temporary vectors; iterate over sets directly.

## CLI Output

- Use `println!` for normal output and `eprintln!` for errors.
- Use `format_runtime`/`format_number` helpers.
- Avoid logging in library modules.
- Keep help strings aligned and in doc comments.
- Prefer user-facing phrases consistent with existing output.

## Safety and Allocation

- Avoid `unsafe` unless absolutely required.
- Prefer stack arrays for fixed-size board data.
- Use `Copy` types for small structs.
- Avoid `Rc`/`Arc` unless required for sharing.
- Do not introduce new global mutable state.

## Documentation

- Module-level docs use `//!` at the top of the file.
- Public items use `///` doc comments and explain intent.
- Keep command docs verbose for CLI help (see `src/main.rs`).
- Prefer examples in docs for puzzle formats when adding parsers/formatters.

## CLI and Command Modules

- Argument parsing uses `clap` derive macros.
- Keep `Args` structs private fields unless needed elsewhere.
- Use `#[clap(short, long)]` consistently for flag naming.
- Print human-friendly output with `println!`/`eprintln!`.
- Favor `format_runtime`/`format_number` for CLI output consistency.

## Testing Patterns

- Tests use `cargo test` with standard Rust test harness.
- Keep tests deterministic; avoid randomness without fixed seeds.
- Prefer focused unit tests per module over broad integration tests.
- Place helper utilities in `src/testing.rs` if shared.
- Name tests after the scenario (e.g., `test_parses_wiki_format`).

## Performance Expectations

- Solver routines are performance-critical; minimize allocations.
- Use iterator chains but avoid excessive cloning in tight loops.
- Favor bitset operations over per-cell branching.
- Avoid repeated `board` queries inside nested loops when values can be cached.
- Use `CellSet` intersections to prune candidate searches early.

## Lints and Clippy

- Respect existing `#![allow(...)]` in `src/main.rs`.
- Do not add new lint allowances without a clear reason.
- Prefer explicit type annotations when inference is unclear.
- Keep `Debug`/`Clone`/`Copy` derives aligned with existing patterns.

## File Organization Notes

- `src/layout/` contains fundamental grid types and bitsets.
- `src/puzzle/` owns board state, actions, effects, and errors.
- `src/solve/` aggregates strategies and algorithm modules.
- `src/io/` handles parsing, formatting, and progress output.
- `src/commands/` contains CLI command implementations.
- `src/build/` handles puzzle generation.

## External Rules

- No `.cursorrules`, `.cursor/rules/*`, or `.github/copilot-instructions.md` found.

## Contribution Tips

- Prefer minimal, targeted changes that match existing patterns.
- Keep whitespace and alignment consistent with nearby code.
- Update documentation when behavior or CLI output changes.
- If adding a new solver algorithm, register it in `src/solve/algorithms`.
- Maintain the ordering of strategies by difficulty.
- Use `Action`/`Effects` plumbing rather than direct board mutation.

## Example Command Cheatsheet

- Run interactive player: `./sudoku-rust`
- Generate puzzle: `./sudoku-rust create`
- Solve puzzle: `./sudoku-rust solve --check`
- Brute force: `./sudoku-rust bingo`
- Extract patterns: `./sudoku-rust extract`
- Find solvable clues: `./sudoku-rust find`
- Profile solver: `./sudoku-rust profile`
