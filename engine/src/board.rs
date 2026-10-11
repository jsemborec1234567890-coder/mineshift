use crate::rules::{CluePolicy, FirstClickPolicy, FlagPolicy, Ruleset};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CellState {
    Covered,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Geometry {
    Square,
    Hex,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ClueType {
    StandardAdjacent,
    Ring2,
}

impl Default for ClueType {
    fn default() -> Self {
        Self::StandardAdjacent
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub active: bool,
    pub is_mine: bool,
    pub state: CellState,
    pub adjacent_mines: u8,
    pub clue_type: ClueType,
    pub ring2_mines: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            active: true,
            is_mine: false,
            state: CellState::Covered,
            adjacent_mines: 0,
            clue_type: ClueType::StandardAdjacent,
            ring2_mines: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum GameStatus {
    Playing,
    Won,
    Lost,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub ruleset: Ruleset,
    #[serde(skip)]
    pub start_time: Option<std::time::Instant>,
    #[serde(skip)]
    pub end_time: Option<std::time::Instant>,
}

impl Board {
    pub fn new_square(width: usize, height: usize, mine_count: usize) -> Self {
        let ruleset = Ruleset::classic_square(width, height, mine_count);
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
            ruleset,
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

        let ruleset = Ruleset::classic_hex(radius, mine_count);
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
            ruleset,
            start_time: None,
            end_time: None,
        }
    }

    pub fn from_ruleset(ruleset: Ruleset) -> Self {
        let mut board = match ruleset.topology {
            Geometry::Square => Self::new_square(ruleset.width, ruleset.height, ruleset.mine_count),
            Geometry::Hex => Self::new_hex(ruleset.radius, ruleset.mine_count),
        };
        board.is_no_guess =
            ruleset.logic_guarantee == crate::rules::LogicGuarantee::CertifiedNoGuess;
        board.ruleset = ruleset;
        board
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

    pub fn ring2_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let d1 = self.neighbors(x, y);
        let mut d1_set: HashSet<(usize, usize)> = d1.iter().copied().collect();
        d1_set.insert((x, y));

        let mut ring2 = HashSet::new();
        for &(nx, ny) in &d1 {
            for (nnx, nny) in self.neighbors(nx, ny) {
                if !d1_set.contains(&(nnx, nny)) {
                    ring2.insert((nnx, nny));
                }
            }
        }
        let mut result: Vec<(usize, usize)> = ring2.into_iter().collect();
        result.sort_unstable();
        result
    }

    pub fn can_flag(&self) -> bool {
        match self.ruleset.flag_policy {
            FlagPolicy::NoFlags => false,
            FlagPolicy::LimitedFlags { max_flags } => self.flags_placed < max_flags,
            FlagPolicy::Standard => true,
        }
    }

    pub fn remaining_flags(&self) -> Option<usize> {
        match self.ruleset.flag_policy {
            FlagPolicy::NoFlags => Some(0),
            FlagPolicy::LimitedFlags { max_flags } => {
                Some(max_flags.saturating_sub(self.flags_placed))
            }
            FlagPolicy::Standard => None,
        }
    }

    fn place_mines(&mut self, safe_x: usize, safe_y: usize) {
        use rand::{Rng, SeedableRng};
        let mut mines_placed = 0;

        let safe_zone: Vec<(usize, usize)> = match self.ruleset.first_click_policy {
            FirstClickPolicy::StandardSafe => vec![(safe_x, safe_y)],
            FirstClickPolicy::GuaranteedZeroOpening
            | FirstClickPolicy::ValidatedDeductionOpening => self
                .neighbors(safe_x, safe_y)
                .into_iter()
                .chain(std::iter::once((safe_x, safe_y)))
                .collect(),
        };

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

        // Compute adjacent mines and ring-2 mines
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                if !self.cells[idx].active || self.cells[idx].is_mine {
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

                let mut r2_count = 0;
                for (r2x, r2y) in self.ring2_neighbors(x, y) {
                    let n_idx = self.get_index(r2x, r2y);
                    if self.cells[n_idx].is_mine {
                        r2_count += 1;
                    }
                }
                self.cells[idx].ring2_mines = r2_count;

                // Configure Ring-2 Clue cells if extended clue policy is enabled
                if self.ruleset.clue_policy == CluePolicy::ExtendedRing2 {
                    // Mark roughly 25% of positive clue cells deterministically as Ring2
                    if count > 0 && (x * 7 + y * 13) % 4 == 0 {
                        self.cells[idx].clue_type = ClueType::Ring2;
                    }
                }
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

        // Flag-dependent chording is disabled under NoFlags policy
        if self.ruleset.flag_policy == FlagPolicy::NoFlags {
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
                if !self.can_flag() {
                    return;
                }
                self.cells[idx].state = CellState::Flagged;
                self.flags_placed += 1;
            }
            CellState::Flagged => {
                self.cells[idx].state = CellState::Covered;
                self.flags_placed = self.flags_placed.saturating_sub(1);
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

    #[test]
    fn test_no_flags_policy() {
        let mut ruleset = Ruleset::classic_square(9, 9, 10);
        ruleset.flag_policy = FlagPolicy::NoFlags;
        let mut board = Board::from_ruleset(ruleset);

        assert!(!board.can_flag());
        assert_eq!(board.remaining_flags(), Some(0));

        board.toggle_flag(1, 1);
        assert_eq!(board.cells[board.get_index(1, 1)].state, CellState::Covered);
        assert_eq!(board.flags_placed, 0);
    }

    #[test]
    fn test_limited_flags_policy() {
        let mut ruleset = Ruleset::classic_square(9, 9, 10);
        ruleset.flag_policy = FlagPolicy::LimitedFlags { max_flags: 2 };
        let mut board = Board::from_ruleset(ruleset);

        assert!(board.can_flag());
        assert_eq!(board.remaining_flags(), Some(2));

        board.toggle_flag(0, 0);
        assert_eq!(board.flags_placed, 1);
        assert_eq!(board.remaining_flags(), Some(1));

        board.toggle_flag(0, 1);
        assert_eq!(board.flags_placed, 2);
        assert_eq!(board.remaining_flags(), Some(0));
        assert!(!board.can_flag());

        // Attempting to place a 3rd flag is rejected
        board.toggle_flag(0, 2);
        assert_eq!(board.cells[board.get_index(0, 2)].state, CellState::Covered);
        assert_eq!(board.flags_placed, 2);

        // Removing a flag restores capacity
        board.toggle_flag(0, 1);
        assert_eq!(board.flags_placed, 1);
        assert!(board.can_flag());
        assert_eq!(board.remaining_flags(), Some(1));
    }

    #[test]
    fn test_ring2_neighbors_square_and_hex() {
        let sq_board = Board::new_square(5, 5, 0);
        // Center cell (2, 2) in a 5x5 square has 8 direct neighbors and 16 ring-2 neighbors
        let sq_r2 = sq_board.ring2_neighbors(2, 2);
        assert_eq!(sq_r2.len(), 16);

        let hex_board = Board::new_hex(3, 0);
        // Center cell of hex board is at (3, 3) in array representation.
        // Direct neighbors: 6. Ring-2 neighbors: 12.
        let hex_r2 = hex_board.ring2_neighbors(3, 3);
        assert_eq!(hex_r2.len(), 12);
    }

    #[test]
    fn test_chording() {
        let mut board = Board::new_square(9, 9, 0);
        board.first_click = false;

        let idx1 = board.get_index(1, 1);
        board.cells[idx1].is_mine = true;
        let idx2 = board.get_index(0, 1);
        board.cells[idx2].adjacent_mines = 1;

        board.reveal(0, 1);
        assert_eq!(
            board.cells[board.get_index(0, 1)].state,
            CellState::Revealed
        );

        // Chord before flagging does nothing
        board.chord(0, 1);
        assert_eq!(board.cells[board.get_index(0, 0)].state, CellState::Covered);

        board.toggle_flag(1, 1);
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

        let active_cells = board.cells.iter().filter(|c| c.active).count();
        assert_eq!(active_cells, 91);
    }

    #[test]
    fn test_no_flags_victory_condition() {
        // Create 3x3 board with 1 mine at (2, 2)
        let mut ruleset = Ruleset::classic_square(3, 3, 1);
        ruleset.flag_policy = FlagPolicy::NoFlags;
        let mut board = Board::from_ruleset(ruleset);
        board.first_click = false;

        // Manually place mine at (2, 2)
        let m_idx = board.get_index(2, 2);
        board.cells[m_idx].is_mine = true;

        // Reveal all 8 safe cells without flagging
        for y in 0..3 {
            for x in 0..3 {
                if !(x == 2 && y == 2) {
                    board.reveal(x, y);
                }
            }
        }

        assert_eq!(
            board.status,
            GameStatus::Won,
            "Board must detect victory when all safe cells revealed under NoFlags"
        );
        assert_eq!(board.flags_placed, 0, "No flags should have been placed");
    }

    #[test]
    fn test_first_click_guaranteed_zero_opening() {
        let mut ruleset = Ruleset::classic_square(9, 9, 10);
        ruleset.first_click_policy = FirstClickPolicy::GuaranteedZeroOpening;
        let mut board = Board::from_ruleset(ruleset);

        board.reveal(4, 4);
        assert!(!board.first_click);

        let cell = board.get_cell(4, 4).unwrap();
        assert_eq!(
            cell.adjacent_mines, 0,
            "Initial clicked cell must have 0 adjacent mines"
        );
        assert_eq!(cell.state, CellState::Revealed);

        // All direct neighbors must be safe and revealed by flood
        for (nx, ny) in board.neighbors(4, 4) {
            let n_cell = board.get_cell(nx, ny).unwrap();
            assert!(
                !n_cell.is_mine,
                "Neighbors must not be mines in GuaranteedZeroOpening"
            );
            assert_eq!(
                n_cell.state,
                CellState::Revealed,
                "Zero opening must flood-reveal all neighbors"
            );
        }
    }

    #[test]
    fn test_extended_ring2_clue_independent_oracle() {
        // Construct 5x5 board
        let mut ruleset = Ruleset::classic_square(5, 5, 0);
        ruleset.clue_policy = CluePolicy::ExtendedRing2;
        let mut board = Board::from_ruleset(ruleset);
        board.first_click = false;

        // Place mines at distance 1 and distance 2 from center (2, 2)
        // Distance 1: (2, 1) and (3, 2) -> 2 adjacent mines
        let idx_d1_a = board.get_index(2, 1);
        let idx_d1_b = board.get_index(3, 2);
        board.cells[idx_d1_a].is_mine = true;
        board.cells[idx_d1_b].is_mine = true;

        // Distance 2 (Ring 2): (0, 0), (4, 4), (2, 4) -> 3 ring-2 mines
        let idx_r2_a = board.get_index(0, 0);
        let idx_r2_b = board.get_index(4, 4);
        let idx_r2_c = board.get_index(2, 4);
        board.cells[idx_r2_a].is_mine = true;
        board.cells[idx_r2_b].is_mine = true;
        board.cells[idx_r2_c].is_mine = true;

        // Recompute clues manually
        let center_idx = board.get_index(2, 2);
        board.cells[center_idx].clue_type = ClueType::Ring2;

        let mut adj_count = 0;
        for (nx, ny) in board.neighbors(2, 2) {
            if board.cells[board.get_index(nx, ny)].is_mine {
                adj_count += 1;
            }
        }
        board.cells[center_idx].adjacent_mines = adj_count;

        let mut r2_count = 0;
        for (r2x, r2y) in board.ring2_neighbors(2, 2) {
            if board.cells[board.get_index(r2x, r2y)].is_mine {
                r2_count += 1;
            }
        }
        board.cells[center_idx].ring2_mines = r2_count;

        assert_eq!(
            board.cells[center_idx].adjacent_mines, 2,
            "Adjacent mines must equal 2"
        );
        assert_eq!(
            board.cells[center_idx].ring2_mines, 3,
            "Ring-2 mines must equal 3"
        );
    }

    #[test]
    fn test_loss_detection_and_reveal_mines() {
        let mut board = Board::new_square(5, 5, 2);
        board.first_click = false;
        let m1 = board.get_index(1, 1);
        let m2 = board.get_index(3, 3);
        board.cells[m1].is_mine = true;
        board.cells[m2].is_mine = true;

        board.reveal(1, 1);
        assert_eq!(board.status, GameStatus::Lost);
        assert_eq!(board.cells[m1].state, CellState::Revealed);
        assert_eq!(
            board.cells[m2].state,
            CellState::Revealed,
            "All mines must be revealed upon defeat"
        );
    }
}
