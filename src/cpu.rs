use std::fs::File;
use std::io::prelude::*;
use std::io;

const TOTAL_MEMORY: usize = 4096;
const PROGRAM_OFFSET: usize = 0x200;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;

#[derive(Debug)]
pub struct CPU {
    // Instruction pointer
    ip: u16,

    // Stack pointer
    sp: u8,
    stack: [u16; STACK_SIZE],

    memory: [u8; TOTAL_MEMORY],
    registers: [u8; REGISTER_COUNT],
    addr_reg: u16,
}

impl CPU {
    pub fn init() -> Self {
        CPU {
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
        self.run_instr(instr);
    }

    fn run_instr(&mut self, instr: u16) {
        println!("Running instruction: {:#06x} @ IP: {:#06x}", instr, self.ip);

        // Increment IP before jumps
        self.ip += 2;

        // The instruction's first nibble indicates the type of operation. Some have multiple
        // different instructions. For those we'll switch later.
        // ASM-like notation and instructions taken from: http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
        let class: u8 = (instr >> 12) as u8;
        match class {
            0x0 => {
                if instr == 0x00EE { // RET
                    self.ip = self.stack[self.sp as usize];
                    self.sp -= 1;
                } else if instr == 0x00E0 { // CLS
                    // TODO clear screen
                } else {
                    panic!("Unimplemented call instruction")
                }
            }
            0x1 => { // 0x1nnn - JP addr
                self.ip = instr & 0x0FFF;
            }
            0x2 => { // 0x2nnn - CALL addr
                self.sp += 1;
                self.stack[self.sp as usize] = self.ip;
                self.ip = instr & 0x0FFF;
            }
            0x3 => { // 0x3xkk - SE Vx, byte
                let register = (instr >> 8) & 0x0F;
                let value = instr as u8;
                if self.registers[register as usize] == value {
                    self.ip += 2;
                }
            }
            0x4 => { // 0x4xkk - SNE Vx, byte
                let register = (instr >> 8) & 0x0F;
                let value = instr as u8;
                if self.registers[register as usize] != value {
                    self.ip += 2;
                }
            }
            0x5 => { // 0x5xy0 - SE Vx, Vy
                if instr & 0x0F != 0 {
                    panic!("Invalid skip operation")
                }

                let register_x = (instr >> 8) & 0x0F;
                let register_y = (instr >> 4) & 0x0F;
                if self.registers[register_x as usize] == self.registers[register_y as usize] {
                    self.ip += 2;
                }
            }
            0x6 => { // 0x6xkk - LE Vx, byte
                let register = (instr >> 8) & 0x0F;
                let value = instr as u8;
                self.registers[register as usize] = value;
            }
            0x7 => { // 0x7xkk - ADD Vx, byte
                let register = (instr >> 8) & 0x0F;
                self.registers[register as usize] += instr as u8;
            }
            0x8 => {
                let register_x = (instr >> 8) & 0x0F;
                let register_y = (instr >> 4) & 0x0F;
                let vx = self.registers[register_x as usize];
                let vy = self.registers[register_y as usize];
                self.registers[register_x as usize] = match (instr & 0x0F) as u8 {
                    0x0 => vy, // 0x8xy0 - LD Vx, Vy
                    0x1 => vx | vy, // 0x8xy1 - OR Vx, Vy
                    0x2 => vx & vy, // 0x8xy2 - AND Vx, Vy
                    0x3 => vx ^ vy, // 0x8xy3 - XOR Vx, Vy
                    0x4 => { // 0x8xy4 - ADD Vx, Vy
                        let (res, ovl) = vx.overflowing_add(vy);
                        self.registers[0xF as usize] = ovl as u8;
                        res
                    }
                    0x5 => { // 0x8xy5 - SUB Vx, Vy
                        let (res, ovl) = vx.overflowing_sub(vy);
                        self.registers[0xF as usize] = !ovl as u8;
                        res
                    }
                    0x6 => { // 0x8xy6 - SHR Vx {, Vy}
                        self.registers[0xF as usize] = vx & 1;
                        vx >> 1
                    }
                    0x7 => { // 0x8xy7 - SUBN Vx, Vy
                        let (res, ovl) = vy.overflowing_sub(vx);
                        self.registers[0xF as usize] = !ovl as u8;
                        res
                    }
                    0x8 => { // 0x8xyE - SHL Vx {, Vy}
                        self.registers[0xF as usize] = vx >> 7;
                        vx << 1
                    }
                    _ => panic!("Invalid bit operation")
                };
            }
            0x9 => { // 0x9xy0 - SNE Vx, Vy
                if instr & 0x0F != 0 {
                    panic!("Invalid skip operation")
                }

                let register_x = (instr >> 8) & 0x0F;
                let register_y = (instr >> 4) & 0x0F;
                if self.registers[register_x as usize] != self.registers[register_y as usize] {
                    self.ip += 2;
                }
            }
            0xA => { // 0xAnnn - LD I, addr
                self.addr_reg = instr & 0x0FFF;
            }
            0xB => { // 0xBnnn - JP V0, addr
                self.ip = (instr & 0x0FFF) + (self.registers[0x0] as u16);
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
