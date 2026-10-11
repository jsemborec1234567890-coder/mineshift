use crate::board::{Board, CellState};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Deduction {
    Safe(usize, usize),
    Mine(usize, usize),
}

pub struct Solver;

impl Solver {
    pub fn deduce(board: &Board) -> Vec<Deduction> {
        let mut deductions = Vec::new();
        let mut constraints: Vec<(HashSet<(usize, usize)>, usize)> = Vec::new();

        // 1. Build initial constraints from revealed cells
        for y in 0..board.height {
            for x in 0..board.width {
                let cell = match board.get_cell(x, y) {
                    Some(c) => c,
                    None => continue,
                };
                if cell.state == CellState::Revealed && !cell.is_mine {
                    let mut hidden_neighbors = HashSet::new();
                    let mut flagged_count = 0;

                    for (nx, ny) in board.neighbors(x, y) {
                        if let Some(n_cell) = board.get_cell(nx, ny) {
                            match n_cell.state {
                                CellState::Covered => {
                                    hidden_neighbors.insert((nx, ny));
                                }
                                CellState::Flagged => {
                                    flagged_count += 1;
                                }
                                _ => {}
                            }
                        }
                    }

                    if !hidden_neighbors.is_empty() {
                        let remaining_mines =
                            (cell.adjacent_mines as i32 - flagged_count as i32).max(0) as usize;
                        constraints.push((hidden_neighbors, remaining_mines));
                    }
                }
            }
        }

        // 2. Direct deductions (Rule 1)
        for (cells, mines) in &constraints {
            if *mines == 0 {
                for &(cx, cy) in cells {
                    deductions.push(Deduction::Safe(cx, cy));
                }
            } else if cells.len() == *mines {
                for &(cx, cy) in cells {
                    deductions.push(Deduction::Mine(cx, cy));
                }
            }
        }

        if !deductions.is_empty() {
            let mut unique = HashSet::new();
            deductions.retain(|d| unique.insert(d.clone()));
            return deductions;
        }

        // 3. Subset deductions (Rule 2)
        for (i, (cells_a, mines_a)) in constraints.iter().enumerate() {
            for (j, (cells_b, mines_b)) in constraints.iter().enumerate() {
                if i != j && cells_a.is_subset(cells_b) && cells_a.len() < cells_b.len() {
                    let diff: HashSet<(usize, usize)> =
                        cells_b.difference(cells_a).copied().collect();
                    let diff_mines = mines_b.saturating_sub(*mines_a);

                    if diff_mines == 0 {
                        for &(cx, cy) in &diff {
                            deductions.push(Deduction::Safe(cx, cy));
                        }
                    } else if diff.len() == diff_mines {
                        for &(cx, cy) in &diff {
                            deductions.push(Deduction::Mine(cx, cy));
                        }
                    }
                }
            }
        }

        let mut unique = HashSet::new();
        deductions.retain(|d| unique.insert(d.clone()));
        deductions
    }
}
