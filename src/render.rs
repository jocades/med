//! # Turns editor state + layout into terminal drawing commands.

use std::io::{self, StdoutLock, Write};

use crossterm::{cursor::*, queue, style::*, terminal::*};

use crate::editor::{Editor, Mode};
use crate::layout::{Layout, Rect, Split};

pub fn render(ed: &Editor, layout: &Layout, stdout: &mut StdoutLock<'static>) -> io::Result<()> {
    queue!(stdout, Clear(ClearType::All))?;
    status(ed, stdout, layout.status)?;
    gutter(ed, stdout, layout.gutter)?;
    buffer(ed, stdout, layout.buffer)?;
    cursor(ed, stdout, layout.buffer)?;
    stdout.flush()?;
    Ok(())
}

pub trait ToColor {
    fn to_color(&self) -> Color;
}

impl ToColor for u32 {
    fn to_color(&self) -> Color {
        Color::Rgb {
            r: (self >> 16) as u8 & 0xff,
            g: (self >> 8) as u8 & 0xff,
            b: *self as u8 & 0xff,
        }
    }
}

pub trait TryToColor {
    type Error;

    fn try_to_color(&self) -> Result<Color, Self::Error>;
}

impl TryToColor for &str {
    type Error = std::num::ParseIntError;

    fn try_to_color(&self) -> Result<Color, Self::Error> {
        u32::from_str_radix(self, 16).map(|n| n.to_color())
    }
}

fn gutter(ed: &Editor, stdout: &mut StdoutLock<'static>, rect: Rect) -> io::Result<()> {
    queue!(stdout, SetForegroundColor(Color::DarkGrey))?;

    let buf = ed.buf();
    let scroll = ed.win().scroll;

    for y in 0..rect.h as usize {
        if buf.lines.get(y).is_some() {
            let line_no = (scroll.y + y + 1).to_string();
            queue!(
                stdout,
                MoveTo(rect.x + 4 - line_no.len() as u16, rect.y + y as u16),
                Print(line_no),
            )?;
        } else {
            queue!(stdout, MoveTo(rect.x, rect.y + y as u16), Print("~"))?;
        }
    }

    queue!(stdout, ResetColor)?;
    Ok(())
}

fn buffer(ed: &Editor, stdout: &mut StdoutLock<'static>, rect: Rect) -> io::Result<()> {
    let buf = ed.buf();
    let scroll = ed.win().scroll;

    queue!(
        stdout,
        // (205, 214, 244)
        SetForegroundColor(0xcdd6f4.to_color()),
    )?;

    for y in 0..rect.h as usize {
        if let Some(line) = buf.lines.get(scroll.y + y) {
            let len = line.len().min(rect.w as usize);
            queue!(
                stdout,
                MoveTo(rect.x, rect.y + y as u16),
                Print(&line[..len])
            )?;
        }
    }
    Ok(())
}

fn status(ed: &Editor, stdout: &mut StdoutLock<'static>, rect: Rect) -> io::Result<()> {
    let mode = format!(" {} ", ed.mode);
    let path = ed
        .buf()
        .path
        .as_ref()
        .map(|p| p.to_str().unwrap())
        .unwrap_or("[No Name]");

    let cur = ed.cursor();
    let pos = format!(" {},{} ", cur.y, cur.x);
    let pad = rect.w as usize - mode.len() - path.len();

    queue!(
        stdout,
        MoveTo(rect.x, rect.y),
        SetBackgroundColor(Color::Red),
        Print(mode.black().on_green()),
        ResetColor,
        Print(path),
        Print(format!("{pos:>pad$}")),
        ResetColor,
    )?;

    Ok(())
}

fn cursor(ed: &Editor, stdout: &mut StdoutLock<'static>, rect: Rect) -> io::Result<()> {
    let cursor = ed.cursor();
    let scroll = ed.win().scroll;

    let style = match ed.mode {
        Mode::Insert => SetCursorStyle::SteadyBar,
        Mode::Normal => SetCursorStyle::SteadyBlock,
    };

    let x = rect.x + (cursor.x - scroll.x) as u16;
    let y = rect.y + (cursor.y - scroll.y) as u16;

    queue!(stdout, style, MoveTo(x, y))?;
    Ok(())
}

fn render_command(_ed: &Editor, stdout: &mut StdoutLock<'static>, rect: Rect) -> io::Result<()> {
    queue!(
        stdout,
        MoveTo(rect.x, rect.y),
        Clear(ClearType::CurrentLine)
    )?;
    Ok(())
}
