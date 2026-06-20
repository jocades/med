use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};

use super::window::Cursor;
use crate::layout::Vec2;

pub struct Buffer {
    pub lines: Vec<String>,    // Invariant: at least one line
    pub path: Option<PathBuf>, // Invariant: absolute path
    pub is_dirty: bool,
    pub ends_with_newline: bool,
    /// To restore the last cursor and scroll when reentering the buffer.
    pub last_view: Option<(Cursor, Vec2<usize>)>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            path: None,
            is_dirty: false,
            ends_with_newline: false,
            last_view: None,
        }
    }
}

impl Buffer {
    pub fn from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();

        let (lines, ends_with_newline) = match fs::read_to_string(path) {
            Ok(s) => Self::lines_from_str(&s),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (vec![String::new()], false),
            Err(e) => return Err(e),
        };

        Ok(Self {
            lines,
            path: Some(path::absolute(path)?),
            ends_with_newline,
            ..Default::default()
        })
    }

    fn lines_from_str(s: &str) -> (Vec<String>, bool) {
        let ends_with_newline = s.ends_with('\n');
        let mut lines: Vec<_> = s.lines().map(str::to_owned).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        (lines, ends_with_newline)
    }

    pub fn write(&mut self) -> io::Result<()> {
        use io::Write;

        let path = self
            .path
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No file name"))?;

        let f = fs::File::create(path)?;
        let mut writer = io::BufWriter::new(f);

        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                writer.write_all(b"\n")?;
            }
            writer.write_all(line.as_bytes())?;
        }

        if self.ends_with_newline && !(self.lines.len() == 1 && self.lines[0].is_empty()) {
            writer.write_all(b"\n")?;
        }

        writer.flush()?;
        self.is_dirty = false;
        Ok(())
    }

    pub fn smudge(&mut self) {
        self.is_dirty = true;
    }
}
