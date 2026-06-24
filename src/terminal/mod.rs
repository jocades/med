//! # Terminal lifecycle and output handle ownership.

use std::io::{self, StdoutLock, Write};

use crossterm::{cursor::*, execute, queue, style::*, terminal::*};

use crate::layout::Rect;

mod buffer;

use buffer::{Buffer, Cell};

pub struct Terminal {
    stdout: StdoutLock<'static>,
    buffers: [Buffer; 2],
    current: usize,
}

pub struct Frame<'a> {
    pub buffer: &'a mut Buffer,
}

impl Frame<'_> {
    pub fn size(&self) -> (usize, usize) {
        (self.buffer.w, self.buffer.h)
    }
}

impl Terminal {
    pub fn new() -> Self {
        let stdout = init();
        let (w, h) = size().expect("failed to get terminal size");
        let buffer = Buffer::new(w as usize, h as usize);
        Self {
            stdout,
            buffers: [buffer.clone(), buffer],
            current: 0,
        }
    }

    pub fn draw<F, R>(&mut self, f: F) -> io::Result<R>
    where
        F: FnOnce(&mut StdoutLock<'static>, Rect) -> io::Result<R>,
    {
        let (w, h) = size()?;
        let screen = Rect::new(0, 0, w, h);
        f(&mut self.stdout, screen)
    }

    pub fn draw_frame<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        let mut frame = self.get_frame();
        f(&mut frame);

        let curr = &self.buffers[self.current];
        let prev = &self.buffers[1 - self.current];
        let changes = prev.diff(curr);

        let prev_x = 0;
        let prev_y = 0;
        let prev_fg = Color::Reset;
        let prev_bg = Color::Reset;

        crate::debug!("{changes:#?}");

        for (x, y, Cell { ch, fg, bg }) in changes {
            if x != prev_x + 1 || y != prev_y {
                queue!(self.stdout, MoveTo(x as u16, y as u16))?;
            }

            if fg != prev_fg {
                queue!(self.stdout, SetForegroundColor(fg))?;
            }

            if bg != prev_bg {
                queue!(self.stdout, SetBackgroundColor(bg))?;
            }

            queue!(self.stdout, Print(ch))?;
        }

        self.stdout.flush()?;
        self.swap_buffers();

        Ok(())
    }

    fn current_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffers[self.current]
    }

    fn get_frame(&mut self) -> Frame<'_> {
        Frame {
            buffer: self.current_buffer_mut(),
        }
    }

    fn swap_buffers(&mut self) {
        self.current = 1 - self.current;
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        restore();
    }
}

pub fn init() -> StdoutLock<'static> {
    try_init().expect("failed to initialize terminal")
}

pub fn try_init() -> io::Result<StdoutLock<'static>> {
    set_panic_hook();
    let mut stdout = io::stdout().lock();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    Ok(stdout)
}

pub fn restore() {
    if let Err(e) = try_restore() {
        eprintln!("failed to restore terminal: {e}")
    }
}

pub fn try_restore() -> io::Result<()> {
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        SetCursorStyle::DefaultUserShape,
    )?;
    disable_raw_mode()?;
    Ok(())
}

fn set_panic_hook() {
    use std::panic;
    use std::sync::Once;

    static PANIC_HOOK: Once = Once::new();

    PANIC_HOOK.call_once(|| {
        let hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            restore();
            hook(info);
        }));
    });
}
