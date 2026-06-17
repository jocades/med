use std::{panic, thread, time::Duration};

use crossterm::event::read;

fn main() {
    // let mut stdout = med::init().unwrap();

    // let _ = panic::catch_unwind(|| {
    //     panic!("first");
    // });

    // panic!("second");

    panic::set_hook(Box::new(|info| {
        println!("Custom panic info");
        println!("{info:?}");
    }));

    panic!("Normal panic");

    read().unwrap();

    // med::restore(&mut stdout).unwrap();
}
