# Sudoku Rust

Sudoku puzzle generator, solver and player console-based application built in Rust.


## Goals

I began this project with a few simple goals.

- [x] Learn Rust
- [x] Have fun building complex algorithms
- [x] Generate new puzzles for my [Sudoku React app](https://github.com/dharkness/sudoku)

Having accomplished those tasks and more, I have added some stretch goals.

- [ ] Build an API to create and solve puzzles for the webapp
- [ ] Store generated puzzles into the webapp's database
- [ ] Serve the React webapp directly from the Rust application


## Running it Yourself

Running the console application is easy.

1. Install Rust
2. Clone the repository
3. Build the application
4. Run it

I recommend using [rustup](https://rustup.rs/) to install Rust
as it's one command and makes keeping up-to-date a breeze.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone git@github.com:dharkness/sudoku-rust.git
cd sudoku-rust
bin/build.sh
./sudoku-rust
```


## Interactive Player

The application will start the interactive player by default
and display a menu of available commands.

```
  O [option]          - view or toggle an option
  N                   - start or input a new puzzle
  C                   - create a new random puzzle

  P [G | K | digit]   - print the full puzzle, givens, knowns, or a single candidate
  X [char]            - export the puzzle with optional character for unsolved cells
  W                   - print URL to play on SudokuWiki.org
  M                   - print the puzzle as a grid suitable for email

  G <cell> <digit>    - set the given (clue) for a cell
  S <cell> <digit>    - solve a cell
  E <cell> <digits>   - erase one or more candidates

  V                   - verify that puzzle is solvable
  F [cell or digit]   - find deductions
  A <num>             - apply a single or all deductions
  B                   - use Bowman's Bingo to solve the puzzle if possible
  R                   - reset candidates based on solved cells
  Z                   - undo last change

  H                   - this help message
  Q                   - quit

      <option> - P, N or H
      <cell>   - A1 to J9
      <digit>  - 1 to 9
      <num>    - any positive number
      <char>   - any single character

  Commands and cells are not case-sensitive - "s a2 4" and "E D8 6" are fine
```

Type `H` or `?` at any time to view this list of commands again.

### Getting Started

It may seem overwhelming at first, but to start quickly use `C` to create
a new random puzzle, or `N` to start a new puzzle from scratch
or by pasting one from another site or application.
Creating a new puzzle will take up to ten seconds to find the fewest
clues possible that still result in a unique solution.

Either way, once you have a puzzle ready, it will be printed to the screen
as a nine-by-nine grid of three-by-three cells. Each cell displays the
remaining candidates or a single digit if given as a clue or solved by you.

```
     1    2    3     4    5    6     7    8    9
  ┍───────────────┬───────────────┬───────────────┐
  │      12·  1·· │           1·· │           ·2· │ 
A │  6   4··  45· │  3    7   ·5· │  9    8   ··· │ A
  │  ·   ···  ··· │       ·   ··· │  ·    ·   ··· │ 
  │               │               │               │
  │ 1·3  1·3  1·· │ 1··  1··      │ 1·3  1··      │ 
B │ ·5·  ···  ·5· │ ·56  ··6   2  │ ··6  ··6   4  │ B
  │ 78·  ·8·  78· │ ··9  ·89   ·  │ 7··  7··   ·  │ 
  │               │               │               │
  │ 123       1·· │ 1··       1·· │ 123  12·      │ 
C │ ···   9   ··· │ ··6   4   ··6 │ ··6  ··6   5  │ C
  │ 78·   ·   78· │ ···   ·   ·8· │ 7··  7··   ·  │ 
  ├───────────────┼───────────────┼───────────────┤
  │ 123  123  1·· │      1·3  1·3 │ 12·  12·  ·2· │ 
D │ 45·  4··  45· │  8   ··6  4·6 │ ·56  ·56  ··6 │ D
  │ ··9  ···  ··9 │  ·   ··9  ··· │ 7··  7·9  7·9 │ 
  │               │               │               │
  │ 123  123  1·· │ 12·       1·3 │ 12·  12·  ·2· │ 
E │ 4··  4··  4·· │ 4·6   5   4·6 │ ··6  ··6  ··6 │ E
  │ ·89  ·8·  ·89 │ 7·9   ·   ··· │ 78·  7·9  789 │ 
  │               │               │               │
  │ 12·           │ 12·  1··  1·· │      12·      │ 
F │ ·5·   7    6  │ ···  ···  ··· │  4   ·5·   3  │ F
  │ ·89   ·    ·  │ ··9  ··9  ··· │  ·   ··9   ·  │ 
  ├───────────────┼───────────────┼───────────────┤
  │ 1··           │ 1··  1··      │ ···       ··· │ 
G │ 4··   5    2  │ 4·6  ··6   9  │ ··6   3   ··6 │ G
  │ 78·   ·    ·  │ ···  ·8·   ·  │ 78·   ·   78· │ 
  │               │               │               │
  │ ···  ···  ··· │ ···  ··3      │ ·2·           │ 
H │ ···  ··6  ··· │ ·56  ··6   7  │ ·56   4    1  │ H
  │ ·89  ·8·  ·89 │ ···  ·8·   ·  │ ·8·   ·    ·  │ 
  │               │               │               │
  │ 1··  1··      │ 1··       1·· │ ···  ···  ··· │ 
J │ 4··  4·6   3  │ 456   2   456 │ ·56  ·56  ··6 │ J
  │ 789  ·8·   ·  │ ···   ·   ·8· │ 78·  7·9  789 │ 
  └───────────────┴───────────────┴───────────────┘
     1    2    3     4    5    6     7    8    9
```

### The Rules of Sudoku

The rules are few and simple:

1. Place the digits `1` through `9` into each cell of the grid.
2. Each row, column, and three-by-three block must contain exactly one of each digit.
3. Every puzzle must have a single unique solution.

### Reading the Board

Each cell will contain either a single digit or a list of candidates.

```
     1    2    3     4
  ┍───────────────┬─────
  │      12·  1·· │      
A │  6   4··  45· │  3  
  │  ·   ···  ··· │      
```

Cell `A1` was given as a clue, so the `6` has a dot below it,
but cell `A4` was solved by the player, so the `3` does not.
Finally, cell `A2` can be solved with either a `1`, `2`, or `4`.

### Printing the Board

The full board with all candidates will be automatically printed
to the screen after every move, but you can print it at any time
using `P` or `P <digit>` to focus on a single candidate.
Here a dot signifies a cell that has the digit as a candidate,
and empty cells have been solved with another digit.

```
    1 2 3   4 5 6   7 8 9
  ┍───────┬───────┬───────┐
A │ •   • │ •   • │ •     │ A
B │     • │ •   • │     • │ B
C │ •   • │ •     │     • │ C
  ├───────┼───────┼───────┤
D │       │       │   2   │ D
E │ •   • │       │       │ E
F │       │   2   │       │ F
  ├───────┼───────┼───────┤
G │       │     • │     • │ G
H │   2   │       │       │ H
J │       │ •   • │ •   • │ J
  └───────┴───────┴───────┘
    1 2 3   4 5 6   7 8 9
```

You can use `P G` to print only the given clues, or `P K` to print
all known (given or solved) cells.

There are several commands to allow sharing a puzzle or playing it
on any site that accepts one of the following formats.

Use `X .` to export the puzzle with a dot for unsolved cells,
the most commonly-supported format.

```
.8......4 3.....5.. .....197. .64....2. .5......9 7...2..1. ...3..6.. 42581.... 9........
```

Use `W` for a format created by Andrew Stuart of [SudokuWiki.org][sudokuwiki].
This will produce a clickable link (control-click in most terminals)
that will open the puzzle there like [this][example].

```
https://www.sudokuwiki.org/sudoku.htm?bd=3681m6n4n8nc0e280h09kim6mkuguk11a0a6340g243kbo03g141ac82210hl2t8t8c805d886118e6ieoeocoaog141g8o8jg05ro8o03b88242c209lglk21pgd60h05118103m048g848g14aea7k7g7kcu9ode
```

Use `M` for a grid suitable for email or other plain-text formats.

```
+-----------------+--------------------+--------------------+
| 1256 8    12679 | 25679 35679 235679 | 123    36   4      |
| 3    1479 12679 | 24679 46789 246789 | 5      68   1268   |
| 256  4    26    | 2456  34568 1      | 9      7    2368   |
+-----------------+--------------------+--------------------+
| 18   6    4     | 1579  35789 35789  | 378    2    3578   |
| 128  5    1238  | 1467  34678 34678  | 3478   3468 9      |
| 7    39   389   | 4569  2     345689 | 348    1    3568   |
+-----------------+--------------------+--------------------+
| 18   17   178   | 3     4579  24579  | 6      4589 12578  |
| 4    2    5     | 8     1     679    | 37     39   37     |
| 9    137  13678 | 24567 4567  24567  | 123478 3458 123578 |
+-----------------+--------------------+--------------------+
```

### Playing the Puzzle

Use `E <cell> <digit>` to remove a single candidate from a cell
and `S <cell> <digit>` to solve a cell. The player will not allow you
to make an invalid move, but if you put yourself in an unwinnable
position, use `Z` to undo your previous moves. Use `R` to reset all
candidates based on the solved cells. The `V` command will tell yuo
if you've reached an impasse.

### Using the Solver

If you get stuck, use `F` to print a list of deductions found by the strategies
known to the solver, and `A` to apply one or all of them.

Lastly, if you wish to waive the white flag and give up, use `B` to solve
the puzzle using Bowman's Bingo, a.k.a. brute force trying all possibilities,
or `Q` to quit the program.


## Puzzle Tools

The application has several commands to help you create, solve and analyze
your own puzzles or those from other sites or collections. You can view
them all with

```bash
./sudoku-rust help
```

which will present this menu:

```
A command-line sudoku player, generator and solver written in Rust

Usage: sudoku-rust [COMMAND]

Commands:
  play     Start the interactive player
  create   Generate a new complete puzzle
  solve    Solve a puzzle or all puzzles from STDIN
  bingo    Brute force a puzzle using Bowman's Bingo
  extract  Extract patterns from puzzles from STDIN
  find     Find a solvable set of clues using patterns from STDIN
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

To see the available options for any command, use this:

```bash
./sudoku-rust help <command>
```

For example, these are the options for the `create` command:

```
Generate a new complete puzzle

Usage: sudoku-rust create [OPTIONS]

Options:
  -r, --randomize            Randomize the cells before generating
  -c, --clues <CLUES>        Stop once a puzzle with the given number of clues is found
  -t, --time <TIME>          Stop after the given number of seconds
  -b, --bar                  Show a progress bar while running
  -s, --solution <SOLUTION>  The completed puzzle to use as a starting point
  -h, --help                 Print help
  -V, --version              Print version
```


## Exploring the Code

The code is organized into several modules, each with a specific purpose.

- `build` - Generate complete puzzles and starting positions
- `commands` - All of the command-line subcommands
- `io` - Puzzle parsers and formatters
- `layout` - The data structures that make up a puzzle board
- `puzzle` - The puzzle board itself and methods for manipulating it
- `solve` - The solver and strategies for solving puzzles

These are some of the more interesting files with the gory details:

- [`layout`](src/layout.rs) - Describes all the board pieces and their relationships
- [`cell_set.rs`](src/layout/cells/cell_set.rs) - The heart of the board and solvers
- [`puzzle`](src/puzzle.rs) - Explains the supporting cast for the puzzle board
- [`board.rs`](src/puzzle/board.rs) - The board itself and its methods
- [`algorithms`](src/solve/algorithms) - Where the real fun happens


[sudokuwiki]: https://www.sudokuwiki.org/
[example]: https://www.sudokuwiki.org/sudoku.htm?bd=3681m6n4n8nc0e280h09kim6mkuguk11a0a6340g243kbo03g141ac82210hl2t8t8c805d886118e6ieoeocoaog141g8o8jg05ro8o03b88242c209lglk21pgd60h05118103m048g848g14aea7k7g7kcu9ode
