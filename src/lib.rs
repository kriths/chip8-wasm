use std::fs::File;
use std::io::prelude::*;

use wasm_bindgen::__rt::std::io::Error;

const TOTAL_MEMORY: usize = 4096;
const PROGRAM_OFFSET: usize = 0x200;

#[derive(Debug)]
struct CPU {
    ip: usize,
    memory: [u8; TOTAL_MEMORY],
    registers: [u8; 16],
}

impl CPU {
    fn init() -> Self {
        CPU {
            ip: 0x200,  // Programs start at address 0x200
            memory: [0; TOTAL_MEMORY],
            registers: [0; 16]
        }
    }

    fn load_from_file(&mut self, name: String) -> Result<(), Error> {
        const BUFFER_SIZE: usize = TOTAL_MEMORY - PROGRAM_OFFSET;
        let mut buffer = [0; BUFFER_SIZE];
        let mut file = File::open(name)?;
        file.read(&mut buffer)?;

        for i in 0..BUFFER_SIZE {
            self.memory[i + PROGRAM_OFFSET] = buffer[i];
        }

        Ok(())
    }

    fn iterate(&mut self) {
        let instr = self.memory[self.ip];
        println!("{}", instr);
    }
}

#[test]
fn main() -> Result<(), Error> {
    let mut C: CPU = CPU::init();

    C.load_from_file(String::from("games/TETRIS"))?;
    C.iterate();

    Ok(())
}
