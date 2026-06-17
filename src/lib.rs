#![allow(unused)]
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
        io::stderr(),
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
