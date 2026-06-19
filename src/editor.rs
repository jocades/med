//! # Pure editor state and behaviour.

use std::path::PathBuf;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};

pub struct Buffer {
    pub lines: Vec<String>,
    pub path: Option<PathBuf>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            path: None,
        }
    }
}

#[derive(Default)]
pub struct Window {
    bufid: usize,
    cursor: Cursor,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    /// For vertical motions
    pub want_x: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
    pub buffers: Vec<Buffer>,
    pub windows: Vec<Window>,
    pub winid: usize,

    pub mode: Mode,
    pub should_quit: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            buffers: vec![Buffer::default()],
            windows: vec![Window::default()],
            winid: 0,
            should_quit: false,
        }
    }

    // Query

    pub fn win(&self) -> &Window {
        &self.windows[self.winid]
    }

    pub fn win_mut(&mut self) -> &mut Window {
        &mut self.windows[self.winid]
    }

    pub fn buf(&self) -> &Buffer {
        let bufid = self.win().bufid;
        &self.buffers[bufid]
    }

    pub fn buf_mut(&mut self) -> &mut Buffer {
        let bufid = self.win().bufid;
        &mut self.buffers[bufid]
    }

    pub fn cursor(&self) -> Cursor {
        self.win().cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.win_mut().cursor
    }

    pub fn line(&self) -> &str {
        self.line_at(0)
    }

    pub fn line_at(&self, offset_y: i32) -> &str {
        let row = self.cursor().y as i32 + offset_y;
        &self.buf().lines[row as usize]
    }

    pub fn line_mut(&mut self) -> &mut String {
        self.line_at_mut(0)
    }

    pub fn line_at_mut(&mut self, offset_y: i32) -> &mut String {
        let row = self.cursor().y as i32 + offset_y;
        &mut self.buf_mut().lines[row as usize]
    }

    pub fn cursor_max_x(&self) -> usize {
        let len = self.line().len();
        match self.mode {
            Mode::Insert => len,
            Mode::Normal => len.saturating_sub(1),
        }
    }

    // Move

    pub fn move_to(&mut self, x: usize, y: usize) {
        let cur = self.cursor_mut();
        *cur = Cursor { x, y, want_x: x };
    }

    pub fn cursor_clamp_x(&mut self) {
        let max = self.cursor_max_x();
        let cur = self.cursor_mut();
        cur.x = cur.x.min(max);
    }

    fn cursor_clamp_sync_x(&mut self) {
        self.cursor_clamp_x();
        let cur = self.cursor_mut();
        cur.want_x = cur.x;
    }

    pub fn move_left(&mut self) {
        let cur = self.cursor_mut();
        cur.x = cur.x.saturating_sub(1);
        cur.want_x = cur.x;
    }

    pub fn move_right(&mut self) {
        let max = self.cursor_max_x();
        let cur = self.cursor_mut();
        if cur.x < max {
            cur.x += 1;
            cur.want_x = cur.x;
        }
    }

    pub fn move_up(&mut self) {
        let cur = self.cursor_mut();
        cur.y = cur.y.saturating_sub(1);
        cur.x = cur.want_x;
        self.cursor_clamp_x();
    }

    pub fn move_down(&mut self) {
        let max = self.buf().lines.len() - 1;
        let cur = self.cursor_mut();
        if cur.y < max {
            cur.y += 1;
            cur.x = cur.want_x;
            self.cursor_clamp_x();
        }
    }

    pub fn move_bol(&mut self) {
        let cur = self.cursor_mut();
        cur.x = 0;
        cur.want_x = cur.x;
    }

    pub fn move_eol(&mut self) {
        let max = self.cursor_max_x();
        let cur = self.cursor_mut();
        cur.x = max;
        cur.want_x = cur.x;
    }

    // Edit

    pub fn insert_char(&mut self, ch: char) {
        let col = self.cursor().x;
        self.line_mut().insert(col, ch);
    }

    pub fn remove_char(&mut self, offset_x: i32) -> char {
        let col = self.cursor().x as i32 + offset_x;
        self.line_mut().remove(col as usize)
    }

    pub fn insert_line(&mut self, offset_y: i32, content: Option<String>) {
        let row = self.cursor().y as i32 + offset_y;
        self.buf_mut()
            .lines
            .insert(row as usize, content.unwrap_or_default());
    }

    pub fn remove_line(&mut self, offset_y: i32) -> String {
        let row = self.cursor().y as i32 + offset_y;
        self.buf_mut().lines.remove(row as usize)
    }

    // Normal commands

    pub fn delete_under_cursor(&mut self) {
        if self.line().is_empty() {
            return;
        }

        self.remove_char(0);
        self.cursor_clamp_sync_x();
    }

    pub fn delete_to_eol(&mut self) {
        let x = self.cursor().x;

        if x >= self.line().len() {
            return;
        }

        self.line_mut().truncate(x);
        self.cursor_clamp_sync_x();
    }

    // Insert commands

    pub fn enter(&mut self) {
        let cur = self.cursor();

        if cur.x == self.line().len() {
            // at end: add new empty line
            self.insert_line(1, None);
        } else {
            // at mid: split
            let rhs = self.line()[cur.x..].to_owned();
            self.line_mut().truncate(cur.x);
            self.insert_line(1, Some(rhs))
        }

        self.move_to(0, cur.y + 1);
    }

    pub fn backspace(&mut self) {
        let cur = self.cursor();

        if cur.x > 0 {
            // at mid or eol: remove char
            self.move_left();
            self.remove_char(0);
            return;
        }

        if cur.y > 0 {
            // at not eof and bol: join with upper line
            let line = self.remove_line(0);
            let nx = self.line_at(-1).len();
            self.move_to(nx, cur.y - 1);
            self.line_mut().push_str(&line);
        }
    }

    pub fn update(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.should_quit = true;
                    return;
                }
                match self.mode {
                    Mode::Normal => normal(self, key),
                    Mode::Insert => insert(self, key),
                }
            }
            Event::Mouse(ev) => {
                if ev.kind == MouseEventKind::Down(MouseButton::Left) {
                    self.move_to(ev.column as usize, ev.row as usize);
                }
            }
            _ => {}
        }
    }
}

fn normal(ed: &mut Editor, key: KeyEvent) {
    match key.code {
        // normal -> insert
        KeyCode::Char('i') => ed.mode = Mode::Insert,
        KeyCode::Char('a') => {
            ed.mode = Mode::Insert;
            ed.move_right();
        }
        KeyCode::Char('I') => {
            ed.mode = Mode::Insert;
            ed.move_bol();
        }
        KeyCode::Char('A') => {
            ed.mode = Mode::Insert;
            ed.move_eol();
        }
        KeyCode::Char('o') => {
            ed.insert_line(1, None);
            ed.mode = Mode::Insert;
            ed.cursor_mut().y += 1;
            ed.move_bol();
        }
        KeyCode::Char('O') => {
            ed.insert_line(0, None);
            ed.mode = Mode::Insert;
            ed.move_bol();
        }

        // movement
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => ed.move_left(),
        KeyCode::Char('l') | KeyCode::Right => ed.move_right(),
        KeyCode::Char('k') | KeyCode::Up => ed.move_up(),
        KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => ed.move_down(),

        KeyCode::Char('$') => {
            ed.move_eol();
        }
        KeyCode::Char('0') => {
            ed.move_bol();
        }

        // edit
        KeyCode::Char('x') => ed.delete_under_cursor(),
        KeyCode::Char('D') => ed.delete_to_eol(),
        KeyCode::Char('C') => {
            ed.mode = Mode::Insert;
            ed.delete_to_eol();
        }

        _ => {}
    }
}

pub fn insert(ed: &mut Editor, key: KeyEvent) {
    match key.code {
        // insert -> normal
        KeyCode::Esc => {
            ed.mode = Mode::Normal;
            ed.move_left();
        }

        // movement
        KeyCode::Left => ed.move_left(),
        KeyCode::Right => ed.move_right(),
        KeyCode::Up => ed.move_up(),
        KeyCode::Down => ed.move_down(),

        // edit
        KeyCode::Char(ch) => {
            ed.insert_char(ch);
            let cur = ed.cursor_mut();
            cur.x += 1;
            cur.want_x = cur.x;
        }

        KeyCode::Enter => ed.enter(),
        KeyCode::Backspace => ed.backspace(),
        KeyCode::Tab => {
            ed.insert_char(' ');
            ed.insert_char(' ');
            let cur = ed.cursor_mut();
            cur.x += 2;
            cur.want_x = cur.x;
        }

        _ => {}
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => f.write_str("NORMAL"),
            Mode::Insert => f.write_str("INSERT"),
        }
    }
}
