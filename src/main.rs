use cpu::CPU;

mod cpu;

fn main() {
    // TODO get file name dynamically
    let file_name = String::from("games/TETRIS");

    let mut emu: CPU = CPU::init();
    emu
        .load_from_file(file_name)
        .expect("Cannot load from file");

    emu.run();
}
