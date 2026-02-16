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

        // When restarting, we want to treat the new board as changed so that
        // the UI can update to show the new starting tiles.
        let mut outcome = self.outcome();
        outcome.changed = true;
        outcome
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

#[cfg(test)]
mod tests {
    use super::*;

    fn board_from_rows(rows: [[Option<u32>; BOARD_SIZE]; BOARD_SIZE]) -> Board {
        let mut board = Board::default();
        for (row, row_cells) in rows.iter().enumerate() {
            for (col, value) in row_cells.iter().enumerate() {
                *board.cell_mut(row, col) = *value;
            }
        }
        board
    }

    fn game_from_rows(
        rows: [[Option<u32>; BOARD_SIZE]; BOARD_SIZE],
        score: u32,
        game_over: bool,
    ) -> Game {
        Game {
            board: board_from_rows(rows),
            score,
            game_over,
        }
    }

    fn outcome_values(
        outcome: &ActionOutcome,
    ) -> [[Option<u32>; BOARD_SIZE]; BOARD_SIZE] {
        let mut values = [[None; BOARD_SIZE]; BOARD_SIZE];
        for (row, row_values) in values.iter_mut().enumerate() {
            for (col, value) in row_values.iter_mut().enumerate() {
                *value = outcome.board[row][col].value;
            }
        }
        values
    }

    fn count_filled(values: &[[Option<u32>; BOARD_SIZE]; BOARD_SIZE]) -> usize {
        values
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.is_some())
            .count()
    }

    #[test]
    fn slide_and_merge_line_merges_each_pair_once() {
        let game = Game::default();
        let mut board = [[CellResult::default(); BOARD_SIZE]; BOARD_SIZE];
        let mut score = 0;

        game.slide_and_merge_line(
            vec![2, 2, 2, 2].into_iter(),
            (0..BOARD_SIZE).map(|col| (0, col)),
            &mut board,
            &mut score,
        );

        assert_eq!(score, 8);
        assert_eq!(board[0][0].value, Some(4));
        assert!(board[0][0].merged);
        assert_eq!(board[0][1].value, Some(4));
        assert!(board[0][1].merged);
        assert_eq!(board[0][2].value, None);
        assert_eq!(board[0][3].value, None);
    }

    #[test]
    fn slide_and_merge_up_merges_columns_correctly() {
        let game = game_from_rows(
            [
                [Some(2), None, None, None],
                [Some(2), None, None, None],
                [Some(4), None, None, None],
                [Some(4), None, None, None],
            ],
            0,
            false,
        );
        let mut outcome = ActionOutcome::default();

        game.slide_and_merge(GameAction::Up, &mut outcome);

        assert_eq!(
            outcome_values(&outcome),
            [
                [Some(4), None, None, None],
                [Some(8), None, None, None],
                [None, None, None, None],
                [None, None, None, None],
            ]
        );
        assert_eq!(outcome.score, 12);
        assert!(outcome.board[0][0].merged);
        assert!(outcome.board[1][0].merged);
    }

    #[test]
    fn slide_and_merge_down_merges_columns_correctly() {
        let game = game_from_rows(
            [
                [Some(2), None, None, None],
                [Some(2), None, None, None],
                [Some(4), None, None, None],
                [Some(4), None, None, None],
            ],
            0,
            false,
        );
        let mut outcome = ActionOutcome::default();

        game.slide_and_merge(GameAction::Down, &mut outcome);

        assert_eq!(
            outcome_values(&outcome),
            [
                [None, None, None, None],
                [None, None, None, None],
                [Some(4), None, None, None],
                [Some(8), None, None, None],
            ]
        );
        assert_eq!(outcome.score, 12);
        assert!(outcome.board[2][0].merged);
        assert!(outcome.board[3][0].merged);
    }

    #[test]
    fn slide_and_merge_right_compacts_toward_right_edge() {
        let game = game_from_rows(
            [
                [Some(2), None, Some(2), Some(2)],
                [None, None, None, None],
                [None, None, None, None],
                [None, None, None, None],
            ],
            0,
            false,
        );
        let mut outcome = ActionOutcome::default();

        game.slide_and_merge(GameAction::Right, &mut outcome);

        assert_eq!(
            outcome_values(&outcome),
            [
                [None, None, Some(2), Some(4)],
                [None, None, None, None],
                [None, None, None, None],
                [None, None, None, None],
            ]
        );
        assert_eq!(outcome.score, 4);
        assert!(outcome.board[0][3].merged);
    }

    #[test]
    fn update_board_sets_changed_only_when_board_differs() {
        let mut game = game_from_rows(
            [
                [Some(2), None, None, None],
                [None, None, None, None],
                [None, None, None, None],
                [None, None, None, None],
            ],
            0,
            false,
        );
        let mut outcome = game.outcome();

        game.update_board(&mut outcome);
        assert!(!outcome.changed);

        outcome.board[0][0].value = Some(8);
        game.update_board(&mut outcome);

        assert!(outcome.changed);
        assert_eq!(game.board.cell(0, 0), Some(8));
    }

    #[test]
    fn check_game_over_is_false_when_empty_cells_exist() {
        let mut game = game_from_rows(
            [
                [Some(2), None, Some(4), Some(8)],
                [Some(16), Some(32), Some(64), Some(128)],
                [Some(256), Some(512), Some(1024), Some(2048)],
                [Some(4096), Some(8192), Some(16384), Some(32768)],
            ],
            0,
            false,
        );
        let mut outcome = ActionOutcome::default();

        game.check_game_over(&mut outcome);

        assert!(!outcome.game_over);
        assert!(!game.is_game_over());
    }

    #[test]
    fn check_game_over_detects_merge_on_last_row() {
        let mut game = game_from_rows(
            [
                [Some(2), Some(4), Some(8), Some(16)],
                [Some(32), Some(64), Some(128), Some(256)],
                [Some(512), Some(1024), Some(2048), Some(4096)],
                [Some(3), Some(6), Some(12), Some(12)],
            ],
            0,
            false,
        );
        let mut outcome = ActionOutcome::default();

        game.check_game_over(&mut outcome);

        assert!(!outcome.game_over);
        assert!(!game.is_game_over());
    }

    #[test]
    fn check_game_over_detects_merge_on_last_column() {
        let mut game = game_from_rows(
            [
                [Some(2), Some(4), Some(8), Some(16)],
                [Some(32), Some(64), Some(128), Some(16)],
                [Some(1024), Some(2048), Some(4096), Some(512)],
                [Some(8192), Some(16384), Some(32768), Some(65536)],
            ],
            0,
            false,
        );
        let mut outcome = ActionOutcome::default();

        game.check_game_over(&mut outcome);

        assert!(!outcome.game_over);
        assert!(!game.is_game_over());
    }

    #[test]
    fn check_game_over_sets_true_when_full_without_merges() {
        let mut game = game_from_rows(
            [
                [Some(2), Some(4), Some(8), Some(16)],
                [Some(32), Some(64), Some(128), Some(256)],
                [Some(512), Some(1024), Some(2048), Some(4096)],
                [Some(3), Some(6), Some(12), Some(24)],
            ],
            0,
            false,
        );
        let mut outcome = ActionOutcome::default();

        game.check_game_over(&mut outcome);

        assert!(outcome.game_over);
        assert!(game.is_game_over());
    }

    #[test]
    fn spawn_random_tile_places_value_in_only_empty_slot() {
        let game = Game::default();
        let mut outcome = ActionOutcome::default();
        let mut values = [
            [Some(8), Some(16), Some(32), Some(64)],
            [Some(128), Some(256), None, Some(512)],
            [Some(1024), Some(2048), Some(4096), Some(8192)],
            [Some(3), Some(6), Some(12), Some(24)],
        ];

        for (row, row_values) in values.iter().enumerate() {
            for (col, value) in row_values.iter().enumerate() {
                outcome.board[row][col].value = *value;
            }
        }

        game.spawn_random_tile(&mut outcome).unwrap();
        values[1][2] = outcome.board[1][2].value;

        assert!(matches!(values[1][2], Some(2 | 4)));
        assert!(!outcome.board[1][2].merged);
    }

    #[test]
    fn spawn_random_tile_returns_error_when_no_empty_cells() {
        let game = Game::default();
        let mut outcome = ActionOutcome::default();
        let values = [
            [Some(2), Some(4), Some(8), Some(16)],
            [Some(32), Some(64), Some(128), Some(256)],
            [Some(512), Some(1024), Some(2048), Some(4096)],
            [Some(3), Some(6), Some(12), Some(24)],
        ];

        for (row, row_values) in values.iter().enumerate() {
            for (col, value) in row_values.iter().enumerate() {
                outcome.board[row][col].value = *value;
            }
        }

        assert!(game.spawn_random_tile(&mut outcome).is_err());
    }

    #[test]
    fn apply_move_returns_snapshot_when_game_is_already_over() {
        let mut game = game_from_rows(
            [
                [Some(2), Some(4), Some(8), Some(16)],
                [Some(32), Some(64), Some(128), Some(256)],
                [Some(512), Some(1024), Some(2048), Some(4096)],
                [Some(3), Some(6), Some(12), Some(24)],
            ],
            77,
            true,
        );
        let before = game.outcome();

        let outcome = game.apply_move(GameAction::Left).unwrap();

        assert_eq!(outcome.score, before.score);
        assert_eq!(outcome.game_over, before.game_over);
        assert_eq!(outcome_values(&outcome), outcome_values(&before));
        assert!(!outcome.changed);
    }

    #[test]
    fn apply_move_without_board_change_does_not_spawn_tile() {
        let mut game = game_from_rows(
            [
                [Some(2), None, None, None],
                [None, None, None, None],
                [None, None, None, None],
                [None, None, None, None],
            ],
            10,
            false,
        );

        let outcome = game.apply_move(GameAction::Left).unwrap();
        let values = outcome_values(&outcome);

        assert!(!outcome.changed);
        assert_eq!(outcome.score, 10);
        assert_eq!(values[0][0], Some(2));
        assert_eq!(count_filled(&values), 1);
        assert_eq!(game.score, 10);
    }

    #[test]
    fn apply_move_merge_updates_score_and_spawns_single_tile() {
        let mut game = game_from_rows(
            [
                [Some(2), Some(2), None, None],
                [None, None, None, None],
                [None, None, None, None],
                [None, None, None, None],
            ],
            0,
            false,
        );

        let outcome = game.apply_move(GameAction::Left).unwrap();
        let values = outcome_values(&outcome);
        let spawned_tiles: Vec<u32> = values
            .iter()
            .enumerate()
            .flat_map(|(row, cols)| {
                cols.iter().enumerate().filter_map(move |(col, value)| {
                    match (row, col, value) {
                        (0, 0, Some(4)) => None,
                        (_, _, Some(v)) => Some(*v),
                        _ => None,
                    }
                })
            })
            .collect();

        assert!(outcome.changed);
        assert_eq!(outcome.score, 4);
        assert_eq!(values[0][0], Some(4));
        assert_eq!(count_filled(&values), 2);
        assert_eq!(spawned_tiles.len(), 1);
        assert!(matches!(spawned_tiles[0], 2 | 4));
        assert_eq!(game.score, 4);
    }

    #[test]
    fn apply_move_can_set_game_over_after_spawn_fills_last_empty_cell() {
        let mut game = game_from_rows(
            [
                [None, Some(8), Some(16), Some(32)],
                [Some(64), Some(128), Some(256), Some(512)],
                [Some(1024), Some(2048), Some(4096), Some(8192)],
                [Some(16384), Some(32768), Some(65536), Some(131072)],
            ],
            0,
            false,
        );

        let outcome = game.apply_move(GameAction::Left).unwrap();
        let values = outcome_values(&outcome);

        assert!(outcome.changed);
        assert!(outcome.game_over);
        assert!(game.is_game_over());
        assert_eq!(count_filled(&values), BOARD_SIZE * BOARD_SIZE);
    }

    #[test]
    fn restart_resets_state_and_creates_starting_tiles() {
        let mut game = game_from_rows(
            [
                [Some(2), Some(4), Some(8), Some(16)],
                [Some(32), Some(64), Some(128), Some(256)],
                [Some(512), Some(1024), Some(2048), Some(4096)],
                [Some(3), Some(6), Some(12), Some(24)],
            ],
            999,
            true,
        );

        let outcome = game.restart();
        let values = outcome_values(&outcome);
        let tiles: Vec<u32> = values
            .iter()
            .flat_map(|row| row.iter().filter_map(|cell| *cell))
            .collect();

        assert_eq!(outcome.score, 0);
        assert!(!outcome.game_over);
        assert!(!game.is_game_over());
        assert_eq!(game.score, 0);
        assert_eq!(tiles.len(), STARTING_TILE_COUNT);
        assert!(tiles.iter().all(|value| matches!(value, 2 | 4)));
    }
}
