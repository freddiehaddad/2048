use rand::prelude::*;

pub(crate) const BOARD_SIZE: usize = 4;

const SPAWN_RATE_2: f64 = 0.9;
const TILE_VALUE_2: u32 = 2;
const TILE_VALUE_4: u32 = 4;

#[derive(Default)]
pub struct Board {
    tiles: [[Option<u32>; BOARD_SIZE]; BOARD_SIZE],
}

#[derive(Default)]
pub struct MergeResult {
    pub merged_tiles: Vec<u32>,
    pub merged_sources: Vec<u32>,
    pub board_changed: bool,
}

impl MergeResult {
    fn new(merged_tiles: Vec<u32>, merged_values: Vec<u32>) -> Self {
        Self {
            merged_tiles,
            merged_sources: merged_values,
            board_changed: false,
        }
    }
}

impl Board {
    // ========================================================================
    // Lifecycle
    // ========================================================================

    pub fn new() -> Self {
        let mut board = Board::default();
        board.initialize();
        board
    }

    pub fn reset(&mut self) {
        self.tiles = Default::default();
        self.initialize();
    }

    // ========================================================================
    // Game Actions
    // ========================================================================

    pub fn move_up(&mut self) -> Vec<MergeResult> {
        self.transpose();
        let results = (0..BOARD_SIZE)
            .map(|row| self.slide_row(row))
            .collect::<Vec<_>>();
        self.transpose();
        results
    }

    pub fn move_down(&mut self) -> Vec<MergeResult> {
        self.transpose();
        self.reverse_rows();
        let results = (0..BOARD_SIZE)
            .map(|row| self.slide_row(row))
            .collect::<Vec<_>>();
        self.reverse_rows();
        self.transpose();
        results
    }

    pub fn move_left(&mut self) -> Vec<MergeResult> {
        (0..BOARD_SIZE)
            .map(|row| self.slide_row(row))
            .collect::<Vec<_>>()
    }

    pub fn move_right(&mut self) -> Vec<MergeResult> {
        self.reverse_rows();
        let results = (0..BOARD_SIZE)
            .map(|row| self.slide_row(row))
            .collect::<Vec<_>>();
        self.reverse_rows();
        results
    }

    pub fn spawn_tile(&mut self) {
        let mut rng = rand::rng();
        if let Some((row, col)) = self.random_empty_position(&mut rng) {
            let value = self.generate_tile_value(&mut rng);
            self.add_tile(row, col, value);
        }
    }

    // ========================================================================
    // Board State Queries
    // ========================================================================

    pub fn is_full(&self) -> bool {
        self.empty_tiles().next().is_none()
    }

    pub fn is_game_over(&self) -> bool {
        self.is_full() && !self.has_adjacent_matches()
    }

    // ========================================================================
    // Iterators & Accessors
    // ========================================================================

    pub fn empty_tiles(&self) -> impl Iterator<Item = (usize, usize)> {
        self.iter_cells()
            .filter_map(|(pos, value)| value.is_none().then_some(pos))
    }

    pub fn iter_tiles(&self) -> impl Iterator<Item = Option<u32>> {
        self.iter_cells().map(|(_, val)| val)
    }

    // ========================================================================
    // PRIVATE METHODS
    // ========================================================================

    // ========================================================================
    // Initialization Helpers
    // ========================================================================

    fn initialize(&mut self) {
        self.spawn_tile();
        self.spawn_tile();
    }

    // ========================================================================
    // Tile Management
    // ========================================================================

    fn add_tile(&mut self, row: usize, col: usize, value: u32) {
        self.tiles[row][col] = Some(value);
    }

    fn random_empty_position(
        &self,
        rng: &mut ThreadRng,
    ) -> Option<(usize, usize)> {
        let empty: Vec<_> = self.empty_tiles().collect();
        if let Some((row, col)) = empty.choose(rng) {
            return Some((*row, *col));
        };
        None
    }

    fn generate_tile_value(&self, rng: &mut ThreadRng) -> u32 {
        if rng.random_bool(SPAWN_RATE_2) {
            TILE_VALUE_2
        } else {
            TILE_VALUE_4
        }
    }

    // ========================================================================
    // Board State Helpers
    // ========================================================================

    fn has_adjacent_matches(&self) -> bool {
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if let Some(current) = self.tiles[row][col] {
                    // check right
                    if col < BOARD_SIZE - 1
                        && self.tiles[row][col + 1] == Some(current)
                    {
                        return true;
                    }
                    // check down
                    if row < BOARD_SIZE - 1
                        && self.tiles[row + 1][col] == Some(current)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    // ========================================================================
    // Slide & Merge Logic
    // ========================================================================

    fn merge_cells(values: &[u32]) -> MergeResult {
        // Merge adjacent matching values into a new vector
        let mut merged_tiles = Vec::new();
        // Capture the merged values for scoring
        let mut merged_values = Vec::new();

        let mut iter = values.iter().copied().peekable();
        while let Some(val) = iter.next() {
            if iter.peek() == Some(&val) {
                merged_values.push(val);
                merged_tiles.push(val * 2);
                // Skip the matched value
                iter.next();
            } else {
                merged_tiles.push(val);
            }
        }
        MergeResult::new(merged_tiles, merged_values)
    }

    fn transpose(&mut self) {
        for i in 0..BOARD_SIZE {
            for j in (i + 1)..BOARD_SIZE {
                let temp = self.tiles[i][j];
                self.tiles[i][j] = self.tiles[j][i];
                self.tiles[j][i] = temp;
            }
        }
    }

    fn reverse_rows(&mut self) {
        for row in self.tiles.iter_mut() {
            row.reverse();
        }
    }

    fn slide_row(&mut self, row: usize) -> MergeResult {
        // Extract the row data using the helper
        let values: Vec<_> = self.extract_row(row).collect();

        // Early exit if no values
        if values.is_empty() {
            return MergeResult::default();
        }

        // Capture the original state of the row
        let original = self.tiles[row];

        // Merge cells
        let mut merged = Board::merge_cells(&values);

        // Write back
        for (col, &value) in merged.merged_tiles.iter().enumerate() {
            self.tiles[row][col] = Some(value);
        }

        // Clear remaining
        for col in merged.merged_tiles.len()..self.tiles.len() {
            self.tiles[row][col] = None;
        }

        // Check if anything changed
        merged.board_changed = self.tiles[row]
            .iter()
            .zip(original.iter())
            .any(|(&current, &original)| current != original);
        merged
    }

    // ========================================================================
    // Data Extraction Helpers
    // ========================================================================

    fn iter_cells(
        &self,
    ) -> impl Iterator<Item = ((usize, usize), Option<u32>)> {
        self.tiles.iter().enumerate().flat_map(|(row_idx, row)| {
            row.iter()
                .enumerate()
                .map(move |(col_idx, &cell)| ((row_idx, col_idx), cell))
        })
    }

    fn extract_row(&self, row: usize) -> impl Iterator<Item = u32> {
        self.tiles[row].iter().filter_map(|&val| val)
    }

    #[cfg(test)]
    pub fn set_tiles(
        &mut self,
        tiles: [[Option<u32>; BOARD_SIZE]; BOARD_SIZE],
    ) {
        self.tiles = tiles;
    }

    #[cfg(test)]
    pub fn get_tiles(&self) -> [[Option<u32>; BOARD_SIZE]; BOARD_SIZE] {
        self.tiles
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slide_left_basic() {
        let mut board = Board::new();
        board.set_tiles([
            [Some(2), None, None, None],
            [None, Some(2), None, None],
            [None, None, Some(2), None],
            [None, None, None, Some(2)],
        ]);

        // Consume the iterator
        for _ in board.move_left() {}

        let tiles = board.get_tiles();
        assert_eq!(tiles[0][0], Some(2));
        assert_eq!(tiles[1][0], Some(2));
        assert_eq!(tiles[2][0], Some(2));
        assert_eq!(tiles[3][0], Some(2));
    }

    #[test]
    fn test_merge_left() {
        let mut board = Board::new();
        board.set_tiles([
            [Some(2), Some(2), None, None],
            [Some(4), Some(4), Some(4), Some(4)],
            [Some(2), Some(2), Some(2), Some(2)],
            [Some(2), Some(4), Some(8), Some(16)],
        ]);

        for _ in board.move_left() {}

        let tiles = board.get_tiles();
        // Row 0: 2 2 -> 4
        assert_eq!(tiles[0], [Some(4), None, None, None]);
        // Row 1: 4 4 4 4 -> 8 8
        assert_eq!(tiles[1], [Some(8), Some(8), None, None]);
        // Row 2: 2 2 2 2 -> 4 4
        assert_eq!(tiles[2], [Some(4), Some(4), None, None]);
        // Row 3: No change
        assert_eq!(tiles[3], [Some(2), Some(4), Some(8), Some(16)]);
    }

    #[test]
    fn test_merge_right() {
        let mut board = Board::new();
        board.set_tiles([
            [Some(2), Some(2), None, None],
            [None, None, Some(2), Some(2)],
            [Some(2), Some(4), Some(2), Some(4)],
            [Some(2), Some(2), Some(2), None],
        ]);

        for _ in board.move_right() {}

        let tiles = board.get_tiles();
        assert_eq!(tiles[0], [None, None, None, Some(4)]);
        assert_eq!(tiles[1], [None, None, None, Some(4)]);
        assert_eq!(tiles[2], [Some(2), Some(4), Some(2), Some(4)]);
        assert_eq!(tiles[3], [None, None, Some(2), Some(4)]);
    }

    #[test]
    fn test_move_up() {
        let mut board = Board::new();
        board.set_tiles([
            [Some(2), None, Some(2), Some(2)],
            [Some(2), None, Some(2), Some(4)],
            [Some(4), None, Some(4), Some(8)],
            [Some(4), None, Some(4), Some(16)],
        ]);

        for _ in board.move_up() {}

        let tiles = board.get_tiles();
        // Col 0: 2,2,4,4 -> 4,8
        assert_eq!(tiles[0][0], Some(4));
        assert_eq!(tiles[1][0], Some(8));
        assert_eq!(tiles[2][0], None);
        // Col 2: 2,2,4,4 -> 4,8
        assert_eq!(tiles[0][2], Some(4));
        assert_eq!(tiles[1][2], Some(8));
        // Col 3: 2,4,8,16 -> No change
        assert_eq!(tiles[0][3], Some(2));
        assert_eq!(tiles[1][3], Some(4));
        assert_eq!(tiles[2][3], Some(8));
        assert_eq!(tiles[3][3], Some(16));
    }

    #[test]
    fn test_merge_priority() {
        // Test [2, 2, 2, 0] -> [4, 2, 0, 0] when moving left
        let mut board = Board::new();
        board.set_tiles([
            [Some(2), Some(2), Some(2), None],
            [None; 4],
            [None; 4],
            [None; 4],
        ]);

        for _ in board.move_left() {}

        let tiles = board.get_tiles();
        assert_eq!(tiles[0], [Some(4), Some(2), None, None]);
    }

    #[test]
    fn test_gap_merge() {
        // Test [2, 0, 2, 0] -> [4, 0, 0, 0] when moving left
        let mut board = Board::new();
        board.set_tiles([
            [Some(2), None, Some(2), None],
            [None; 4],
            [None; 4],
            [None; 4],
        ]);

        for _ in board.move_left() {}

        let tiles = board.get_tiles();
        assert_eq!(tiles[0], [Some(4), None, None, None]);
    }

    #[test]
    fn test_no_change_locked_board() {
        // A full board with no possible moves
        let mut board = Board::new();
        board.set_tiles([
            [Some(2), Some(4), Some(8), Some(16)],
            [Some(32), Some(64), Some(128), Some(256)],
            [Some(2), Some(4), Some(8), Some(16)],
            [Some(32), Some(64), Some(128), Some(256)],
        ]);

        let results: Vec<_> = board.move_left();
        // None of the rows should report a change
        assert!(results.iter().all(|r| !r.board_changed));
    }

    #[test]
    fn test_game_over_conditions() {
        let mut board = Board::new();

        // 1. Empty board -> Not game over
        board.set_tiles([[None; 4]; 4]);
        assert!(!board.is_game_over());

        // 2. Full board, possible matches -> Not game over
        board.set_tiles([
            [Some(2), Some(2), Some(4), Some(8)],
            [Some(16), Some(32), Some(64), Some(128)],
            [Some(2), Some(4), Some(8), Some(16)],
            [Some(32), Some(64), Some(128), Some(256)],
        ]);
        assert!(!board.is_game_over());

        // 3. Full board, no matches -> Game Over
        board.set_tiles([
            [Some(2), Some(4), Some(8), Some(16)],
            [Some(32), Some(64), Some(128), Some(256)],
            [Some(2), Some(4), Some(8), Some(16)],
            [Some(32), Some(64), Some(128), Some(256)],
        ]);
        assert!(board.is_game_over());
    }

    #[test]
    fn test_move_down() {
        let mut board = Board::new();
        board.set_tiles([
            [Some(2), None, Some(4), Some(16)],
            [Some(2), None, Some(4), Some(8)],
            [Some(4), None, Some(2), Some(4)],
            [Some(4), None, Some(2), Some(2)],
        ]);

        for _ in board.move_down() {}

        let tiles = board.get_tiles();
        // Col 0: 2,2,4,4 -> 4,8 (at bottom)
        assert_eq!(tiles[3][0], Some(8));
        assert_eq!(tiles[2][0], Some(4));
        assert_eq!(tiles[1][0], None);
        // Col 2: 4,4,2,2 -> 8,4 (at bottom)
        assert_eq!(tiles[3][2], Some(4));
        assert_eq!(tiles[2][2], Some(8));
        assert_eq!(tiles[1][2], None);
        // Col 3: 16,8,4,2 -> No change
        assert_eq!(tiles[0][3], Some(16));
        assert_eq!(tiles[1][3], Some(8));
        assert_eq!(tiles[2][3], Some(4));
        assert_eq!(tiles[3][3], Some(2));
    }

    #[test]
    fn test_spawn_tile() {
        let mut board = Board::new();
        // Clear board first to be sure
        board.set_tiles([[None; 4]; 4]);

        // Spawn 1
        board.spawn_tile();
        let count_1 = board.iter_tiles().filter(|t| t.is_some()).count();
        assert_eq!(count_1, 1);

        // Spawn 2
        board.spawn_tile();
        let count_2 = board.iter_tiles().filter(|t| t.is_some()).count();
        assert_eq!(count_2, 2);
    }

    #[test]
    fn test_reset() {
        let mut board = Board::new();
        board.set_tiles([
            [Some(2), Some(4), Some(8), Some(16)],
            [Some(32), Some(64), Some(128), Some(256)],
            [Some(2), Some(4), Some(8), Some(16)],
            [Some(32), Some(64), Some(128), Some(256)],
        ]);

        board.reset();

        let count = board.iter_tiles().filter(|t| t.is_some()).count();
        // Reset should clear and spawn 2 new tiles
        assert_eq!(count, 2);
    }

    #[test]
    fn test_is_full() {
        let mut board = Board::new();

        // Not full
        board.set_tiles([[None; 4]; 4]);
        assert!(!board.is_full());

        // Almost full
        let mut tiles = [[Some(2); 4]; 4];
        tiles[0][0] = None;
        board.set_tiles(tiles);
        assert!(!board.is_full());

        // Full
        tiles[0][0] = Some(2);
        board.set_tiles(tiles);
        assert!(board.is_full());
    }
}
