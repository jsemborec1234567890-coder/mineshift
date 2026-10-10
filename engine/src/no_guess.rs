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
