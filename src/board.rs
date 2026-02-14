pub(crate) const BOARD_SIZE: usize = 4;

#[derive(Debug, Default)]
pub struct Board {
    cells: [[Option<u32>; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    // Returns an iterator over the board cells in row major order
    pub fn iter_cells(&self) -> impl Iterator<Item = &Option<u32>> {
        self.cells.iter().flat_map(|v| v.iter())
    }
}
