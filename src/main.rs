use std::io;
use std::time::Instant;

use crossterm::{event, queue};

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
        let start = Instant::now();
        terminal.draw(|stdout, screen| {
            crossterm::execute!(
                stdout,
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
                crossterm::cursor::MoveTo(0, 0),
                crossterm::style::SetBackgroundColor(crossterm::style::Color::White),
                crossterm::style::SetForegroundColor(crossterm::style::Color::Black),
                crossterm::style::Print("Foo"),
            )

            // let layout = Layout::from_screen(screen).unwrap();
            // editor.win_mut().sync_view(layout.buffer.h);
            // render(&editor, &layout, stdout)
        })?;

        // terminal.draw_frame(|frame| {
        //     frame.buffer.put_str(
        //         0,
        //         0,
        //         "Foo",
        //         crossterm::style::Color::Black,
        //         crossterm::style::Color::White,
        //     );
        // })?;
        med::debug!("took {:?}", start.elapsed());

        let event = event::read()?;
        // med::debug!("{event:?}");
        editor.update(event);
    }

    Ok(())
}
