use crossterm::event::{KeyCode, KeyEvent};

use super::motion::{self, Motion};
use super::{Editor, Mode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Delete,
    Change,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    pub op: Operator,
    pub count: Option<u32>,
}

impl State {
    pub fn new(op: Operator) -> Self {
        Self { op, count: None }
    }
}

pub fn handle(ed: &mut Editor, key: KeyEvent, state: State) {
    let count = ed.take_pending_count();

    if let Some(motion) = motion::parse(key.code, count) {
        match state.op {
            Operator::Delete => ed.mode = Mode::Normal,
            Operator::Change => ed.mode = Mode::Insert,
        }

        match state.op {
            Operator::Delete | Operator::Change => match motion {
                Motion::Left(_) => todo!(),
                Motion::Right(_) => todo!(),
                Motion::Up(n) => {
                    let start = ed.cursor().y.saturating_sub(n);
                }
                Motion::Down(n) => {
                    let _ = ed.delete_lines(n + 1);
                }
                Motion::Bol => ed.delete_to_bol(),
                Motion::Eol => ed.delete_to_eol(),
            },
        }

        return;
    }

    match key.code {
        KeyCode::Esc => {
            ed.mode = Mode::Normal;
        }
        KeyCode::Char(ch) if ch.is_ascii_digit() => {
            ed.mode = Mode::OperatorPending(State {
                op: state.op,
                count: Some(ch.to_digit(10).unwrap()),
            });
        }

        KeyCode::Char('d') if state.op == Operator::Delete => {
            ed.delete_line();
            ed.mode = Mode::Normal;
        }

        KeyCode::Char('c') if state.op == Operator::Change => {
            ed.line_mut().clear();
            ed.move_bol();
            ed.mode = Mode::Insert;
        }

        _ => {}
    }
}
