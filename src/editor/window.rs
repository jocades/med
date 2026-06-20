use crate::layout::Vec2;

#[derive(Debug, Default, Clone, Copy)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    /// Preferred column for vertical movement.
    pub want_x: usize,
}

#[derive(Default)]
pub struct Window {
    pub bufid: usize,
    pub cursor: Cursor,
    /// The top-left viewport offset into the buffer.
    pub scroll: Vec2<usize>,
}

impl Window {
    pub fn set_buf(&mut self, bufid: usize, last_view: Option<(Cursor, Vec2<usize>)>) {
        self.bufid = bufid;
        if let Some((cursor, scroll)) = last_view {
            self.cursor = cursor;
            self.scroll = scroll;
        } else {
            self.cursor = Cursor::default();
            self.scroll = Vec2::default();
        }
    }

    pub fn sync_view(&mut self, view_h: u16) {
        let h = view_h as usize;

        if self.cursor.y < self.scroll.y {
            self.scroll.y = self.cursor.y;
        } else if self.cursor.y >= self.scroll.y + h {
            self.scroll.y = self.cursor.y + 1 - h;
        }
    }
}
