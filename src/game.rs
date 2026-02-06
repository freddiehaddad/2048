use crate::board::Board;

pub const GAME_TITLE: &str = " 2048 ";

#[derive(Default)]
pub struct Game {
    board: Board,
    score: u32,
}

impl Game {
    // ========================================================================
    // Lifecycle
    // ========================================================================

    pub fn new() -> Self {
        Self {
            board: Board::new(),
            score: 0,
        }
    }

    pub fn reset(&mut self) {
        self.board.reset();
        self.score = 0;
    }

    // ========================================================================
    // Game State Queries
    // ========================================================================

    pub fn is_game_over(&self) -> bool {
        self.board.is_game_over()
    }

    // ========================================================================
    // Game Actions
    // ========================================================================

    pub fn move_up(&mut self) {
        let results = self.board.move_up();
        self.process_move_results(results.into_iter());
    }

    pub fn move_down(&mut self) {
        let results = self.board.move_down();
        self.process_move_results(results.into_iter());
    }

    pub fn move_left(&mut self) {
        let results = self.board.move_left();
        self.process_move_results(results.into_iter());
    }

    pub fn move_right(&mut self) {
        let results = self.board.move_right();
        self.process_move_results(results.into_iter());
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn process_move_results(
        &mut self,
        results: impl Iterator<Item = crate::board::MergeResult>,
    ) {
        let mut board_changed = false;

        for result in results {
            if result.board_changed {
                board_changed = true;
            }

            // Score is the sum of the resulting values of any merges.
            // merged_sources contains the input values (e.g. two 2s merging puts one 2 here).
            // So we add value * 2.
            for &source_value in result.merged_sources.iter() {
                self.score += source_value * 2;
            }
        }

        if board_changed {
            self.board.spawn_tile();
        }
    }

    // ========================================================================
    // Board State Queries
    // ========================================================================

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn score(&self) -> u32 {
        self.score
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    impl Game {
        pub fn set_board(
            &mut self,
            tiles: [[Option<u32>; crate::board::BOARD_SIZE];
                crate::board::BOARD_SIZE],
        ) {
            self.board.set_tiles(tiles);
        }
    }

    #[test]
    fn test_score_update_on_merge() {
        let mut game = Game::new();
        game.set_board([
            [Some(2), Some(2), None, None],
            [None; 4],
            [None; 4],
            [None; 4],
        ]);
        game.score = 0;

        // Move Left: 2+2 -> 4. Score should increase by 4.
        game.move_left();

        assert_eq!(game.score, 4);
    }

    #[test]
    fn test_no_score_update_on_slide() {
        let mut game = Game::new();
        game.set_board([
            [Some(2), None, None, None],
            [None; 4],
            [None; 4],
            [None; 4],
        ]);
        game.score = 10;

        // Move Left: 2 slides but no merge.
        game.move_left();

        assert_eq!(game.score, 10);
    }

    #[test]
    fn test_game_over_delegation() {
        let mut game = Game::new();
        // Empty board
        game.set_board([[None; 4]; 4]);
        assert!(!game.is_game_over());

        // Full locked board
        game.set_board([
            [Some(2), Some(4), Some(8), Some(16)],
            [Some(32), Some(64), Some(128), Some(256)],
            [Some(2), Some(4), Some(8), Some(16)],
            [Some(32), Some(64), Some(128), Some(256)],
        ]);
        assert!(game.is_game_over());
    }

    #[test]
    fn test_reset_game() {
        let mut game = Game::new();
        game.score = 500;
        game.set_board([[Some(2); 4]; 4]);

        game.reset();

        assert_eq!(game.score, 0);
        // Board should be reset (2 tiles)
        let tile_count =
            game.board().iter_tiles().filter(|t| t.is_some()).count();
        assert_eq!(tile_count, 2);
    }
}
