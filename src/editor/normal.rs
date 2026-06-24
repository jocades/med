use crossterm::event::{KeyCode, KeyEvent};

use super::motion::{self, Motion};
use super::operator::{self, Operator};
use super::{Editor, Mode};

pub fn handle(ed: &mut Editor, key: KeyEvent) {
    use KeyCode::*;

    if let Char(ch) = key.code
        && ch.is_ascii_digit()
        && (ch != '0' || ed.pending_count.is_some())
    {
        let n = ch.to_digit(10).unwrap();
        ed.push_pending_count_digit(n);
        return;
    }

    let count = ed.take_pending_count();

    if let Some(motion) = motion::parse(key.code, count) {
        motion::apply(ed, motion);
        return;
    }

    match key.code {
        // -> insert
        Char('i') => ed.mode = Mode::Insert,
        Char('a') => {
            ed.mode = Mode::Insert;
            ed.move_right();
        }
        Char('I') => {
            ed.mode = Mode::Insert;
            ed.move_bol();
        }
        Char('A') => {
            ed.mode = Mode::Insert;
            ed.move_eol();
        }
        Char('o') => {
            ed.insert_line(1, None);
            ed.mode = Mode::Insert;
            ed.cursor_mut().y += 1;
            ed.move_bol();
        }
        Char('O') => {
            ed.insert_line(0, None);
            ed.mode = Mode::Insert;
            ed.move_bol();
        }
        Char('s') => {
            if ed.line().len() != 0 {
                ed.remove_char(0);
            }
            ed.mode = Mode::Insert;
        }

        // -> command
        Char(':') => {
            ed.cmdline.clear();
            ed.message = None;
            ed.mode = Mode::Command;
        }

        // -> operator pending
        Char('d') => ed.mode = Mode::OperatorPending(operator::State::new(Operator::Delete)),
        Char('c') => ed.mode = Mode::OperatorPending(operator::State::new(Operator::Change)),

        // motion
        Char('G') => {
            ed.cursor_mut().y = ed.buf().lines.len() - 1;
            ed.cursor_clamp_x();
        }

        // edit
        Char('x') => ed.delete_under_cursor(),
        Char('D') => ed.delete_to_eol(),
        Char('C') => {
            ed.mode = Mode::Insert;
            ed.delete_to_eol();
        }

        _ => {}
    }
}
