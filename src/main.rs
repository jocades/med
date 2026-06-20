use std::io;

use crossterm::event;

use med::editor::{Buffer, Editor};
use med::layout::Layout;
use med::render::render;
use med::terminal::Terminal;

fn main() -> io::Result<()> {
    let buffer = match std::env::args().nth(1) {
        None => Buffer::default(),
        Some(path) => Buffer::from_path(&path)?,
    };

    let mut terminal = Terminal::new();
    let mut editor = Editor::with_buffer(buffer);

    while !editor.should_quit {
        terminal.draw(|stdout, screen| {
            let layout = Layout::from_screen(screen).unwrap();
            editor.win_mut().sync_view(layout.buffer.h);
            render(&editor, &layout, stdout)
        })?;

        let event = event::read()?;
        editor.update(event);
    }

    Ok(())
}
