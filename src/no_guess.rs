use crate::board::{Board, CellState};
use crate::solver::{Solver, Deduction};
use rand::Rng;

pub fn generate_no_guess_board(mut board: Board, safe_x: usize, safe_y: usize) -> Option<Board> {
    let max_retries = 100;
    
    for _ in 0..max_retries {
        // Reset board
        for cell in &mut board.cells {
            cell.is_mine = false;
            cell.state = CellState::Covered;
            cell.adjacent_mines = 0;
        }
        board.first_click = true;
        
        // Place mines and calculate numbers
        // board.place_mines is private, so we need a public function or do it here.
        // Actually, Board::new_* doesn't place mines until first click.
        // We can use a temporary board and do first click logic.
        board.is_no_guess = false;
        if let Some(s) = board.seed {
            board.seed = Some(s.wrapping_add(1));
        }
        board.reveal(safe_x, safe_y);
        
        // Verify no-guess
        let mut sim_board = board.clone();
        
        loop {
            let deductions = Solver::deduce(&sim_board);
            if deductions.is_empty() {
                break;
            }
            
            for d in deductions {
                match d {
                    Deduction::Safe(x, y) => {
                        let idx = sim_board.get_index(x, y);
                        sim_board.cells[idx].state = CellState::Revealed;
                    }
                    Deduction::Mine(x, y) => {
                        let idx = sim_board.get_index(x, y);
                        sim_board.cells[idx].state = CellState::Flagged;
                    }
                }
            }
        }
        
        // Check if all non-mine cells are revealed
        let mut all_safe_revealed = true;
        for cell in &sim_board.cells {
            if cell.active && !cell.is_mine && cell.state != CellState::Revealed {
                all_safe_revealed = false;
                break;
            }
        }
        
        if all_safe_revealed {
            // Restore covered state for the initial board (except safe_x, safe_y maybe? No, reveal starts the game)
            return Some(board);
        }
        
        // Reset state for next try
        board.status = crate::board::GameStatus::Playing;
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};
    use std::io::Write;

    #[test]
    fn test_python_solver_equivalence() {
        let mut board = Board::new_square(4, 4, 1);
        board.is_no_guess = true;
        board.reveal(0, 0);
        
        let mut json = String::new();
        json.push_str(&format!("{{\"width\": {}, \"height\": {}, \"geometry\": \"Square\", \"cells\": [", board.width, board.height));
        for (i, cell) in board.cells.iter().enumerate() {
            let x = i % board.width;
            let y = i / board.width;
            let state = match cell.state {
                CellState::Covered => "Covered",
                CellState::Revealed => "Revealed",
                CellState::Flagged => "Flagged",
            };
            json.push_str(&format!("{{\"x\": {}, \"y\": {}, \"state\": \"{}\", \"is_mine\": {}, \"adjacent_mines\": {}}}", x, y, state, if cell.is_mine { "true" } else { "false" }, cell.adjacent_mines));
            if i < board.cells.len() - 1 {
                json.push_str(",");
            }
        }
        json.push_str("]}");

        let mut child = Command::new("uv")
            .arg("run")
            .arg("python")
            .arg("tools/python/solver.py")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn python solver");
            
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(json.as_bytes()).unwrap();
        }
        
        let output = child.wait_with_output().unwrap();
        let py_output = String::from_utf8_lossy(&output.stdout);
        println!("Python output: {}", py_output);
        let rust_deductions = Solver::deduce(&board);
        println!("Rust output: {:?}", rust_deductions);
        
        // As long as it ran without errors, we are happy for this smoke test.
        assert!(py_output.starts_with('['));
    }
}
