//! # Pure editor state and behaviour.

mod buffer;
mod cmdline;
mod insert;
mod motion;
mod normal;
mod operator;
mod text;
mod window;

use std::path::PathBuf;

use crossterm::event::{Event, KeyCode, KeyModifiers};

pub use buffer::Buffer;
pub use cmdline::CmdLine;
pub use window::{Cursor, Window};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    OperatorPending(operator::State),
}

pub struct Editor {
    pub mode: Mode,

    pub buffers: Vec<Buffer>, // Invariant: at least one buffer
    pub windows: Vec<Window>, // Invariant: at least one window
    pub winid: usize,

    pub cmdline: CmdLine,
    pub message: Option<Message>,

    pending_count: Option<usize>,
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
            pending_count: None,
            should_quit: false,
        }
    }

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
        self.line_off(0)
    }

    pub fn line_off(&self, offset_y: i32) -> &str {
        let row = self.cursor().y as i32 + offset_y;
        &self.buf().lines[row as usize]
    }

    pub fn line_mut(&mut self) -> &mut String {
        self.line_off_mut(0)
    }

    pub fn line_off_mut(&mut self, offset_y: i32) -> &mut String {
        let row = self.cursor().y as i32 + offset_y;
        &mut self.buf_mut().lines[row as usize]
    }

    pub fn set_message(&mut self, msg: Message) {
        self.message = Some(msg)
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

    fn clear_pending_count(&mut self) {
        self.pending_count = None;
    }

    fn take_pending_count(&mut self) -> usize {
        self.pending_count.take().unwrap_or(1)
    }

    fn push_pending_count_digit(&mut self, n: u32) {
        let count = self.pending_count.unwrap_or(0);
        self.pending_count = Some(count * 10 + n as usize);
    }

    pub fn update(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.should_quit = true;
                    return;
                }
                match self.mode {
                    Mode::Normal => normal::handle(self, key),
                    Mode::Insert => insert::handle(self, key),
                    Mode::Command => cmdline::handle(self, key),
                    Mode::OperatorPending(state) => operator::handle(self, key, state),
                }
            }
            _ => {}
        }
    }
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

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal | Self::OperatorPending(_) => f.write_str("NORMAL"),
            Mode::Insert => f.write_str("INSERT"),
            Mode::Command => f.write_str("COMMAND"),
        }
    }
}
