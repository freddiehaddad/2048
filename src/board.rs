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
