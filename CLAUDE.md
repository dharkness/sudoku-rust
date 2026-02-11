# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Test Commands

```bash
# Build (release mode)
bin/build.sh
# or directly:
cargo build --release

# Run tests
cargo test

# Run a single test
cargo test test_name

# Run tests for a specific module
cargo test module_name::

# Lint
cargo clippy

# Run the application
./sudoku-rust          # Interactive player (default)
./sudoku-rust help     # Show available commands
```

## Architecture Overview

This is a Sudoku puzzle generator, solver, and interactive player. The codebase uses bitset-based data structures throughout for performance.

### Module Structure

- **layout/** - Core data structures representing the puzzle grid (cells, houses, coordinates)
- **puzzle/** - Board state management, actions, effects, and strategy definitions
- **solve/** - 28 solving algorithms organized by difficulty
- **io/** - Puzzle parsing and formatting (packed, wiki, grid formats)
- **commands/** - CLI commands (play, create, solve, bingo, extract, find)
- **build/** - Puzzle generation

### Key Data Structures

All built on efficient bitset representations:

- **`Bit`** (u128) - Single bit for cell position
- **`Cell`** (u8) - Cell index 0-80
- **`CellSet`** (u128) - Bitset of cells with O(1) set operations
- **`Digit`** (u8) - Digit value 1-9
- **`DigitSet`** (u16) - Bitset of candidate digits
- **`House`** - Row, column, or block (3x3 box)
- **`Board`** - Central state with multiple redundant views for fast lookups:
  - `values: [Value; 81]` - Actual digits
  - `candidate_digits_by_cell: [DigitSet; 81]` - Candidates per cell
  - `candidate_cells_by_digit: [CellSet; 9]` - Cells per candidate digit

### Solving Flow

Algorithms are tried in order of increasing difficulty (Trivial → Basic → Tough → Diabolical → Extreme). Each returns `Effects` containing `Action`s that modify the board. The `Changer` applies actions and automatically removes candidates from peer cells.

Key solving algorithm files are in `src/solve/algorithms/`:
- Singles (naked/hidden)
- Pairs/Triples/Quads (naked/hidden)
- Fish patterns (X-Wing, Swordfish, Jellyfish)
- Wing patterns (Y-Wing, XYZ-Wing, WXYZ-Wing)
- Unique rectangles and chains
- Brute force (Bowman's Bingo)

### Patterns and Conventions

**Bitset operations:**
```rust
let union = set1 | set2;
let intersection = set1 & set2;
let difference = set1 - set2;
```

**Action construction:**
```rust
let mut action = Action::new(strategy);
 action.erase_cells(cells, digit);
 action.clue_cells_for_digit(...);
effects.add_action(action);
```

**Board updates are immutable-style** - operations return new boards rather than modifying in place.

**Every cell belongs to exactly 3 houses** (1 row, 1 column, 1 block). Algorithms iterate over houses to find patterns.
