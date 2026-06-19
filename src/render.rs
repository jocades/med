//! # Turns editor state + layout into terminal drawing commands.

use std::io::{self, StdoutLock, Write};

use crossterm::{cursor::*, queue, style::*, terminal::*};

use crate::editor::{Editor, Mode};
use crate::layout::{Rect, Split};

pub fn render(ed: &Editor, stdout: &mut StdoutLock<'static>) -> io::Result<()> {
    let (w, h) = size()?;
    let screen = Rect::new(0, 0, w, h);

    let [window, status, command] = screen
        .vsplit([Split::Fill, Split::Fixed(1), Split::Fixed(1)])
        .unwrap();

    let [gutter, buffer] = window.hsplit([Split::Fixed(5), Split::Fill]).unwrap();

    queue!(stdout, Clear(ClearType::All))?;
    render_gutter(ed, stdout, gutter)?;
    render_buffer(ed, stdout, buffer)?;
    render_status(ed, stdout, status)?;
    render_command(ed, stdout, command)?;
    render_cursor(ed, stdout, buffer)?;
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

fn render_gutter(ed: &Editor, stdout: &mut StdoutLock<'static>, rect: Rect) -> io::Result<()> {
    static mut NUMBERS: Vec<String> = Vec::new();
    queue!(stdout, SetForegroundColor(Color::DarkGrey))?;

    let buf = ed.buf();

    for y in 0..rect.h as usize {
        if buf.lines.get(y).is_some() {
            #[allow(static_mut_refs)]
            let line_no = unsafe {
                if let Some(line_no) = NUMBERS.get(y) {
                    line_no
                } else {
                    NUMBERS.push((y + 1).to_string());
                    &NUMBERS[y]
                }
            };

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

fn render_buffer(ed: &Editor, stdout: &mut StdoutLock<'static>, rect: Rect) -> io::Result<()> {
    let buf = ed.buf();

    queue!(
        stdout,
        // (205, 214, 244)
        SetForegroundColor(0xcdd6f4.to_color()),
    )?;

    for y in 0..rect.h as usize {
        if let Some(line) = buf.lines.get(y) {
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

fn render_status(ed: &Editor, stdout: &mut StdoutLock<'static>, rect: Rect) -> io::Result<()> {
    let cursor = ed.cursor();

    let mode = format!(" {} ", ed.mode);
    let path = " src/main.rs ";
    let cursor = format!(" {},{} ", cursor.y, cursor.x);

    let pad = rect.w as usize - mode.len() - path.len();

    queue!(
        stdout,
        MoveTo(rect.x, rect.y),
        SetBackgroundColor(Color::Red),
        Print(mode.black().on_green()),
        ResetColor,
        Print(path),
        Print(format!("{cursor:>pad$}")),
        ResetColor,
    )?;

    Ok(())
}

fn render_cursor(ed: &Editor, stdout: &mut StdoutLock<'static>, rect: Rect) -> io::Result<()> {
    let cursor = ed.cursor();

    let style = match ed.mode {
        Mode::Insert => SetCursorStyle::SteadyBar,
        Mode::Normal => SetCursorStyle::SteadyBlock,
    };

    let x = rect.x + cursor.x as u16;
    let y = rect.y + cursor.y as u16;

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
