# MINECORE

MINECORE is an open-source, extensible puzzle engine and logical solver. Originally developed as the robust foundation for MINE//SHIFT, MINECORE provides a completely decoupled library for creating, managing, solving, and generating grid-based deduction puzzles (like Minesweeper).

## Features
- **Topological Independence:** Abstractions for both Square and Hexagonal grids natively.
- **Logically Bounded Solver:** A safe, iterative logical solver evaluating direct and subset constraints without recursive explosions or brute-forcing hidden state. 
- **No-Guess Verification:** Generates boards that are strictly solvable without guessing from the very first click, with enforceable budget and generation limits.
- **Deterministic State:** Puzzle states are fully serializable and seeded, making Daily Challenges or synchronized multiplayer states trivially reproducible.
- **Game Modes:** Support for dynamic, generic rulesets including Survival, Blitz, and Expedition stages.

## Usage Example

```rust
use minecore::board::{Board, Geometry, GameStatus};
use minecore::solver::Solver;

fn main() {
    // 1. Create a board
    let mut board = Board::new_square(16, 16, 40);

    // 2. Reveal a safe starting cell
    board.is_no_guess = true; // Automatically invokes the No Guess generator on the first click
    board.reveal(0, 0);

    // 3. Run bounded logical analysis
    let deductions = Solver::deduce(&board);
    println!("Solver found {} logical deductions.", deductions.len());
}
```

## Architecture

- `board.rs`: Provides the unified `Board` struct holding a linear array of `Cell` objects. `neighbors()` implements the specific geometry math mapping coordinates.
- `solver.rs`: The rules-engine. Consumes visible `CellState::Revealed` information and yields `Deduction::Safe` or `Deduction::Mine`.
- `no_guess.rs`: The generation loop. Reseeds and mutates puzzles until the `Solver` can conclusively prove a 100% win rate from the designated starting coordinate.
- `mode.rs`: Constants and structures defining rulesets across the platform.

## Licensing
This engine is provided under the MIT OR Apache-2.0 license (pending repository owner final approval).
