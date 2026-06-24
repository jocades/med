use super::{Cursor, Editor, Mode};

impl Editor {
    // --  Move --
    pub fn move_to(&mut self, x: usize, y: usize) {
        *self.cursor_mut() = Cursor { x, y, want_x: x };
        self.cursor_clamp_x();
    }

    pub fn cursor_clamp_x(&mut self) {
        let max = self.cursor_max_x();
        let cur = self.cursor_mut();
        cur.x = cur.x.min(max);
    }

    pub fn cursor_clamp_sync_x(&mut self) {
        self.cursor_clamp_x();
        let cur = self.cursor_mut();
        cur.want_x = cur.x;
    }

    pub fn move_left(&mut self) {
        let cur = self.cursor_mut();
        cur.x = cur.x.saturating_sub(1);
        cur.want_x = cur.x;
    }

    pub fn move_left_n(&mut self, count: usize) {
        let cur = self.cursor_mut();
        cur.x = cur.x.saturating_sub(count);
        cur.want_x = cur.x;
    }

    pub fn move_right(&mut self) {
        let max = self.cursor_max_x();
        let cur = self.cursor_mut();
        if cur.x < max {
            cur.x += 1;
            cur.want_x = cur.x;
        }
    }

    pub fn move_right_n(&mut self, count: usize) {
        let max = self.cursor_max_x();
        let cur = self.cursor_mut();
        cur.x = (cur.x + count).min(max);
        cur.want_x = cur.x;
    }

    pub fn move_up(&mut self) {
        let cur = self.cursor_mut();
        cur.y = cur.y.saturating_sub(1);
        cur.x = cur.want_x;
        self.cursor_clamp_x();
    }

    pub fn move_up_n(&mut self, count: usize) {
        let cur = self.cursor_mut();
        cur.y = cur.y.saturating_sub(count);
        cur.x = cur.want_x;
        self.cursor_clamp_x();
    }

    pub fn move_down(&mut self) {
        let max = self.buf().lines.len() - 1;
        let cur = self.cursor_mut();
        if cur.y < max {
            cur.y += 1;
            cur.x = cur.want_x;
            self.cursor_clamp_x();
        }
    }

    pub fn move_down_n(&mut self, count: usize) {
        let max = self.buf().lines.len() - 1;
        let cur = self.cursor_mut();
        cur.y = (cur.y + count).min(max);
        cur.x = cur.want_x;
        self.cursor_clamp_x();
    }

    pub fn move_bol(&mut self) {
        let cur = self.cursor_mut();
        cur.x = 0;
        cur.want_x = cur.x;
    }

    pub fn move_eol(&mut self) {
        let max = self.cursor_max_x();
        let cur = self.cursor_mut();
        cur.x = max;
        cur.want_x = cur.x;
    }

    // -- Edit --
    pub fn insert_char(&mut self, ch: char) {
        let col = self.cursor().x;
        self.line_mut().insert(col, ch);
        self.buf_mut().smudge();
    }

    pub fn remove_char(&mut self, offset_x: i32) -> char {
        let col = self.cursor().x as i32 + offset_x;
        self.buf_mut().smudge();
        self.line_mut().remove(col as usize)
    }

    pub fn insert_line(&mut self, offset_y: i32, content: Option<String>) {
        let row = self.cursor().y as i32 + offset_y;
        self.buf_mut().smudge();
        self.buf_mut()
            .lines
            .insert(row as usize, content.unwrap_or_default());
    }

    pub fn remove_line(&mut self, offset_y: i32) -> String {
        let row = self.cursor().y as i32 + offset_y;
        self.buf_mut().smudge();
        self.buf_mut().lines.remove(row as usize)
    }

    fn cursor_max_x(&self) -> usize {
        let len = self.line().len();
        match self.mode {
            Mode::Insert => len,
            Mode::Normal | Mode::OperatorPending(_) => len.saturating_sub(1),
            Mode::Command => unreachable!("cmdline uses its own cursor semantics"),
        }
    }

    pub fn delete_line(&mut self) {
        let line_count = self.buf().lines.len();

        if line_count == 1 {
            self.line_mut().clear();
        } else {
            self.remove_line(0);
        }

        if self.cursor().y == line_count - 1 {
            self.move_up();
        }

        self.cursor_clamp_sync_x();
    }

    pub fn delete_lines(&mut self, count: usize) -> Vec<String> {
        let start = self.cursor().y;
        let end = (start + count).min(self.buf().lines.len());

        let deleted: Vec<_> = self.buf_mut().lines.drain(start..end).collect();

        if self.buf().lines.is_empty() {
            self.buf_mut().lines.push(String::new());
        }

        let ny = start.min(self.buf().lines.len() - 1);
        self.cursor_mut().y = ny;
        self.cursor_clamp_sync_x();
        self.buf_mut().smudge();

        deleted
    }

    pub fn delete_under_cursor(&mut self) {
        if self.line().is_empty() {
            return;
        }

        self.remove_char(0);
        self.cursor_clamp_sync_x();
    }

    pub fn delete_to_eol(&mut self) {
        let x = self.cursor().x;

        if x >= self.line().len() {
            return;
        }

        self.line_mut().truncate(x);
        self.cursor_clamp_sync_x();
        self.buf_mut().smudge();
    }

    pub fn delete_to_bol(&mut self) {
        let x = self.cursor().x;
        if x == 0 {
            return;
        }
        self.line_mut().drain(..x);
        self.cursor_clamp_sync_x();
        self.buf_mut().smudge();
    }

    pub fn enter(&mut self) {
        let cur = self.cursor();

        if cur.x == self.line().len() {
            // at end: add new empty line
            self.insert_line(1, None);
        } else {
            // at mid: split
            let rhs = self.line()[cur.x..].to_owned();
            self.line_mut().truncate(cur.x);
            self.insert_line(1, Some(rhs))
        }

        self.move_to(0, cur.y + 1);
    }

    pub fn backspace(&mut self) {
        let cur = self.cursor();

        if cur.x > 0 {
            // at mid or eol: remove char
            self.move_left();
            self.remove_char(0);
            return;
        }

        if cur.y > 0 {
            // at not eof and bol: join with upper line
            let line = self.remove_line(0);
            let nx = self.line_off(-1).len();
            self.move_to(nx, cur.y - 1);
            self.line_mut().push_str(&line);
        }
    }
}
