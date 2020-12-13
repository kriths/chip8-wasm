use std::fs::File;
use std::io::prelude::*;
use std::io;

const TOTAL_MEMORY: usize = 4096;
const PROGRAM_OFFSET: usize = 0x200;

#[derive(Debug)]
pub struct Emulator {
    ip: usize,
    memory: [u8; TOTAL_MEMORY],
    registers: [u8; 16],
    addr_reg: u16,
}

impl Emulator {
    pub fn init() -> Self {
        Emulator {
            ip: 0x200,  // Programs start at address 0x200
            memory: [0; TOTAL_MEMORY],
            registers: [0; 16],
            addr_reg: 0x0000,
        }
    }

    pub fn load_from_file(&mut self, name: String) -> Result<(), io::Error> {
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
        let instr = u16::from_be_bytes([self.memory[self.ip], self.memory[self.ip + 1]]);
        println!("Running instruction: {:#06x}", instr);

        // Increment IP before jumps
        self.ip += 2;

        if instr >> 12 == 0x0A {
            // 0xAnnn - LD I, addr
            self.addr_reg = instr & 0x0FFF;
        } else {
            panic!("Invalid instruction")
        }
    }

    pub fn run(&mut self) {
        loop {
            self.iterate();
        }
    }
}
