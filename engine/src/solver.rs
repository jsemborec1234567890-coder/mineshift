use crate::board::{Board, CellState, ClueType};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Deduction {
    Safe(usize, usize),
    Mine(usize, usize),
}

impl Deduction {
    pub fn coords(&self) -> (usize, usize) {
        match *self {
            Deduction::Safe(x, y) => (x, y),
            Deduction::Mine(x, y) => (x, y),
        }
    }

    pub fn is_safe(&self) -> bool {
        matches!(self, Deduction::Safe(..))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeductionType {
    DirectSafe,
    DirectMine,
    SubsetSafe,
    SubsetMine,
    Ring2Constraint,
    OverlappingConstraint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Explanation {
    pub conclusion: Deduction,
    pub deduction_type: DeductionType,
    pub applicable_clues: Vec<(usize, usize)>,
    pub relevant_unknowns: Vec<(usize, usize)>,
    pub explanation_text: String,
}

#[derive(Clone, Debug)]
struct Constraint {
    origin: (usize, usize),
    is_ring2: bool,
    cells: HashSet<(usize, usize)>,
    mines: usize,
}

pub struct Solver;

impl Solver {
    const MAX_WORK_BUDGET: usize = 2000;

    pub fn deduce(board: &Board) -> Vec<Deduction> {
        let mut deductions = Vec::new();
        let constraints = Self::build_constraints(board);

        // 1. Direct deductions
        for c in &constraints {
            if c.mines == 0 {
                for &(cx, cy) in &c.cells {
                    deductions.push(Deduction::Safe(cx, cy));
                }
            } else if c.cells.len() == c.mines {
                for &(cx, cy) in &c.cells {
                    deductions.push(Deduction::Mine(cx, cy));
                }
            }
        }

        if !deductions.is_empty() {
            let mut unique = HashSet::new();
            deductions.retain(|d| unique.insert(d.clone()));
            return deductions;
        }

        // 2. Subset deductions
        let mut work_count = 0;
        for (i, c_a) in constraints.iter().enumerate() {
            for (j, c_b) in constraints.iter().enumerate() {
                work_count += 1;
                if work_count > Self::MAX_WORK_BUDGET {
                    break;
                }
                if i != j && c_a.cells.is_subset(&c_b.cells) && c_a.cells.len() < c_b.cells.len() {
                    let diff: HashSet<(usize, usize)> =
                        c_b.cells.difference(&c_a.cells).copied().collect();
                    let diff_mines = c_b.mines.saturating_sub(c_a.mines);

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
            if work_count > Self::MAX_WORK_BUDGET {
                break;
            }
        }

        if !deductions.is_empty() {
            let mut unique = HashSet::new();
            deductions.retain(|d| unique.insert(d.clone()));
            return deductions;
        }

        // 3. Overlapping constraint reduction
        for (i, c_a) in constraints.iter().enumerate() {
            for (j, c_b) in constraints.iter().enumerate() {
                if i >= j {
                    continue;
                }
                work_count += 1;
                if work_count > Self::MAX_WORK_BUDGET {
                    break;
                }

                let intersection: HashSet<(usize, usize)> =
                    c_a.cells.intersection(&c_b.cells).copied().collect();
                if intersection.is_empty() {
                    continue;
                }

                let only_a: HashSet<(usize, usize)> =
                    c_a.cells.difference(&c_b.cells).copied().collect();
                let only_b: HashSet<(usize, usize)> =
                    c_b.cells.difference(&c_a.cells).copied().collect();

                // If max mines in intersection forced by A means B's remaining must be mines
                let max_mines_in_inter = intersection.len().min(c_a.mines).min(c_b.mines);
                if c_b.mines > max_mines_in_inter && only_b.len() == c_b.mines - max_mines_in_inter
                {
                    for &(cx, cy) in &only_b {
                        deductions.push(Deduction::Mine(cx, cy));
                    }
                }

                // If min mines in intersection forced by A leaves 0 for B
                let min_mines_in_inter_a = c_a.mines.saturating_sub(only_a.len());
                if min_mines_in_inter_a == c_b.mines && !only_b.is_empty() {
                    for &(cx, cy) in &only_b {
                        deductions.push(Deduction::Safe(cx, cy));
                    }
                }
            }
        }

        let mut unique = HashSet::new();
        deductions.retain(|d| unique.insert(d.clone()));
        deductions
    }

    pub fn explain_step(board: &Board) -> Option<Explanation> {
        let constraints = Self::build_constraints(board);

        // 1. Direct Safe
        for c in &constraints {
            if c.mines == 0 && !c.cells.is_empty() {
                let first = *c.cells.iter().next().unwrap();
                let text = if c.is_ring2 {
                    format!(
                        "Ring-2 clue at ({}, {}) has all required distance-2 mines flagged. Cell ({}, {}) is safe.",
                        c.origin.0, c.origin.1, first.0, first.1
                    )
                } else {
                    format!(
                        "Clue at ({}, {}) has all adjacent mines flagged. Cell ({}, {}) is safe.",
                        c.origin.0, c.origin.1, first.0, first.1
                    )
                };
                return Some(Explanation {
                    conclusion: Deduction::Safe(first.0, first.1),
                    deduction_type: if c.is_ring2 {
                        DeductionType::Ring2Constraint
                    } else {
                        DeductionType::DirectSafe
                    },
                    applicable_clues: vec![c.origin],
                    relevant_unknowns: c.cells.iter().copied().collect(),
                    explanation_text: text,
                });
            }
        }

        // 2. Direct Mine
        for c in &constraints {
            if c.cells.len() == c.mines && !c.cells.is_empty() {
                let first = *c.cells.iter().next().unwrap();
                let text = if c.is_ring2 {
                    format!(
                        "Ring-2 clue at ({}, {}) requires {} distance-2 mines with exactly {} unrevealed cells remaining. Cell ({}, {}) is a mine.",
                        c.origin.0,
                        c.origin.1,
                        c.mines,
                        c.cells.len(),
                        first.0,
                        first.1
                    )
                } else {
                    format!(
                        "Clue at ({}, {}) requires {} mines with exactly {} unrevealed neighbors remaining. Cell ({}, {}) is a mine.",
                        c.origin.0,
                        c.origin.1,
                        c.mines,
                        c.cells.len(),
                        first.0,
                        first.1
                    )
                };
                return Some(Explanation {
                    conclusion: Deduction::Mine(first.0, first.1),
                    deduction_type: if c.is_ring2 {
                        DeductionType::Ring2Constraint
                    } else {
                        DeductionType::DirectMine
                    },
                    applicable_clues: vec![c.origin],
                    relevant_unknowns: c.cells.iter().copied().collect(),
                    explanation_text: text,
                });
            }
        }

        // 3. Subset Reductions
        for (i, c_a) in constraints.iter().enumerate() {
            for (j, c_b) in constraints.iter().enumerate() {
                if i != j && c_a.cells.is_subset(&c_b.cells) && c_a.cells.len() < c_b.cells.len() {
                    let diff: HashSet<(usize, usize)> =
                        c_b.cells.difference(&c_a.cells).copied().collect();
                    let diff_mines = c_b.mines.saturating_sub(c_a.mines);

                    if diff_mines == 0 && !diff.is_empty() {
                        let first = *diff.iter().next().unwrap();
                        return Some(Explanation {
                            conclusion: Deduction::Safe(first.0, first.1),
                            deduction_type: DeductionType::SubsetSafe,
                            applicable_clues: vec![c_a.origin, c_b.origin],
                            relevant_unknowns: diff.into_iter().collect(),
                            explanation_text: format!(
                                "Subset deduction: all mines for ({}, {}) are contained within ({}, {}). The outer cell ({}, {}) must be safe.",
                                c_a.origin.0,
                                c_a.origin.1,
                                c_b.origin.0,
                                c_b.origin.1,
                                first.0,
                                first.1
                            ),
                        });
                    } else if diff.len() == diff_mines && !diff.is_empty() {
                        let first = *diff.iter().next().unwrap();
                        return Some(Explanation {
                            conclusion: Deduction::Mine(first.0, first.1),
                            deduction_type: DeductionType::SubsetMine,
                            applicable_clues: vec![c_a.origin, c_b.origin],
                            relevant_unknowns: diff.into_iter().collect(),
                            explanation_text: format!(
                                "Subset deduction: clue at ({}, {}) requires additional mines outside ({}, {}). Cell ({}, {}) must be a mine.",
                                c_b.origin.0,
                                c_b.origin.1,
                                c_a.origin.0,
                                c_a.origin.1,
                                first.0,
                                first.1
                            ),
                        });
                    }
                }
            }
        }

        None
    }

    fn build_constraints(board: &Board) -> Vec<Constraint> {
        let mut constraints = Vec::new();

        for y in 0..board.height {
            for x in 0..board.width {
                let cell = match board.get_cell(x, y) {
                    Some(c) => c,
                    None => continue,
                };
                if cell.state == CellState::Revealed && !cell.is_mine {
                    // Standard adjacent constraint
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
                        let remaining =
                            (cell.adjacent_mines as i32 - flagged_count as i32).max(0) as usize;
                        constraints.push(Constraint {
                            origin: (x, y),
                            is_ring2: false,
                            cells: hidden_neighbors,
                            mines: remaining,
                        });
                    }

                    // Extended Ring-2 constraint
                    if cell.clue_type == ClueType::Ring2 {
                        let mut hidden_r2 = HashSet::new();
                        let mut flagged_r2 = 0;

                        for (r2x, r2y) in board.ring2_neighbors(x, y) {
                            if let Some(r2_cell) = board.get_cell(r2x, r2y) {
                                match r2_cell.state {
                                    CellState::Covered => {
                                        hidden_r2.insert((r2x, r2y));
                                    }
                                    CellState::Flagged => {
                                        flagged_r2 += 1;
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if !hidden_r2.is_empty() {
                            let remaining_r2 =
                                (cell.ring2_mines as i32 - flagged_r2 as i32).max(0) as usize;
                            constraints.push(Constraint {
                                origin: (x, y),
                                is_ring2: true,
                                cells: hidden_r2,
                                mines: remaining_r2,
                            });
                        }
                    }
                }
            }
        }

        constraints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_safe_and_mine_deductions() {
        let mut board = Board::new_square(3, 3, 0);
        board.first_click = false;

        let idx_center = board.get_index(1, 1);
        let idx_corner = board.get_index(0, 0);

        board.cells[idx_center].state = CellState::Revealed;
        board.cells[idx_center].adjacent_mines = 1;
        board.cells[idx_corner].state = CellState::Flagged;

        // Remaining 7 neighbors should be deduced as Safe
        let deductions = Solver::deduce(&board);
        assert!(!deductions.is_empty());
        assert!(deductions.iter().all(|d| d.is_safe()));
        assert!(deductions.contains(&Deduction::Safe(0, 1)));
        assert!(deductions.contains(&Deduction::Safe(2, 2)));

        let explanation = Solver::explain_step(&board).unwrap();
        assert_eq!(explanation.deduction_type, DeductionType::DirectSafe);
        assert!(explanation.explanation_text.contains("Cell"));
    }

    #[test]
    fn test_subset_deduction_classic_1_2_1() {
        let mut board = Board::new_square(3, 2, 0);
        board.first_click = false;

        let idx_00 = board.get_index(0, 0);
        let idx_10 = board.get_index(1, 0);
        let idx_20 = board.get_index(2, 0);

        board.cells[idx_00].state = CellState::Revealed;
        board.cells[idx_00].adjacent_mines = 1;

        board.cells[idx_10].state = CellState::Revealed;
        board.cells[idx_10].adjacent_mines = 2;

        board.cells[idx_20].state = CellState::Revealed;
        board.cells[idx_20].adjacent_mines = 1;

        let deductions = Solver::deduce(&board);
        assert!(deductions.contains(&Deduction::Mine(2, 1)));
        assert!(deductions.contains(&Deduction::Mine(0, 1)));

        let explanation = Solver::explain_step(&board).unwrap();
        assert_eq!(explanation.deduction_type, DeductionType::SubsetMine);
    }

    #[test]
    fn test_ring2_clue_deduction() {
        let mut board = Board::new_square(5, 5, 0);
        board.first_click = false;

        let idx_22 = board.get_index(2, 2);
        board.cells[idx_22].state = CellState::Revealed;
        board.cells[idx_22].clue_type = ClueType::Ring2;
        board.cells[idx_22].ring2_mines = 0;

        let deductions = Solver::deduce(&board);
        assert!(deductions.contains(&Deduction::Safe(0, 0)));
        assert!(deductions.contains(&Deduction::Safe(4, 4)));
    }
}
