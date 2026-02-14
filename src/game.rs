use crate::board::{BOARD_SIZE, Board};

pub(crate) const TITLE: &str = " 2048 ";

const STARTING_TILE_TWO: u32 = 2;
const STARTING_TILE_FOUR: u32 = 4;
const STARTING_TILE_TWO_PROBABILITY: f64 = 0.9;
const STARTING_TILE_FOUR_PROBABILITY: f64 = 0.1;

#[derive(Debug)]
pub enum GameAction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Default)]
pub struct CellResult {
    pub value: Option<u32>,
    pub merged: bool,
}

#[derive(Debug, Default)]
pub struct ActionOutcome {
    pub board: [[CellResult; BOARD_SIZE]; BOARD_SIZE],
    pub score: u32,
}

#[derive(Debug, Default)]
pub struct Game {
    board: Board,
    score: u32,
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn apply_move(&mut self, direction: GameAction) -> ActionOutcome {
        ActionOutcome::default()
    }

    pub fn restart(&mut self) -> ActionOutcome {
        ActionOutcome::default()
    }
}
