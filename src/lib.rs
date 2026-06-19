pub mod editor;
pub mod layout;
pub mod render;
pub mod terminal;

use std::{fs::File, sync::LazyLock};

pub(crate) static mut DEBUG_FILE: LazyLock<File> = LazyLock::new(|| {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
        .expect("failed to open log file")
});

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        use std::io::Write;
        #[allow(static_mut_refs)]
        unsafe { writeln!($crate::DEBUG_FILE, $($arg)*).unwrap(); }
    }};
}
