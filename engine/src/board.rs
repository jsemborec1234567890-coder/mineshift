use rand::Rng;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellState {
    Covered,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Geometry {
    Square,
    Hex,
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub active: bool,
    pub is_mine: bool,
    pub state: CellState,
    pub adjacent_mines: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            active: true,
            is_mine: false,
            state: CellState::Covered,
            adjacent_mines: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameStatus {
    Playing,
    Won,
    Lost,
}

#[derive(Clone)]
pub struct Board {
    pub geometry: Geometry,
    pub radius: usize,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub mine_count: usize,
    pub status: GameStatus,
    pub first_click: bool,
    pub is_no_guess: bool,
    pub seed: Option<u64>,
    pub flags_placed: usize,
    pub start_time: Option<std::time::Instant>,
    pub end_time: Option<std::time::Instant>,
}

impl Board {
    pub fn new_square(width: usize, height: usize, mine_count: usize) -> Self {
        Self {
            geometry: Geometry::Square,
            radius: 0,
            width,
            height,
            cells: vec![Cell::default(); width * height],
            mine_count,
            status: GameStatus::Playing,
            first_click: true,
            is_no_guess: false,
            seed: None,
            flags_placed: 0,
            start_time: None,
            end_time: None,
        }
    }

    pub fn new_hex(radius: usize, mine_count: usize) -> Self {
        let width = radius * 2 + 1;
        let height = radius * 2 + 1;
        let mut cells = vec![Cell::default(); width * height];

        let r_isize = radius as isize;
        for y in 0..height {
            for x in 0..width {
                let q = x as isize - r_isize;
                let r = y as isize - r_isize;
                let active = q.abs() <= r_isize && r.abs() <= r_isize && (q + r).abs() <= r_isize;
                cells[y * width + x].active = active;
            }
        }

        Self {
            geometry: Geometry::Hex,
            radius,
            width,
            height,
            cells,
            mine_count,
            status: GameStatus::Playing,
            first_click: true,
            is_no_guess: false,
            seed: None,
            flags_placed: 0,
            start_time: None,
            end_time: None,
        }
    }

    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height {
            let cell = &self.cells[self.get_index(x, y)];
            if cell.active {
                return Some(cell);
            }
        }
        None
    }

    pub fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let x = x as isize;
        let y = y as isize;
        let w = self.width as isize;
        let h = self.height as isize;

        let deltas = match self.geometry {
            Geometry::Square => vec![
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ],
            Geometry::Hex => vec![(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)],
        };

        deltas
            .into_iter()
            .filter_map(move |(dx, dy)| {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= 0 && nx < w && ny >= 0 && ny < h {
                    let cell = &self.cells[(ny * w + nx) as usize];
                    if cell.active {
                        return Some((nx as usize, ny as usize));
                    }
                }
                None
            })
            .collect()
    }

    fn place_mines(&mut self, safe_x: usize, safe_y: usize) {
        use rand::{Rng, SeedableRng};
        let mut mines_placed = 0;

        let safe_zone: Vec<(usize, usize)> = self
            .neighbors(safe_x, safe_y)
            .into_iter()
            .chain(std::iter::once((safe_x, safe_y)))
            .collect();

        if let Some(s) = self.seed {
            let mut rng = rand::rngs::StdRng::seed_from_u64(s);
            while mines_placed < self.mine_count {
                let x = rng.gen_range(0..self.width);
                let y = rng.gen_range(0..self.height);

                let idx = self.get_index(x, y);
                if !self.cells[idx].active || safe_zone.contains(&(x, y)) {
                    continue;
                }

                if !self.cells[idx].is_mine {
                    self.cells[idx].is_mine = true;
                    mines_placed += 1;
                }
            }
        } else {
            let mut rng = rand::thread_rng();
            while mines_placed < self.mine_count {
                let x = rng.gen_range(0..self.width);
                let y = rng.gen_range(0..self.height);

                let idx = self.get_index(x, y);
                if !self.cells[idx].active || safe_zone.contains(&(x, y)) {
                    continue;
                }

                if !self.cells[idx].is_mine {
                    self.cells[idx].is_mine = true;
                    mines_placed += 1;
                }
            }
        }

        // Compute adjacent mines
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                if self.cells[idx].is_mine {
                    continue;
                }

                let mut count = 0;
                for (nx, ny) in self.neighbors(x, y) {
                    let n_idx = self.get_index(nx, ny);
                    if self.cells[n_idx].is_mine {
                        count += 1;
                    }
                }
                self.cells[idx].adjacent_mines = count;
            }
        }
    }

    pub fn reveal(&mut self, x: usize, y: usize) {
        if !matches!(self.status, GameStatus::Playing) {
            return;
        }

        if self.first_click {
            self.start_time = Some(std::time::Instant::now());
            self.place_mines(x, y);
            self.first_click = false;
        }

        let cell = match self.get_cell(x, y) {
            Some(c) => c,
            None => return,
        };

        if cell.state != CellState::Covered {
            return;
        }

        let idx = self.get_index(x, y);
        self.cells[idx].state = CellState::Revealed;

        if self.cells[idx].is_mine {
            self.status = GameStatus::Lost;
            self.end_time = Some(std::time::Instant::now());
            self.reveal_all_mines();
            return;
        }

        if self.cells[idx].adjacent_mines == 0 {
            self.flood_reveal(x, y);
        }

        self.check_victory();
    }

    pub fn chord(&mut self, x: usize, y: usize) {
        if !matches!(self.status, GameStatus::Playing) {
            return;
        }

        let cell = match self.get_cell(x, y) {
            Some(c) => c,
            None => return,
        };

        if cell.state != CellState::Revealed || cell.adjacent_mines == 0 {
            return;
        }

        let mut flag_count = 0;
        for (nx, ny) in self.neighbors(x, y) {
            let n_idx = self.get_index(nx, ny);
            if self.cells[n_idx].state == CellState::Flagged {
                flag_count += 1;
            }
        }

        if flag_count == cell.adjacent_mines {
            for (nx, ny) in self.neighbors(x, y) {
                let n_idx = self.get_index(nx, ny);
                if self.cells[n_idx].state == CellState::Covered {
                    self.reveal(nx, ny);
                }
            }
        }
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        if !matches!(self.status, GameStatus::Playing) {
            return;
        }

        let idx = self.get_index(x, y);
        match self.cells[idx].state {
            CellState::Covered => {
                self.cells[idx].state = CellState::Flagged;
                self.flags_placed += 1;
            }
            CellState::Flagged => {
                self.cells[idx].state = CellState::Covered;
                self.flags_placed -= 1;
            }
            _ => {}
        }
    }

    fn flood_reveal(&mut self, start_x: usize, start_y: usize) {
        let mut queue = VecDeque::new();
        queue.push_back((start_x, start_y));

        while let Some((x, y)) = queue.pop_front() {
            for (nx, ny) in self.neighbors(x, y) {
                let n_idx = self.get_index(nx, ny);
                if self.cells[n_idx].state == CellState::Covered && !self.cells[n_idx].is_mine {
                    self.cells[n_idx].state = CellState::Revealed;
                    if self.cells[n_idx].adjacent_mines == 0 {
                        queue.push_back((nx, ny));
                    }
                }
            }
        }
    }

    fn reveal_all_mines(&mut self) {
        for cell in &mut self.cells {
            if cell.is_mine && cell.state == CellState::Covered {
                cell.state = CellState::Revealed;
            }
        }
    }

    fn check_victory(&mut self) {
        let mut unrevealed_safes = 0;
        for cell in &self.cells {
            if cell.active && !cell.is_mine && cell.state != CellState::Revealed {
                unrevealed_safes += 1;
            }
        }

        if unrevealed_safes == 0 {
            self.status = GameStatus::Won;
            self.end_time = Some(std::time::Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_initialization() {
        let board = Board::new_square(9, 9, 10);
        assert_eq!(board.width, 9);
        assert_eq!(board.height, 9);
        assert_eq!(board.mine_count, 10);
        assert!(matches!(board.status, GameStatus::Playing));
        assert!(board.first_click);
    }

    #[test]
    fn test_first_click_safety() {
        let mut board = Board::new_square(9, 9, 10);
        board.reveal(0, 0);
        assert!(!board.first_click);
        let cell = board.get_cell(0, 0).unwrap();
        assert!(!cell.is_mine);
        assert_eq!(cell.state, CellState::Revealed);
        assert!(matches!(board.status, GameStatus::Playing));

        let mut mine_count = 0;
        for c in &board.cells {
            if c.is_mine {
                mine_count += 1;
            }
        }
        assert_eq!(mine_count, 10);
    }

    #[test]
    fn test_flagging() {
        let mut board = Board::new_square(9, 9, 10);
        board.toggle_flag(1, 1);
        let cell = board.get_cell(1, 1).unwrap();
        assert_eq!(cell.state, CellState::Flagged);
        assert_eq!(board.flags_placed, 1);

        board.toggle_flag(1, 1);
        let cell = board.get_cell(1, 1).unwrap();
        assert_eq!(cell.state, CellState::Covered);
        assert_eq!(board.flags_placed, 0);
    }
}

#[test]
fn test_chording() {
    let mut board = Board::new_square(9, 9, 0); // No random mines
    board.first_click = false;

    // Manually place a mine and numbers
    let idx1 = board.get_index(1, 1);
    board.cells[idx1].is_mine = true;
    let idx2 = board.get_index(0, 1);
    board.cells[idx2].adjacent_mines = 1;

    // Reveal 0,1
    board.reveal(0, 1);
    assert_eq!(
        board.cells[board.get_index(0, 1)].state,
        CellState::Revealed
    );

    // Try chording before flagging (should do nothing)
    board.chord(0, 1);
    assert_eq!(board.cells[board.get_index(0, 0)].state, CellState::Covered);

    // Flag the mine
    board.toggle_flag(1, 1);

    // Chord again (should reveal surrounding safe cells)
    board.chord(0, 1);
    assert_eq!(
        board.cells[board.get_index(0, 0)].state,
        CellState::Revealed
    );
    assert_eq!(
        board.cells[board.get_index(0, 2)].state,
        CellState::Revealed
    );
}

#[test]
fn test_hex_initialization() {
    let board = Board::new_hex(5, 10);
    assert_eq!(board.geometry, Geometry::Hex);
    assert_eq!(board.radius, 5);
    assert_eq!(board.mine_count, 10);

    // Hex grid with radius R has 3*R*(R+1) + 1 cells.
    // For R=5, that is 3*5*6 + 1 = 91 active cells.
    let active_cells = board.cells.iter().filter(|c| c.active).count();
    assert_eq!(active_cells, 91);
}
