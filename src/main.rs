use std::io::{self, StdoutLock, Write};

use anyhow::Result;
use crossterm::event::{KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::{
    cursor::*,
    event::{self, Event, KeyCode},
    execute,
    style::*,
    terminal::*,
};

fn init() -> Result<StdoutLock<'static>> {
    let mut stdout = io::stdout().lock();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    Ok(stdout)
}

fn restore(stdout: &mut StdoutLock<'static>) -> Result<()> {
    execute!(
        stdout,
        SetCursorStyle::DefaultUserShape,
        LeaveAlternateScreen,
    )?;
    disable_raw_mode()?;
    Ok(())
}

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
    // stdout: StdoutLock<'static>,
    mode: Mode,
    buffer: Vec<String>,
    cursor: Vec2,
    should_quit: bool,
}

impl App {
    fn start() -> Self {
        // let stdout = init().unwrap();

        Self {
            // stdout,
            mode: Mode::Normal,
            buffer: vec![String::new()],
            cursor: Vec2::zero(),
            should_quit: false,
        }
    }

    fn current_line(&self) -> &str {
        &self.buffer[self.cursor.y]
    }

    fn current_line_mut(&mut self) -> &mut String {
        &mut self.buffer[self.cursor.y]
    }

    fn move_left(&mut self) {
        self.cursor.x = self.cursor.x.saturating_sub(1);
    }

    fn move_right(&mut self) {
        if self.cursor.x < self.current_line().len() - (self.mode == Mode::Normal) as usize {
            self.cursor.x += 1;
        }
    }

    fn move_up(&mut self) {
        self.cursor.y = self.cursor.y.saturating_sub(1);
    }

    fn move_down(&mut self) {
        if self.cursor.y < self.buffer.len() - 1 {
            self.cursor.y += 1;
        }
    }
}

// impl Drop for App {
//     fn drop(&mut self) {
//         restore(&mut self.stdout).unwrap();
//     }
// }

fn render(app: &App, stdout: &mut StdoutLock<'static>) -> Result<()> {
    let (_, h) = size()?;

    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    // Buffer
    for (i, line) in app.buffer.iter().enumerate() {
        queue!(stdout, Print(line), MoveTo(0, i as u16 + 1))?;
    }

    // Bottom bar
    queue!(stdout, MoveTo(0, h.saturating_sub(1)))?;
    write!(
        stdout,
        "mode: {} row: {} col: {} lines: {}",
        app.mode,
        app.cursor.y,
        app.cursor.x,
        app.buffer.len(),
    )?;

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

fn update(app: &mut App) -> Result<()> {
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
        KeyCode::Char('i') => app.mode = Mode::Insert,
        KeyCode::Char('a') => {
            app.mode = Mode::Insert;
            app.move_right();
        }

        KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => app.move_left(),
        KeyCode::Char('l') | KeyCode::Right => app.move_right(),
        KeyCode::Char('k') | KeyCode::Up => app.move_up(),
        KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => app.move_down(),

        KeyCode::Char('x') => {
            if !app.current_line().is_empty() {
                app.buffer[app.cursor.y].remove(app.cursor.x);
                if app.cursor.x == app.current_line().len() {
                    app.move_left();
                }
            }
        }

        _ => {}
    }
}

fn insert(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.move_left();
        }

        KeyCode::Left => app.move_left(),
        KeyCode::Right => app.move_right(),
        KeyCode::Up => app.move_up(),
        KeyCode::Down => app.move_down(),

        KeyCode::Backspace => {
            if app.cursor.x > 0 {
                app.cursor.x -= 1;
                app.buffer[app.cursor.y].remove(app.cursor.x);
            } else if app.cursor.y > 0 {
                // remove line
                app.buffer.remove(app.cursor.y);
                app.cursor.y -= 1;
                app.cursor.x = app.buffer[app.cursor.y].len();
            }
        }

        KeyCode::Enter => {
            if app.cursor.x == app.current_line().len() {
                // at eol: add new empty line
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

        KeyCode::Char(ch) => {
            app.buffer[app.cursor.y].insert(app.cursor.x, ch);
            app.cursor.x += 1;
        }

        _ => {}
    }
}

fn main() -> Result<()> {
    let buffer = match std::env::args().nth(1) {
        Some(path) => std::fs::read_to_string(&path)?
            .lines()
            .map(str::to_owned)
            .collect(),
        None => vec![String::new()],
    };

    let mut stdout = init()?;

    let mut app = App {
        mode: Mode::Normal,
        buffer,
        cursor: Vec2::zero(),
        should_quit: false,
    };

    while !app.should_quit {
        render(&app, &mut stdout)?;
        update(&mut app)?;
    }

    restore(&mut stdout)?;

    Ok(())
}
