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

    // Sets the value of the cell at the given coordinates.
    pub fn set_cell(&mut self, row: usize, col: usize, value: u32) {
        self.cells[row][col] = Some(value);
    }
}
