use rand::prelude::*;

use crate::board::{BOARD_SIZE, Board};

pub(crate) const TITLE: &str = " 2048 ";

const STARTING_TILE_COUNT: usize = 2;
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
    pub score: u32,
    pub board: [[CellResult; BOARD_SIZE]; BOARD_SIZE],
}

impl From<(u32, &Board)> for ActionOutcome {
    fn from((score, board): (u32, &Board)) -> Self {
        let mut outcome = ActionOutcome {
            score,
            ..Default::default()
        };

        for ((row, col), cell) in board.iter_cells() {
            outcome.board[row][col].value = *cell;
        }

        outcome
    }
}

#[derive(Debug, Default)]
pub struct Game {
    score: u32,
    board: Board,
}

impl Game {
    pub fn new() -> Self {
        Self {
            score: 0,
            board: Game::initialize_board(),
        }
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn apply_move(&mut self, direction: GameAction) -> ActionOutcome {
        ActionOutcome::default()
    }

    pub fn restart(&mut self) -> ActionOutcome {
        self.score = 0;
        self.board = Game::initialize_board();

        ActionOutcome::from((self.score, &self.board))
    }

    // Spawns a new tile with the appropriate probability distribution.
    fn spawn_tile() -> u32 {
        let mut rng = rand::rng();
        if rng.random_bool(STARTING_TILE_TWO_PROBABILITY) {
            STARTING_TILE_TWO
        } else {
            STARTING_TILE_FOUR
        }
    }

    fn initialize_board() -> Board {
        // Buffer that will be filled with random coordinates to place the
        // starting tiles.
        let mut cells: [Option<(usize, usize)>; STARTING_TILE_COUNT] =
            [None; STARTING_TILE_COUNT];

        let mut board = Board::default();

        // Pick random coordinates on the board to place the starting tiles.
        board
            .iter_cells()
            .map(|(coord, _)| Some(coord))
            .sample_fill(&mut rand::rng(), &mut cells);

        // Place the starting tiles on the board.
        for (row, col) in cells.into_iter().flatten() {
            board.set_cell(row, col, Game::spawn_tile());
        }

        board
    }
}
