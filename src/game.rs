use crate::board::Board;

const STARTING_TILE_TWO: u32 = 2;
const STARTING_TILE_FOUR: u32 = 4;
const STARTING_TILE_TWO_PROBABILITY: f64 = 0.9;
const STARTING_TILE_FOUR_PROBABILITY: f64 = 0.1;

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
}
