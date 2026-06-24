use crossterm::{
    Command,
    style::{Color, Print},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub cells: Vec<Cell>,
    pub w: usize,
    pub h: usize,
}

impl Buffer {
    pub fn new(w: usize, h: usize) -> Self {
        let cells = vec![Cell::default(); w * h];
        Self { cells, w, h }
    }

    #[inline]
    fn index_of(&self, x: usize, y: usize) -> usize {
        self.w * y + x
    }

    #[inline]
    fn coord_of(&self, index: usize) -> (usize, usize) {
        (index % self.w, index / self.w)
    }

    pub fn get(&self, x: usize, y: usize) -> &Cell {
        let index = self.index_of(x, y);
        &self.cells[index]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        let index = self.index_of(x, y);
        &mut self.cells[index]
    }

    pub fn put(&mut self, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
        let index = self.index_of(x, y);
        if let Some(cell) = self.cells.get_mut(index) {
            *cell = Cell { ch, fg, bg };
        }
    }

    pub fn put_str(&mut self, x: usize, y: usize, s: &str, fg: Color, bg: Color) {
        let start = self.index_of(x, y);

        // TODO: use char_indices when handling utf8
        for (offset, ch) in s.chars().enumerate() {
            if let Some(cell) = self.cells.get_mut(start + offset) {
                *cell = Cell { ch, fg, bg };
            }
        }
    }

    pub fn diff(&self, other: &Self) -> Vec<(usize, usize, Cell)> {
        assert_eq!(self.w, other.w);
        assert_eq!(self.h, other.h);

        let mut changes = Vec::new();

        for (i, (a, b)) in self.cells.iter().zip(&other.cells).enumerate() {
            if a != b {
                let (x, y) = self.coord_of(i);
                changes.push((x, y, *b));
            }
        }

        changes
    }
}
