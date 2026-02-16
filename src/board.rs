pub(crate) const BOARD_SIZE: usize = 4;

#[derive(Debug, Default)]
pub struct Board {
    cells: [[Option<u32>; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    // Returns an iterator over the board cells and coordinates in row major
    // order in the form ((row, col), value).
    pub fn iter_cells(
        &self,
    ) -> impl Iterator<Item = ((usize, usize), &Option<u32>)> {
        self.cells.iter().enumerate().flat_map(|(row, row_cells)| {
            row_cells
                .iter()
                .enumerate()
                .map(move |(col, col_cell)| ((row, col), col_cell))
        })
    }

    pub fn col(&self, col: usize) -> impl DoubleEndedIterator<Item = u32> {
        self.cells.iter().filter_map(move |row| row[col])
    }

    pub fn row(&self, row: usize) -> impl DoubleEndedIterator<Item = u32> {
        self.cells[row].iter().copied().flatten()
    }

    pub fn cell(&self, row: usize, col: usize) -> Option<u32> {
        self.cells[row][col]
    }

    pub fn cell_mut(&mut self, row: usize, col: usize) -> &mut Option<u32> {
        &mut self.cells[row][col]
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

    #[test]
    fn iter_cells_is_row_major_with_coordinates() {
        let board = board_from_rows([
            [Some(2), None, Some(4), None],
            [None, Some(8), None, None],
            [Some(16), None, None, Some(32)],
            [None, None, None, Some(64)],
        ]);

        let cells: Vec<((usize, usize), Option<u32>)> = board
            .iter_cells()
            .map(|(coord, value)| (coord, *value))
            .collect();

        let expected_coords: Vec<(usize, usize)> = (0..BOARD_SIZE)
            .flat_map(|row| (0..BOARD_SIZE).map(move |col| (row, col)))
            .collect();

        assert_eq!(cells.len(), BOARD_SIZE * BOARD_SIZE);
        assert_eq!(
            cells.iter().map(|(coord, _)| *coord).collect::<Vec<_>>(),
            expected_coords
        );
        assert_eq!(cells[0], ((0, 0), Some(2)));
        assert_eq!(cells[5], ((1, 1), Some(8)));
        assert_eq!(cells[15], ((3, 3), Some(64)));
    }

    #[test]
    fn col_filters_empty_values_and_can_reverse() {
        let board = board_from_rows([
            [Some(2), None, None, None],
            [None, None, None, None],
            [Some(4), None, None, None],
            [Some(8), None, None, None],
        ]);

        assert_eq!(board.col(0).collect::<Vec<_>>(), vec![2, 4, 8]);
        assert_eq!(board.col(0).rev().collect::<Vec<_>>(), vec![8, 4, 2]);
    }

    #[test]
    fn row_filters_empty_values_and_can_reverse() {
        let board = board_from_rows([
            [None, None, None, None],
            [None, None, None, None],
            [None, Some(4), None, Some(8)],
            [None, None, None, None],
        ]);

        assert_eq!(board.row(2).collect::<Vec<_>>(), vec![4, 8]);
        assert_eq!(board.row(2).rev().collect::<Vec<_>>(), vec![8, 4]);
    }

    #[test]
    fn cell_and_cell_mut_round_trip() {
        let mut board = Board::default();

        assert_eq!(board.cell(1, 2), None);
        *board.cell_mut(1, 2) = Some(32);
        assert_eq!(board.cell(1, 2), Some(32));
        *board.cell_mut(1, 2) = None;
        assert_eq!(board.cell(1, 2), None);
    }
}
