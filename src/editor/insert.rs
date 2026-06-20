use crossterm::event::{KeyCode, KeyEvent};

use super::{Editor, Mode};

pub fn handle(ed: &mut Editor, key: KeyEvent) {
    match key.code {
        // insert -> normal
        KeyCode::Esc => {
            ed.mode = Mode::Normal;
            ed.move_left();
        }

        // movement
        KeyCode::Left => ed.move_left(),
        KeyCode::Right => ed.move_right(),
        KeyCode::Up => ed.move_up(),
        KeyCode::Down => ed.move_down(),

        // edit
        KeyCode::Char(ch) => {
            ed.insert_char(ch);
            let cur = ed.cursor_mut();
            cur.x += 1;
            cur.want_x = cur.x;
        }

        KeyCode::Enter => ed.enter(),
        KeyCode::Backspace => ed.backspace(),
        KeyCode::Tab => {
            ed.insert_char(' ');
            ed.insert_char(' ');
            let cur = ed.cursor_mut();
            cur.x += 2;
            cur.want_x = cur.x;
        }

        _ => {}
    }
}
