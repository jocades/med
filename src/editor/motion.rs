use super::Editor;
use crossterm::event::KeyCode;

pub enum Motion {
    Left(usize),
    Right(usize),
    Up(usize),
    Down(usize),
    Bol,
    Eol,
}

pub fn parse(code: KeyCode, count: usize) -> Option<Motion> {
    use KeyCode::*;
    let motion = match code {
        Char('h') | Left => Motion::Left(count),
        Char('l') | Right => Motion::Right(count),
        Char('k') | Up => Motion::Up(count),
        Char('j') | Down => Motion::Down(count),
        Char('$') => Motion::Eol,
        Char('0') => Motion::Bol,
        _ => return None,
    };
    Some(motion)
}

pub fn apply(ed: &mut Editor, motion: Motion) {
    match motion {
        Motion::Left(n) => ed.move_left_n(n),
        Motion::Right(n) => ed.move_right_n(n),
        Motion::Up(n) => ed.move_up_n(n),
        Motion::Down(n) => ed.move_down_n(n),
        Motion::Bol => ed.move_bol(),
        Motion::Eol => ed.move_eol(),
    }
}
