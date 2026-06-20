use crossterm::event::{KeyCode, KeyEvent};

use super::{Editor, Mode};

pub fn handle(ed: &mut Editor, key: KeyEvent) {
    match key.code {
        // normal -> insert
        KeyCode::Char('i') => ed.mode = Mode::Insert,
        KeyCode::Char('a') => {
            ed.mode = Mode::Insert;
            ed.move_right();
        }
        KeyCode::Char('I') => {
            ed.mode = Mode::Insert;
            ed.move_bol();
        }
        KeyCode::Char('A') => {
            ed.mode = Mode::Insert;
            ed.move_eol();
        }
        KeyCode::Char('o') => {
            ed.insert_line(1, None);
            ed.mode = Mode::Insert;
            ed.cursor_mut().y += 1;
            ed.move_bol();
        }
        KeyCode::Char('O') => {
            ed.insert_line(0, None);
            ed.mode = Mode::Insert;
            ed.move_bol();
        }

        // movement
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => ed.move_left(),
        KeyCode::Char('l') | KeyCode::Right => ed.move_right(),
        KeyCode::Char('k') | KeyCode::Up => ed.move_up(),
        KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => ed.move_down(),

        KeyCode::Char('$') => {
            ed.move_eol();
        }
        KeyCode::Char('0') => {
            ed.move_bol();
        }

        // edit
        KeyCode::Char('x') => ed.delete_under_cursor(),
        KeyCode::Char('D') => ed.delete_to_eol(),
        KeyCode::Char('C') => {
            ed.mode = Mode::Insert;
            ed.delete_to_eol();
        }

        // normal -> command
        KeyCode::Char(':') => {
            ed.cmdline.clear();
            ed.message = None;
            ed.mode = Mode::Command;
        }

        _ => {}
    }
}
