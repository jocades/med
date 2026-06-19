use std::io;

use crossterm::event;

use med::editor::Editor;
use med::render::render;
use med::terminal::Terminal;

fn main() -> io::Result<()> {
    let mut term = Terminal::new();
    let mut ed = Editor::new();

    while !ed.should_quit {
        render(&ed, term.stdout())?;
        let event = event::read()?;
        ed.update(event);
    }

    Ok(())
}
