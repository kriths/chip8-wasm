use std::{thread, time};

use cpu::CPU;

mod cpu;
mod screen;
mod timer;

fn main() {
    // TODO get file name dynamically
    let file_name = String::from("games/TETRIS");

    let mut emu: CPU = CPU::new();
    emu
        .load_from_file(file_name)
        .expect("Cannot load from file");

    loop {
        emu.tick();
        thread::sleep(time::Duration::from_millis(100));
    }
}
