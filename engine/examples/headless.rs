use minecore::board::{Board, CellState};
use minecore::solver::Solver;

fn main() {
    println!("MINECORE Headless Example");
    
    // 1. Create a board
    let mut board = Board::new_square(16, 16, 40);
    
    // 2. Play a move
    board.is_no_guess = true;
    println!("Generating No Guess board from first click at (0, 0)...");
    
    // In headless, we simulate the logic of the main app for first click:
    if let Some(mut new_b) = minecore::no_guess::generate_no_guess_board(board.clone(), 0, 0) {
        new_b.is_no_guess = true;
        new_b.first_click = false;
        board = new_b;
        println!("Board successfully generated and guaranteed strictly logically solvable.");
    } else {
        println!("Failed to generate within 100 tries, falling back to random.");
        board.reveal(0, 0);
    }
    
    // 3. Run solver
    let deductions = Solver::deduce(&board);
    println!("Initial logical deductions available: {}", deductions.len());
    for d in deductions.iter().take(5) {
        println!(" - {:?}", d);
    }
    
    if deductions.len() > 5 {
        println!(" - ...and {} more", deductions.len() - 5);
    }
}
