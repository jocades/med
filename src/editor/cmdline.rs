use std::path;

use crossterm::event::{KeyCode, KeyEvent};

use super::{Editor, Message, Mode};

pub fn handle(ed: &mut Editor, key: KeyEvent) {
    match key.code {
        KeyCode::Char(ch) => ed.cmdline.insert_char(ch),
        KeyCode::Left => ed.cmdline.move_left(),
        KeyCode::Right => ed.cmdline.move_right(),
        KeyCode::Backspace => ed.cmdline.remove_char(),
        KeyCode::Esc => ed.mode = Mode::Normal,
        KeyCode::Enter => submit(ed),
        _ => {}
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not an editor command: {0}")]
    NotAnEditorCommand(String),
    #[error("Missing argument: {0}")]
    MissingArgument(&'static str),
}

#[derive(Default)]
pub struct CmdLine {
    pub buf: String,
    pub cursor: usize,
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

    pub fn clear(&mut self) {
        self.buf.clear();
        self.cursor = 0;
    }
}

pub enum Command {
    Quit,
    Write(Option<String>),
    Edit(String),
}

pub fn parse(input: &str) -> Result<Option<Command>, Error> {
    let mut args = input.split_whitespace();

    let Some(name) = args.next() else {
        return Ok(None);
    };

    let cmd = match name {
        "q" => Command::Quit,
        "w" => {
            if let Some(path) = args.next() {
                Command::Write(Some(path.into()))
            } else {
                Command::Write(None)
            }
        }
        "e" => {
            let Some(path) = args.next() else {
                return Err(Error::MissingArgument("path"));
            };
            Command::Edit(path.into())
        }
        _ => return Err(Error::NotAnEditorCommand(name.into())),
    };

    Ok(Some(cmd))
}

fn submit(ed: &mut Editor) {
    match parse(ed.cmdline.buf.trim()) {
        Ok(None) => {}
        Ok(Some(cmd)) => execute(ed, cmd),
        Err(e) => ed.set_message(Message::from_error(e)),
    }
    ed.mode = Mode::Normal;
}

fn execute(ed: &mut Editor, cmd: Command) {
    match cmd {
        Command::Quit => ed.should_quit = true,
        Command::Write(dest) => {
            if let Some(path) = dest {
                let path = path::absolute(path).unwrap();
                ed.buf_mut().path = Some(path);
            }

            match ed.buf_mut().write() {
                Err(e) => ed.set_message(Message::from_error(e)),
                Ok((lines, bytes)) => {
                    let path = ed.buf().path.as_ref().unwrap().display();
                    let info = format!("\"{path}\" {lines}L, {bytes}B written");
                    ed.set_message(Message::info(info))
                }
            }
        }
        Command::Edit(path) => {
            let path = path::absolute(path).unwrap();
            if let Some(bufid) = ed.buf_open(path) {
                ed.buf_leave();
                let last_view = ed.buffers[bufid].last_view;
                ed.win_mut().set_buf(bufid, last_view);
            }
        }
    }
}
