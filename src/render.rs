//! # Turns editor state + layout into terminal drawing commands.

use std::io::{self, StdoutLock, Write};

use crossterm::{cursor::*, queue, style::*, terminal::*};

use crate::editor::{Editor, MessageKind, Mode};
use crate::layout::{Layout, Rect};

pub fn render(ed: &Editor, layout: &Layout, stdout: &mut StdoutLock<'static>) -> io::Result<()> {
    queue!(stdout, Clear(ClearType::All))?;
    status(ed, layout.status, stdout)?;
    gutter(ed, layout.gutter, stdout)?;
    buffer(ed, layout.buffer, stdout)?;
    cmdline(ed, layout.cmdline, stdout)?;
    cursor(ed, layout, stdout)?;
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

fn gutter(ed: &Editor, rect: Rect, stdout: &mut StdoutLock<'static>) -> io::Result<()> {
    queue!(stdout, SetForegroundColor(Color::DarkGrey))?;

    let buf = ed.buf();
    let scroll = ed.win().scroll;

    for row in 0..rect.h as usize {
        queue!(stdout, MoveTo(rect.x, rect.y + row as u16))?;
        let y = scroll.y + row;
        if buf.lines.get(y).is_some() {
            let line_no = format!("{:>4}", y + 1);
            queue!(stdout, Print(line_no))?;
        } else {
            queue!(stdout, Print("~"))?;
        }
    }

    queue!(stdout, ResetColor)?;
    Ok(())
}

const FG: Color = Color::Rgb {
    r: 205,
    g: 214,
    b: 244,
};

fn buffer(ed: &Editor, rect: Rect, stdout: &mut StdoutLock<'static>) -> io::Result<()> {
    let buf = ed.buf();
    let scroll = ed.win().scroll;

    queue!(
        stdout,
        // (205, 214, 244)
        // SetForegroundColor(0xcdd6f4.to_color()),
        SetForegroundColor(FG),
    )?;

    for row in 0..rect.h as usize {
        if let Some(line) = buf.lines.get(scroll.y + row) {
            let len = line.len().min(rect.w as usize);
            queue!(
                stdout,
                MoveTo(rect.x, rect.y + row as u16),
                Print(&line[..len])
            )?;
        }
    }
    Ok(())
}

fn status(ed: &Editor, rect: Rect, stdout: &mut StdoutLock<'static>) -> io::Result<()> {
    let mode = format!(" {} ", ed.mode);
    let path = ed
        .buf()
        .path
        .as_ref()
        .map(|p| p.to_str().unwrap())
        .unwrap_or("[No Name]");
    let mut path = format!(" {path}");
    if ed.buf().is_dirty {
        path.push_str(" [+]")
    }

    let pad = rect.w as usize - mode.len() - path.len();
    let cur = ed.cursor();
    let pos = format!(
        " {},{}  {:.0}% ",
        cur.y + 1,
        cur.x + 1,
        (cur.y as f32 / ed.buf().lines.len() as f32) * 100.0
    );

    let bg = Color::Rgb {
        r: 24,
        g: 24,
        b: 37,
    };

    queue!(
        stdout,
        MoveTo(rect.x, rect.y),
        Print(mode.with(bg).on_green()),
        SetColors(Colors::new(FG, bg)),
        Print(path),
        Print(format!("{pos:>pad$}")),
        ResetColor,
    )?;

    Ok(())
}

fn cmdline(ed: &Editor, rect: Rect, stdout: &mut StdoutLock<'static>) -> io::Result<()> {
    if ed.mode == Mode::Command {
        queue!(
            stdout,
            MoveTo(rect.x, rect.y),
            Print(":"),
            Print(&ed.cmdline.buf)
        )?;
    } else if let Some(message) = &ed.message {
        let fg = match message.kind {
            MessageKind::Info => FG,
            MessageKind::Error => Color::Red,
        };
        queue!(
            stdout,
            MoveTo(rect.x, rect.y),
            SetForegroundColor(fg),
            Print(&message.text),
            ResetColor,
        )?;
    } else {
        // clear
    }
    Ok(())
}

fn cursor(ed: &Editor, layout: &Layout, stdout: &mut StdoutLock<'static>) -> io::Result<()> {
    let cursor = ed.cursor();
    let scroll = ed.win().scroll;

    let style = match ed.mode {
        Mode::Normal => SetCursorStyle::SteadyBlock,
        Mode::Insert | Mode::Command => SetCursorStyle::SteadyBar,
    };

    let (x, y) = match ed.mode {
        Mode::Normal | Mode::Insert => {
            let rect = layout.buffer;
            let x = rect.x + (cursor.x - scroll.x) as u16;
            let y = rect.y + (cursor.y - scroll.y) as u16;
            (x, y)
        }
        Mode::Command => {
            let rect = layout.cmdline;
            (rect.x + ed.cmdline.cursor as u16 + 1, rect.y)
        }
    };

    queue!(stdout, style, MoveTo(x, y))?;
    Ok(())
}
