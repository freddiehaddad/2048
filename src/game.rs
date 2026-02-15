use rand::prelude::*;

use crate::board::{BOARD_SIZE, Board};

pub(crate) const TITLE: &str = " 2048 ";

const STARTING_TILE_COUNT: usize = 2;
const STARTING_TILE_TWO: u32 = 2;
const STARTING_TILE_FOUR: u32 = 4;
const STARTING_TILE_TWO_PROBABILITY: f64 = 0.9;

#[derive(Debug)]
pub enum GameAction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default)]
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

    pub fn restart(&mut self) -> ActionOutcome {
        self.score = 0;
        self.board = Game::initialize_board();

        ActionOutcome::from((self.score, &self.board))
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn apply_move(&mut self, direction: GameAction) -> ActionOutcome {
        match direction {
            GameAction::Up => self.slide_and_merge(direction),
            GameAction::Down => self.slide_and_merge(direction),
            GameAction::Left => self.slide_and_merge(direction),
            GameAction::Right => self.slide_and_merge(direction),
        }
    }

    // Helper function that slides and merges the tiles in a single column
    // according to the game rules, updating the board and score as necessary.
    fn slide_and_merge_column(
        &self,
        tiles: impl Iterator<Item = u32>,
        rows: impl Iterator<Item = usize>,
        col: usize,
        board: &mut [[CellResult; BOARD_SIZE]; BOARD_SIZE],
        score: &mut u32,
    ) {
        let mut tiles = tiles.peekable();
        for row in rows {
            let Some(tile) = tiles.next() else {
                break;
            };

            if let Some(&next_tile) = tiles.peek()
                && tile == next_tile
            {
                let tile_sum = tile + next_tile;
                board[row][col] = CellResult {
                    value: Some(tile_sum),
                    merged: true,
                };
                *score += tile_sum;
                tiles.next();
            } else {
                board[row][col] = CellResult {
                    value: Some(tile),
                    merged: false,
                };
            }
        }
    }

    // Slides and merges the tiles in the given direction according to the game
    // rules, updating the board and score as necessary.
    fn slide_and_merge(&mut self, direction: GameAction) -> ActionOutcome {
        let mut board: [[CellResult; BOARD_SIZE]; BOARD_SIZE] =
            [[CellResult::default(); BOARD_SIZE]; BOARD_SIZE];
        let mut score = 0;

        match direction {
            GameAction::Up => {
                for col in 0..BOARD_SIZE {
                    self.slide_and_merge_column(
                        self.board.col(col),
                        0..BOARD_SIZE,
                        col,
                        &mut board,
                        &mut score,
                    );
                }
            }
            GameAction::Down => {
                for col in 0..BOARD_SIZE {
                    self.slide_and_merge_column(
                        self.board.col(col).rev(),
                        (0..BOARD_SIZE).rev(),
                        col,
                        &mut board,
                        &mut score,
                    );
                }
            }
            GameAction::Left => unimplemented!(),
            GameAction::Right => unimplemented!(),
        }

        // Update the game board with the new tile positions and values.
        for ((row, col), cell) in
            board.iter().enumerate().flat_map(|(row, row_cells)| {
                row_cells
                    .iter()
                    .enumerate()
                    .map(move |(col, col_cell)| ((row, col), col_cell))
            })
        {
            self.board.set_cell(row, col, cell.value);
        }

        ActionOutcome { score, board }
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

    // Initializes the board with the starting tiles in random positions.
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
            board.set_cell(row, col, Some(Game::spawn_tile()));
        }

        board
    }
}
