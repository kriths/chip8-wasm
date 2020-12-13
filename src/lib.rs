use std::fs::File;
use std::io::prelude::*;
use std::io;

const TOTAL_MEMORY: usize = 4096;
const PROGRAM_OFFSET: usize = 0x200;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;

#[derive(Debug)]
pub struct Emulator {
    // Instruction pointer
    ip: u16,

    // Stack pointer
    sp: u8,
    stack: [u16; STACK_SIZE],

    memory: [u8; TOTAL_MEMORY],
    registers: [u8; REGISTER_COUNT],
    addr_reg: u16,
}

impl Emulator {
    pub fn init() -> Self {
        Emulator {
            ip: PROGRAM_OFFSET as u16,
            sp: 0x00,
            stack: [0; STACK_SIZE],
            memory: [0; TOTAL_MEMORY],
            registers: [0; REGISTER_COUNT],
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
        let instr = u16::from_be_bytes([self.memory[self.ip as usize], self.memory[(self.ip + 1) as usize]]);
        println!("Running instruction: {:#06x} @ IP: {:#06x}", instr, self.ip);

        // Increment IP before jumps
        self.ip += 2;

        // The instruction's first nibble indicates the type of operation. Some have multiple
        // different instructions. For those we'll switch later.
        let class: u8 = (instr >> 12) as u8;
        match class {
            0x02 => { // 0x2nnn - CALL addr
                self.stack[self.sp as usize] = self.ip;
                self.sp += 1;
                self.ip = instr & 0x0FFF;
            }
            0x0A => { // 0xAnnn - LD I, addr
                self.addr_reg = instr & 0x0FFF;
            }
            _ => panic!("Invalid instruction")
        }
    }

    pub fn run(&mut self) {
        loop {
            self.iterate();
        }
    }
}
