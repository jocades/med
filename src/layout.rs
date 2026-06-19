//! # Geometry and region splitting.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    NotEnoughSpace,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Split {
    Fill,
    Fixed(u16),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    pub fn hsplit<const N: usize>(&self, splits: [Split; N]) -> Result<[Rect; N], Error> {
        let lengths = compute_segments(&splits, self.w)?;

        let mut x = self.x;
        let mut out = [Rect::default(); N];

        for (i, w) in lengths.into_iter().enumerate() {
            out[i] = Rect::new(x, self.y, w, self.h);
            x += w;
        }

        Ok(out)
    }

    pub fn vsplit<const N: usize>(&self, splits: [Split; N]) -> Result<[Rect; N], Error> {
        let lengths = compute_segments(&splits, self.h)?;

        let mut y = self.y;
        let mut out = [Rect::default(); N];

        for (i, h) in lengths.into_iter().enumerate() {
            out[i] = Rect::new(self.x, y, self.w, h);
            y += h;
        }

        Ok(out)
    }
}

fn compute_segments<const N: usize>(
    splits: &[Split; N],
    available: u16,
) -> Result<[u16; N], Error> {
    let mut fixed_total = 0u16;
    let mut fill_count = 0u16;
    let mut last_fill_index = None;

    for (i, split) in splits.iter().enumerate() {
        match split {
            Split::Fixed(n) => {
                fixed_total = fixed_total.checked_add(*n).ok_or(Error::NotEnoughSpace)?;
            }
            Split::Fill => {
                fill_count += 1;
                last_fill_index = Some(i);
            }
        }
    }

    let min_required = fixed_total
        .checked_add(fill_count)
        .ok_or(Error::NotEnoughSpace)?;

    if min_required > available {
        return Err(Error::NotEnoughSpace);
    }

    let remaining = available - fixed_total;
    let fill_len = if fill_count > 0 {
        remaining / fill_count
    } else {
        0
    };

    let mut out = [0u16; N];
    let mut used = 0;

    for (i, split) in splits.iter().enumerate() {
        let len = match split {
            Split::Fixed(n) => *n,
            Split::Fill if last_fill_index == Some(i) => available - used,
            Split::Fill => fill_len,
        };

        out[i] = len;
        used += len;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{Rect, Split};

    type Result<T> = std::result::Result<T, super::Error>;

    #[test]
    fn rect_hsplit() -> Result<()> {
        let a = dbg!(Rect::new(0, 0, 4, 10).hsplit([Split::Fixed(2), Split::Fixed(2)]))?;
        assert_eq!(a[0], Rect::new(0, 0, 2, 10));
        assert_eq!(a[1], Rect::new(2, 0, 2, 10));

        let b =
            dbg!(Rect::new(0, 0, 4, 10).hsplit([Split::Fixed(1), Split::Fill, Split::Fixed(1)]))?;
        assert_eq!(b[0], Rect::new(0, 0, 1, 10));
        assert_eq!(b[1], Rect::new(1, 0, 2, 10));
        assert_eq!(b[2], Rect::new(3, 0, 1, 10));

        Ok(())
    }

    // #[test]
    // fn rect_vsplit() {
    //     let a = dbg!(Rect::new(0, 0, 10, 4).vsplit(&[Split::Fixed(2), Split::Fixed(2)]));
    //     assert_eq!(a.len(), 2);
    //     assert_eq!(a[0], Rect::new(0, 0, 10, 2));
    //     assert_eq!(a[1], Rect::new(0, 2, 10, 2));
    //
    //     let b =
    //         dbg!(Rect::new(0, 0, 10, 4).vsplit(&[Split::Fixed(1), Split::Fill, Split::Fixed(1)]));
    //     assert_eq!(b.len(), 3);
    //     assert_eq!(b[0], Rect::new(0, 0, 10, 1));
    //     assert_eq!(b[1], Rect::new(0, 1, 10, 2));
    //     assert_eq!(b[2], Rect::new(0, 3, 10, 1));
    // }
    //
    // // todo
    // #[test]
    // fn not_enough_room() {
    //     let _ = dbg!(Rect::new(0, 0, 10, 10).hsplit(&[Split::Fixed(20)]));
    // }
}
