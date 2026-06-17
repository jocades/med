use std::io::{self, StdoutLock, Write};

use crossterm::event::{KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::{
    cursor::*,
    event::{self, Event, KeyCode},
    style::*,
    terminal::*,
};

use med::{init, restore};

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: usize,
    y: usize,
}

impl Vec2 {
    const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    const fn zero() -> Self {
        Self::new(0, 0)
    }
}

#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => f.write_str("NORMAL"),
            Mode::Insert => f.write_str("INSERT"),
        }
    }
}

struct App {
    mode: Mode,
    buffer: Vec<String>,
    cursor: Vec2,
    should_quit: bool,
}

impl App {
    fn current_line(&self) -> &str {
        &self.buffer[self.cursor.y]
    }

    fn current_line_mut(&mut self) -> &mut String {
        &mut self.buffer[self.cursor.y]
    }

    fn cursor_max_x(&self) -> usize {
        let len = self.current_line().len();
        match self.mode {
            Mode::Insert => len,
            Mode::Normal => len.saturating_sub(1),
        }
    }

    fn cursor_clamp_x(&mut self) {
        self.cursor.x = self.cursor.x.min(self.cursor_max_x());
    }

    fn move_left(&mut self) {
        self.cursor.x = self.cursor.x.saturating_sub(1);
    }

    fn move_right(&mut self) {
        if self.cursor.x < self.cursor_max_x() {
            self.cursor.x += 1;
        }
    }

    fn move_up(&mut self) {
        self.cursor.y = self.cursor.y.saturating_sub(1);
        self.cursor_clamp_x();
    }

    fn move_down(&mut self) {
        if self.cursor.y < self.buffer.len() - 1 {
            self.cursor.y += 1;
            self.cursor_clamp_x();
        }
    }

    fn move_bol(&mut self) {
        self.cursor.x = 0;
    }

    fn move_eol(&mut self) {
        self.cursor.x = self.cursor_max_x();
    }
}

// impl Drop for App {
//     fn drop(&mut self) {
//         restore(&mut self.stdout).unwrap();
//     }
// }

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

fn render(app: &App, stdout: &mut StdoutLock<'static>) -> io::Result<()> {
    let (_, h) = size()?;

    queue!(stdout, Clear(ClearType::All))?;

    // Buffer
    for (i, line) in app.buffer.iter().enumerate() {
        queue!(stdout, MoveTo(0, i as u16), Print(line))?;
    }

    // Statusline
    queue!(
        stdout,
        MoveTo(0, h.saturating_sub(1)),
        SetForegroundColor(Color::Black),
        SetBackgroundColor(Color::White),
    )?;
    write!(
        stdout,
        "mode: {} row: {} col: {} lines: {}",
        app.mode,
        app.cursor.y,
        app.cursor.x,
        app.buffer.len(),
    )?;
    queue!(stdout, ResetColor)?;

    // Cursor
    let cursor_style = match app.mode {
        Mode::Insert => SetCursorStyle::SteadyBar,
        _ => SetCursorStyle::SteadyBlock,
    };

    queue!(
        stdout,
        cursor_style,
        MoveTo(app.cursor.x as u16, app.cursor.y as u16)
    )?;

    stdout.flush()?;
    Ok(())
}

fn update(app: &mut App) -> io::Result<()> {
    match event::read()? {
        Event::Key(key) => {
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.should_quit = true;
                return Ok(());
            }

            match app.mode {
                Mode::Normal => normal(app, key),
                Mode::Insert => insert(app, key),
            }
        }
        _ => {}
    }

    Ok(())
}

fn normal(app: &mut App, key: KeyEvent) {
    match key.code {
        // normal -> insert
        KeyCode::Char('i') => app.mode = Mode::Insert,
        KeyCode::Char('a') => {
            app.mode = Mode::Insert;
            app.move_right();
        }
        KeyCode::Char('I') => {
            app.mode = Mode::Insert;
            app.cursor.x = 0;
        }
        KeyCode::Char('A') => {
            app.mode = Mode::Insert;
            app.move_eol();
        }
        KeyCode::Char('o') => {
            app.buffer.insert(app.cursor.y + 1, String::new());
            app.mode = Mode::Insert;
            app.cursor.x = 0;
            app.cursor.y += 1;
        }
        KeyCode::Char('O') => {
            app.buffer.insert(app.cursor.y, String::new());
            app.mode = Mode::Insert;
            app.cursor.x = 0;
        }

        // movement
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => app.move_left(),
        KeyCode::Char('l') | KeyCode::Right => app.move_right(),
        KeyCode::Char('k') | KeyCode::Up => app.move_up(),
        KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => app.move_down(),

        KeyCode::Char('$') => {
            app.move_eol();
        }
        KeyCode::Char('0') => {
            app.move_bol();
        }

        // edit
        KeyCode::Char('x') => {
            if !app.current_line().is_empty() {
                app.buffer[app.cursor.y].remove(app.cursor.x);
                if app.cursor.x == app.current_line().len() {
                    app.move_left();
                }
            }
        }

        KeyCode::Char('D') => {
            if app.cursor.x < app.current_line().len() {
                app.buffer[app.cursor.y].truncate(app.cursor.x);
                app.move_left();
            }
        }

        _ => {}
    }
}

fn insert(app: &mut App, key: KeyEvent) {
    match key.code {
        // insert -> normal
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.move_left();
        }

        // movement
        KeyCode::Left => app.move_left(),
        KeyCode::Right => app.move_right(),
        KeyCode::Up => app.move_up(),
        KeyCode::Down => app.move_down(),

        // edit
        KeyCode::Char(ch) => {
            app.buffer[app.cursor.y].insert(app.cursor.x, ch);
            app.cursor.x += 1;
        }

        KeyCode::Enter => {
            if app.cursor.x == app.current_line().len() {
                // at end: add new empty line
                app.buffer.insert(app.cursor.y + 1, String::new());
            } else {
                // at mid: split
                let rhs = app.current_line()[app.cursor.x..].to_owned();
                app.buffer[app.cursor.y].truncate(app.cursor.x);
                app.buffer.insert(app.cursor.y + 1, rhs);
            }

            app.cursor.y += 1;
            app.cursor.x = 0;
        }

        KeyCode::Backspace => {
            if app.cursor.x > 0 {
                // at mid or end: remove char
                app.cursor.x -= 1;
                app.buffer[app.cursor.y].remove(app.cursor.x);
            } else if app.cursor.y > 0 {
                // at not eof and bol: join with upper line
                let line = app.buffer.remove(app.cursor.y);
                app.cursor.y -= 1;
                app.cursor.x = app.current_line().len();
                app.current_line_mut().push_str(&line);
            }
        }

        _ => {}
    }
}

struct Terminal {
    stdout: StdoutLock<'static>,
}

impl Terminal {
    fn new() -> Self {
        Self { stdout: init() }
    }

    fn stdout(&mut self) -> &mut StdoutLock<'static> {
        &mut self.stdout
    }

    #[allow(unused)]
    fn draw<R>(&mut self, f: impl FnOnce(&mut StdoutLock<'static>) -> R) -> R {
        f(&mut self.stdout)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        restore();
    }
}

fn main() -> io::Result<()> {
    let mut term = Terminal::new();

    let mut app = App {
        mode: Mode::Normal,
        buffer: vec![String::new()],
        cursor: Vec2::zero(),
        should_quit: false,
    };

    while !app.should_quit {
        // term.draw(|stdout| render(&app, stdout))?;
        render(&app, term.stdout())?;
        update(&mut app)?;
    }

    Ok(())
}
