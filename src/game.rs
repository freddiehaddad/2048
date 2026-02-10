use crate::board::{BOARD_SIZE, Board};

pub const GAME_TITLE: &str = " 2048 ";

#[derive(Default, Clone, Copy)]
pub struct TurnResult {
    // Merged cells for UI highlighting.
    pub merged: [[bool; BOARD_SIZE]; BOARD_SIZE],
}

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

    pub fn move_up(&mut self) -> TurnResult {
        let outcome = self.board.move_up();
        self.handle_outcome(outcome)
    }

    pub fn move_down(&mut self) -> TurnResult {
        let outcome = self.board.move_down();
        self.handle_outcome(outcome)
    }

    pub fn move_left(&mut self) -> TurnResult {
        let outcome = self.board.move_left();
        self.handle_outcome(outcome)
    }

    pub fn move_right(&mut self) -> TurnResult {
        let outcome = self.board.move_right();
        self.handle_outcome(outcome)
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn handle_outcome(
        &mut self,
        outcome: crate::board::MoveOutcome,
    ) -> TurnResult {
        if outcome.board_changed {
            self.board.spawn_tile();
        }

        self.score += outcome.score_delta;

        TurnResult {
            merged: outcome.merged,
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

    #[test]
    fn test_spawn_on_move() {
        let mut game = Game::new();
        game.set_board([
            [Some(2), None, None, None],
            [None; 4],
            [None; 4],
            [None; 4],
        ]);

        // Move Left: No change (already at edge)
        game.move_left();
        let count_after_no_change =
            game.board().iter_tiles().filter(|t| t.is_some()).count();
        assert_eq!(
            count_after_no_change, 1,
            "Should not spawn if board didn't change"
        );

        // Move Right: Change (slides to right)
        game.move_right();
        let count_after_change =
            game.board().iter_tiles().filter(|t| t.is_some()).count();
        assert_eq!(count_after_change, 2, "Should spawn if board changed");
    }
}
