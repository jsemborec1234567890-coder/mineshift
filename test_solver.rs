use mineshift::board::{Board, CellState};
use mineshift::solver::{Solver, Deduction};
use std::process::{Command, Stdio};
use std::io::Write;

fn main() {
    let mut board = Board::new_square(4, 4, 1);
    board.is_no_guess = true;
    board.reveal(0, 0); // This should run the no-guess logic
    
    // Convert board to JSON manually for testing
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
        .expect("Failed to spawn");
        
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(json.as_bytes()).unwrap();
    }
    
    let output = child.wait_with_output().unwrap();
    println!("Python deductions: {}", String::from_utf8_lossy(&output.stdout));
    println!("Rust deductions: {:?}", Solver::deduce(&board));
}
