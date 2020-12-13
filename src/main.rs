use lib::Emulator;

mod lib;

fn main() {
    // TODO get file name dynamically
    let file_name = String::from("games/TETRIS");

    let mut emu: Emulator = Emulator::init();
    emu
        .load_from_file(file_name)
        .expect("Cannot load from file");

    emu.run();
}
