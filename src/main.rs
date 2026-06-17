use std::io;

use crossterm::event;

use med::editor::{self, Editor};
use med::terminal::Terminal;

fn main() -> io::Result<()> {
    let mut term = Terminal::new();
    let mut ed = Editor::new();

    while !ed.should_quit {
        // term.draw(|stdout| render(&app, stdout))?;
        editor::render(&ed, term.stdout())?;

        let event = event::read()?;
        ed.update(event);
    }

    Ok(())
}
