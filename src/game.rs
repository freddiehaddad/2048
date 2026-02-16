use anyhow::{Result, bail};
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
    pub changed: bool,
    pub game_over: bool,
    pub board: [[CellResult; BOARD_SIZE]; BOARD_SIZE],
}

impl ActionOutcome {
    fn iter_cells(
        &self,
    ) -> impl Iterator<Item = ((usize, usize), &CellResult)> {
        self.board.iter().enumerate().flat_map(|(row, row_cells)| {
            row_cells
                .iter()
                .enumerate()
                .map(move |(col, col_cell)| ((row, col), col_cell))
        })
    }
}

impl From<&Game> for ActionOutcome {
    fn from(game: &Game) -> Self {
        let mut outcome = ActionOutcome {
            score: game.score,
            game_over: game.game_over,
            ..Default::default()
        };

        for ((row, col), cell) in game.board.iter_cells() {
            outcome.board[row][col].value = *cell;
        }

        outcome
    }
}

#[derive(Debug, Default)]
pub struct Game {
    board: Board,
    score: u32,
    game_over: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Game::initialize_board(),
            ..Default::default()
        }
    }

    pub fn outcome(&self) -> ActionOutcome {
        ActionOutcome::from(self)
    }

    pub fn restart(&mut self) -> ActionOutcome {
        self.score = 0;
        self.game_over = false;
        self.board = Game::initialize_board();

        self.outcome()
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn apply_move(
        &mut self,
        direction: GameAction,
    ) -> Result<ActionOutcome> {
        if self.is_game_over() {
            return Ok(self.outcome());
        }

        let mut outcome = ActionOutcome::default();
        self.slide_and_merge(direction, &mut outcome);
        self.update_board(&mut outcome);

        self.update_score(&mut outcome);

        if outcome.changed {
            self.spawn_random_tile(&mut outcome)?;
            self.update_board(&mut outcome);
        }

        self.check_game_over(&mut outcome);

        Ok(outcome)
    }

    fn update_score(&mut self, outcome: &mut ActionOutcome) {
        self.score += outcome.score;
        outcome.score = self.score;
    }

    // Helper function that slides and merges a single line of tiles in the given
    // direction, updating the board and score as necessary.
    fn slide_and_merge_line(
        &self,
        tiles: impl Iterator<Item = u32>,
        positions: impl Iterator<Item = (usize, usize)>,
        board: &mut [[CellResult; BOARD_SIZE]; BOARD_SIZE],
        score: &mut u32,
    ) {
        let mut tiles = tiles.peekable();
        for (row, col) in positions {
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
    fn slide_and_merge(
        &self,
        direction: GameAction,
        outcome: &mut ActionOutcome,
    ) {
        match direction {
            GameAction::Up => {
                for col in 0..BOARD_SIZE {
                    self.slide_and_merge_line(
                        self.board.col(col),
                        (0..BOARD_SIZE).map(|row| (row, col)),
                        &mut outcome.board,
                        &mut outcome.score,
                    );
                }
            }
            GameAction::Down => {
                for col in 0..BOARD_SIZE {
                    self.slide_and_merge_line(
                        self.board.col(col).rev(),
                        (0..BOARD_SIZE).map(|row| (row, col)).rev(),
                        &mut outcome.board,
                        &mut outcome.score,
                    );
                }
            }
            GameAction::Left => {
                for row in 0..BOARD_SIZE {
                    self.slide_and_merge_line(
                        self.board.row(row),
                        (0..BOARD_SIZE).map(|col| (row, col)),
                        &mut outcome.board,
                        &mut outcome.score,
                    );
                }
            }
            GameAction::Right => {
                for row in 0..BOARD_SIZE {
                    self.slide_and_merge_line(
                        self.board.row(row).rev(),
                        (0..BOARD_SIZE).map(|col| (row, col)).rev(),
                        &mut outcome.board,
                        &mut outcome.score,
                    );
                }
            }
        }
    }

    fn check_game_over(&mut self, outcome: &mut ActionOutcome) {
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                // If there is an empty cell, the game is not over.
                let Some(current_tile) = self.board.cell(row, col) else {
                    return;
                };

                // If there is a mergeable tile to the right.
                if col + 1 < BOARD_SIZE
                    && self.board.cell(row, col + 1) == Some(current_tile)
                {
                    return;
                }

                // If there is a mergeable tile below, the game is not over.
                if row + 1 < BOARD_SIZE
                    && self.board.cell(row + 1, col) == Some(current_tile)
                {
                    return;
                }
            }
        }

        outcome.game_over = true;
        self.game_over |= outcome.game_over;
    }

    fn update_board(&mut self, outcome: &mut ActionOutcome) {
        let mut changed = false;
        for ((row, col), cell) in outcome.iter_cells() {
            if cell.value != self.board.cell(row, col) {
                *self.board.cell_mut(row, col) = cell.value;
                changed = true;
            }
        }
        outcome.changed |= changed;
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

    fn spawn_random_tile(&self, outcome: &mut ActionOutcome) -> Result<()> {
        // Pick random coordinates on the board to place the starting tiles.
        let Some((row, col)) = outcome
            .iter_cells()
            .filter(|(_, cell)| cell.value.is_none())
            .map(|(pos, _)| pos)
            .choose(&mut rand::rng())
        else {
            bail!("No empty cell available to spawn a random tile");
        };

        // Place the starting tiles on the board.
        outcome.board[row][col] = CellResult {
            value: Some(Game::spawn_tile()),
            ..Default::default()
        };

        Ok(())
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
            *board.cell_mut(row, col) = Some(Game::spawn_tile());
        }

        board
    }
}
