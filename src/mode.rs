use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameModeId {
    Classic,
    Sprint,
    NoGuess,
    Blitz,
    Survival,
    Daily,
    Expedition,
    Zen,
    Sandbox,
}

impl GameModeId {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Sprint => "Sprint",
            Self::NoGuess => "No Guess",
            Self::Blitz => "Blitz",
            Self::Survival => "Survival",
            Self::Daily => "Daily Challenge",
            Self::Expedition => "Expedition",
            Self::Zen => "Zen",
            Self::Sandbox => "Sandbox",
        }
    }

    pub fn desc(&self) -> &'static str {
        match self {
            Self::Classic => "Standard rules. No time pressure.",
            Self::Sprint => "Clear the board as fast as possible.",
            Self::NoGuess => "100% logically solvable puzzles.",
            Self::Blitz => "Score points before time runs out.",
            Self::Survival => "Clear stages with limited lives.",
            Self::Daily => "Fixed daily reproducible challenge.",
            Self::Expedition => "30 handcrafted campaign levels.",
            Self::Zen => "Relaxing, with optional hints.",
            Self::Sandbox => "Fully customizable rules.",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModeState {
    pub id: GameModeId,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub score: i64,
    pub lives: i32,
    pub stage: i32,
    pub time_limit_sec: Option<f64>,
}

impl ModeState {
    pub fn new(id: GameModeId) -> Self {
        let (lives, time_limit_sec) = match id {
            GameModeId::Survival => (3, None),
            GameModeId::Blitz => (1, Some(180.0)),
            _ => (1, None),
        };
        Self {
            id,
            start_time: None,
            end_time: None,
            score: 0,
            lives,
            stage: 1,
            time_limit_sec,
        }
    }
}
