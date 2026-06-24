#![allow(unused)]
use crossterm::{
    Command,
    style::{Color, Print},
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Cell {
    ch: char,
    fg: Color,
    bg: Color,
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
struct Buffer {
    cells: Vec<Cell>,
    w: usize,
    h: usize,
}

impl Buffer {
    fn new(w: usize, h: usize) -> Self {
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

    fn get(&self, x: usize, y: usize) -> &Cell {
        let index = self.index_of(x, y);
        &self.cells[index]
    }

    fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        let index = self.index_of(x, y);
        &mut self.cells[index]
    }

    fn put(&mut self, x: usize, y: usize, ch: char) {
        self.put_styled(x, y, ch, Color::Reset, Color::Reset);
    }

    fn put_styled(&mut self, x: usize, y: usize, ch: char, fg: Color, bg: Color) {
        let index = self.index_of(x, y);
        if let Some(cell) = self.cells.get_mut(index) {
            *cell = Cell { ch, fg, bg };
        }
    }

    fn put_str(&mut self, x: usize, y: usize, s: &str) {
        self.put_str_styled(x, y, s, Color::White, Color::Black);
    }

    fn put_str_styled(&mut self, x: usize, y: usize, s: &str, fg: Color, bg: Color) {
        let start = self.index_of(x, y);
        // TODO: use char_indices when handling utf8
        for (offset, ch) in s.chars().enumerate() {
            if let Some(cell) = self.cells.get_mut(start + offset) {
                *cell = Cell { ch, fg, bg };
            }
        }
    }

    fn diff(&self, other: &Self) -> Vec<(usize, usize, Cell)> {
        assert_eq!(self.w, other.w);
        assert_eq!(self.h, other.h);

        // let mut changes = Vec::new();
        //
        // for (i, (a, b)) in self.cells.iter().zip(&other.cells).enumerate() {
        //     if a != b {
        //         let (x, y) = self.coord_of(i);
        //         changes.push((x, y, *b));
        //     }
        // }
        //
        // changes

        self.cells
            .iter()
            .zip(&other.cells)
            .enumerate()
            .filter_map(|(i, (a, b))| {
                (a != b).then(|| {
                    let (x, y) = self.coord_of(i);
                    (x, y, *b)
                })
            })
            .collect()

        // self.cells
        //     .iter()
        //     .zip(&other.cells)
        //     .enumerate()
        //     .filter(|(_, (a, b))| a != b)
        //     .map(|(i, (_, &cell))| {
        //         let (x, y) = self.coord_of(i);
        //         (x, y, cell)
        //     })
        //     .collect()
    }

    fn iter_coords(&self) -> CoordsIter<'_> {
        CoordsIter { idx: 0, buf: self }
    }
}

struct CoordsIter<'a> {
    idx: usize,
    buf: &'a Buffer,
}

impl<'a> Iterator for CoordsIter<'a> {
    type Item = ((usize, usize), &'a Cell);

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.cells.get(self.idx).map(|cell| {
            let item = (self.buf.coord_of(self.idx), cell);
            self.idx += 1;
            item
        })
    }
}

#[derive(Debug)]
enum MockCommand {
    MoveTo(usize, usize),
    SetBackground(Color),
    SetForeground(Color),
    Print(char),
}

fn main() {
    let mut buffer = Buffer::new(2, 2);

    let mut prev = buffer.clone();

    buffer.put_str(0, 0, "abc");
    buffer.put_styled(1, 1, 'd', Color::Blue, Color::Red);

    let changes = dbg!(prev.diff(&buffer));

    let mut mock_commands = Vec::new();

    let prev_x = 0;
    let prev_y = 0;
    let prev_fg = Color::Reset;
    let prev_bg = Color::Reset;

    for (x, y, Cell { ch, fg, bg }) in changes {
        if x != prev_x + 1 || y != prev_y {
            mock_commands.push(MockCommand::MoveTo(x, y));
        }

        if fg != prev_fg {
            mock_commands.push(MockCommand::SetForeground(fg));
        }

        if bg != prev_bg {
            mock_commands.push(MockCommand::SetBackground(bg));
        }

        mock_commands.push(MockCommand::Print(ch));
    }

    std::mem::swap(&mut buffer, &mut prev);

    dbg!(mock_commands);

    // for ((x, y), cell) in buffer.iter_coords() {
    //     println!("{x},{y}: {cell:?}");
    // }
}
