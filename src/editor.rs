//! # Pure editor state and behaviour.

use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::layout::Vec2;

pub struct Buffer {
    pub lines: Vec<String>,    // Invariant: at least one line
    pub path: Option<PathBuf>, // Invariant: absolute path
    pub is_dirty: bool,
    pub ends_with_newline: bool,
    /// To restore the last cursor and scroll when reentering the buffer.
    pub last_view: Option<(Cursor, Vec2<usize>)>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            path: None,
            is_dirty: false,
            ends_with_newline: false,
            last_view: None,
        }
    }
}

impl Buffer {
    pub fn from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();

        let (lines, ends_with_newline) = match fs::read_to_string(path) {
            Ok(s) => Self::lines_from_str(&s),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (vec![String::new()], false),
            Err(e) => return Err(e),
        };

        Ok(Self {
            lines,
            path: Some(path::absolute(path)?),
            ends_with_newline,
            ..Default::default()
        })
    }

    fn lines_from_str(s: &str) -> (Vec<String>, bool) {
        let ends_with_newline = s.ends_with('\n');
        let mut lines: Vec<_> = s.lines().map(str::to_owned).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        (lines, ends_with_newline)
    }

    pub fn write(&mut self) -> io::Result<()> {
        use io::Write;

        let path = self
            .path
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No file name"))?;

        let f = fs::File::create(path)?;
        let mut writer = io::BufWriter::new(f);

        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                writer.write_all(b"\n")?;
            }
            writer.write_all(line.as_bytes())?;
        }

        if self.ends_with_newline && !(self.lines.len() == 1 && self.lines[0].is_empty()) {
            writer.write_all(b"\n")?;
        }

        writer.flush()?;
        self.is_dirty = false;
        Ok(())
    }

    pub fn smudge(&mut self) {
        self.is_dirty = true;
    }
}

#[derive(Default)]
pub struct Window {
    pub bufid: usize,
    pub cursor: Cursor,
    /// The top-left viewport offset into the buffer.
    pub scroll: Vec2<usize>,
}

impl Window {
    pub fn set_buf(&mut self, bufid: usize, last_view: Option<(Cursor, Vec2<usize>)>) {
        self.bufid = bufid;
        if let Some((cursor, scroll)) = last_view {
            self.cursor = cursor;
            self.scroll = scroll;
        } else {
            self.cursor = Cursor::default();
            self.scroll = Vec2::default();
        }
    }

    pub fn sync_view(&mut self, view_h: u16) {
        let h = view_h as usize;

        if self.cursor.y < self.scroll.y {
            self.scroll.y = self.cursor.y;
        } else if self.cursor.y >= self.scroll.y + h {
            self.scroll.y = self.cursor.y + 1 - h;
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    /// Preferred column for vertical movement.
    pub want_x: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

#[derive(Default)]
pub struct CmdLine {
    pub buf: String,
    pub cursor: usize,
}

#[derive(Clone, Copy)]
pub enum MessageKind {
    Info,
    Error,
}

pub struct Message {
    pub kind: MessageKind,
    pub text: String,
}

impl Message {
    fn info(text: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Info,
            text: text.into(),
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Error,
            text: text.into(),
        }
    }

    fn from_error(e: impl std::error::Error) -> Self {
        Self::error(e.to_string())
    }
}

pub struct Editor {
    pub buffers: Vec<Buffer>, // Invariant: at least one buffer

    pub windows: Vec<Window>, // Invariant: at least one window
    pub winid: usize,

    pub mode: Mode,

    pub cmdline: CmdLine,

    pub message: Option<Message>,

    pub should_quit: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self::with_buffer(Buffer::default())
    }
}

impl Editor {
    pub fn with_buffer(buffer: Buffer) -> Self {
        Self {
            mode: Mode::Normal,
            buffers: vec![buffer],
            windows: vec![Window::default()],
            winid: 0,
            cmdline: CmdLine::default(),
            message: None,
            should_quit: false,
        }
    }

    pub fn set_message(&mut self, msg: Message) {
        self.message = Some(msg)
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
            Mode::Command => unreachable!("cmdline uses its own cursor semantics"),
        }
    }

    // Move

    pub fn move_to(&mut self, x: usize, y: usize) {
        *self.cursor_mut() = Cursor { x, y, want_x: x };
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
        self.buf_mut().smudge();
    }

    pub fn remove_char(&mut self, offset_x: i32) -> char {
        let col = self.cursor().x as i32 + offset_x;
        self.buf_mut().smudge();
        self.line_mut().remove(col as usize)
    }

    pub fn insert_line(&mut self, offset_y: i32, content: Option<String>) {
        let row = self.cursor().y as i32 + offset_y;
        self.buf_mut().smudge();
        self.buf_mut()
            .lines
            .insert(row as usize, content.unwrap_or_default());
    }

    pub fn remove_line(&mut self, offset_y: i32) -> String {
        let row = self.cursor().y as i32 + offset_y;
        self.buf_mut().smudge();
        self.buf_mut().lines.remove(row as usize)
    }

    // Normal commands

    pub fn delete_under_cursor(&mut self) {
        if self.line().is_empty() {
            return;
        }

        self.remove_char(0);
        self.cursor_clamp_sync_x();
        self.buf_mut().smudge();
    }

    pub fn delete_to_eol(&mut self) {
        let x = self.cursor().x;

        if x >= self.line().len() {
            return;
        }

        self.line_mut().truncate(x);
        self.cursor_clamp_sync_x();
        self.buf_mut().smudge();
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
        self.buf_mut().smudge();
    }

    pub fn backspace(&mut self) {
        let cur = self.cursor();

        if cur.x > 0 {
            // at mid or eol: remove char
            self.move_left();
            self.remove_char(0);
            self.buf_mut().smudge();
            return;
        }

        if cur.y > 0 {
            // at not eof and bol: join with upper line
            let line = self.remove_line(0);
            let nx = self.line_at(-1).len();
            self.move_to(nx, cur.y - 1);
            self.line_mut().push_str(&line);
            self.buf_mut().smudge();
        }
    }

    // cmdline

    pub fn cmdline_submit(&mut self) {
        match Self::cmdline_parse(self.cmdline.buf.trim()) {
            Ok(None) => {}
            Ok(Some(cmd)) => match cmd {
                Command::Quit => self.should_quit = true,
                Command::Write => {
                    if let Err(e) = self.buf_mut().write() {
                        self.set_message(Message::from_error(e));
                    } else {
                        let path = self.buf().path.as_ref().unwrap().display();
                        let msg = format!("{path} written");
                        self.set_message(Message::info(msg))
                    }
                }
                Command::Edit(path) => {
                    let path = path::absolute(path).unwrap();
                    if let Some(bufid) = self.buf_open(path) {
                        self.buf_leave();
                        let last_view = self.buffers[bufid].last_view;
                        self.win_mut().set_buf(bufid, last_view);
                    }
                }
            },
            Err(e) => self.set_message(Message::from_error(e)),
        }
        self.mode = Mode::Normal;
    }

    fn buf_open(&mut self, path: PathBuf) -> Option<usize> {
        if let Some(bufid) = self
            .buffers
            .iter()
            .position(|b| b.path.as_ref() == Some(&path))
        {
            crate::debug!("Open existing buf: id = {bufid}");
            return Some(bufid);
        }

        match Buffer::from_path(path) {
            Err(e) => {
                self.set_message(Message::from_error(e));
                None
            }
            Ok(buffer) => {
                self.buffers.push(buffer);
                let bufid = self.buffers.len() - 1;
                crate::debug!("Open new buf: id = {bufid}");
                Some(bufid)
            }
        }
    }

    fn buf_leave(&mut self) {
        let win = self.win();
        self.buf_mut().last_view = Some((win.cursor, win.scroll));
    }

    pub fn cmdline_parse(input: &str) -> Result<Option<Command<'_>>, CmdError> {
        let mut args = input.split_whitespace();
        let Some(name) = args.next() else {
            return Ok(None);
        };

        let cmd = match name {
            "q" => Command::Quit,
            "w" => Command::Write,
            "e" => {
                let Some(path) = args.next() else {
                    return Err(CmdError::MissingArgument("path".into()));
                };
                Command::Edit(path)
            }
            _ => return Err(CmdError::NotAnEditorCommand(name.into())),
        };

        Ok(Some(cmd))
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
                    Mode::Command => command(self, key),
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

        // normal -> command
        KeyCode::Char(':') => {
            ed.cmdline.clear();
            ed.message = None;
            ed.mode = Mode::Command;
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

#[derive(thiserror::Error, Debug)]
pub enum CmdError {
    #[error("Not an editor command: {0}")]
    NotAnEditorCommand(String),
    #[error("Missing argument: {0}")]
    MissingArgument(String),
}

pub enum Command<'a> {
    Quit,
    Write,
    Edit(&'a str),
}

impl CmdLine {
    fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn move_right(&mut self) {
        if self.cursor < self.buf.len() {
            self.cursor += 1;
        }
    }

    fn insert_char(&mut self, ch: char) {
        self.buf.insert(self.cursor, ch);
        self.cursor += 1;
    }

    fn remove_char(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        self.cursor -= 1;
        self.buf.remove(self.cursor);
    }

    fn clear(&mut self) {
        self.buf.clear();
        self.cursor = 0;
    }
}

fn command(ed: &mut Editor, key: KeyEvent) {
    match key.code {
        KeyCode::Char(ch) => ed.cmdline.insert_char(ch),
        KeyCode::Left => ed.cmdline.move_left(),
        KeyCode::Right => ed.cmdline.move_right(),

        KeyCode::Backspace => ed.cmdline.remove_char(),

        KeyCode::Enter => {
            crate::debug!("cmdline = {}", ed.cmdline.buf);
            ed.cmdline_submit();
        }

        KeyCode::Esc => ed.mode = Mode::Normal,
        _ => {}
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => f.write_str("NORMAL"),
            Mode::Insert => f.write_str("INSERT"),
            Mode::Command => f.write_str("COMMAND"),
        }
    }
}
