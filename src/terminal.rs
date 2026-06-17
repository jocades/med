use std::io::{self, StdoutLock};

use crossterm::{cursor::SetCursorStyle, execute, terminal::*};

pub struct Terminal {
    stdout: StdoutLock<'static>,
}

impl Terminal {
    pub fn new() -> Self {
        Self { stdout: init() }
    }

    pub fn stdout(&mut self) -> &mut StdoutLock<'static> {
        &mut self.stdout
    }

    #[allow(unused)]
    pub fn draw<R>(&mut self, f: impl FnOnce(&mut StdoutLock<'static>) -> R) -> R {
        f(&mut self.stdout)
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
