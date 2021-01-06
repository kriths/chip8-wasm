use std::fs::File;
use std::io;
use std::io::prelude::*;

use rand::prelude::*;

use crate::screen::Screen;
use crate::timer::Timer;

const TOTAL_MEMORY: usize = 4096;
const PROGRAM_OFFSET: usize = 0x200;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;

pub struct CPU {
    // Instruction pointer
    ip: u16,

    // Stack pointer
    sp: u8,
    stack: [u16; STACK_SIZE],

    memory: [u8; TOTAL_MEMORY],
    registers: [u8; REGISTER_COUNT],
    addr_reg: u16,

    rng: Box<dyn RngCore>,

    screen: Screen,

    delay_timer: Timer,
    sound_timer: Timer,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            ip: PROGRAM_OFFSET as u16,
            sp: 0x00,
            stack: [0; STACK_SIZE],
            memory: [0; TOTAL_MEMORY],
            registers: [0; REGISTER_COUNT],
            addr_reg: 0x0000,
            rng: Box::new(rand::thread_rng()),
            screen: Screen::new(),
            delay_timer: Timer::new(),
            sound_timer: Timer::new(),
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

    pub fn tick(&mut self) {
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
                if instr == 0x00EE { // 0x00EE - RET
                    self.ip = self.stack[self.sp as usize];
                    self.sp -= 1;
                } else if instr == 0x00E0 { // 0x00E0 - CLS
                    self.screen.clear();
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
                let register = ((instr >> 8) & 0x0F) as usize;
                self.registers[register] = self.registers[register].wrapping_add(instr as u8);
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
                    0xE => { // 0x8xyE - SHL Vx {, Vy}
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
            0xC => { // 0xCxkk - RND Vx, byte
                let register = (instr >> 8) & 0x0F;
                let mask = instr as u8;
                let rnd = self.rng.next_u32() as u8;
                self.registers[register as usize] = rnd & mask;
            }
            0xD => { // 0xDxyn - DRW Vx, Vy, nibble
                let mut any_changes = false;
                let x = ((instr >> 8) & 0x0F) as u8;
                let y_start = ((instr >> 12) & 0x0F) as u8;
                let line_count = (instr & 0x0F) as u8;
                for i in 0..=line_count {
                    let line = self.memory[(self.addr_reg + i as u16) as usize];
                    any_changes |= self.screen.draw_sprite_line(x, y_start + i, line);
                }
                self.registers[0xF] = any_changes as u8;
            }
            0xF => {
                let value = ((instr >> 8) & 0x0F) as u8;
                match instr as u8 {
                    0x07 => { // 0xFx07 - LD Vx, DT
                        self.registers[value as usize] = self.delay_timer.get_timeout();
                    }
                    0x15 => { // 0xFx15 - LD DT, Vx
                        self.delay_timer.set_timeout(self.registers[value as usize]);
                    }
                    0x18 => { // 0xFx18 - LD ST, Vx
                        self.sound_timer.set_timeout(self.registers[value as usize]);
                    }
                    0x1E => { // 0xFx1E - ADD I, Vx
                        self.addr_reg += self.registers[value as usize] as u16;
                    }
                    _ => panic!("Invalid instruction F")
                }
            }
            _ => panic!("Invalid instruction")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instr_ret() {
        let mut cpu = CPU::new();
        cpu.ip = 0x1234;
        cpu.stack[0] = 0x1111;
        cpu.stack[1] = 0x2222;
        cpu.stack[2] = 0x3333;
        cpu.sp = 1;

        cpu.run_instr(0x00EE);

        assert_eq!(0, cpu.sp);
        assert_eq!(0x2222, cpu.ip);
    }

    #[test]
    fn instr_jmp_imm() {
        let mut cpu = CPU::new();
        cpu.ip = 0x1234;
        cpu.sp = 1;

        cpu.run_instr(0x1456);

        assert_eq!(1, cpu.sp);
        assert_eq!(0x0456, cpu.ip);
    }

    #[test]
    fn instr_call() {
        let mut cpu = CPU::new();
        cpu.ip = 0x1234;
        cpu.stack[0] = 0x1111;
        cpu.stack[1] = 0x2222;
        cpu.stack[2] = 0x3333;
        cpu.sp = 0;

        cpu.run_instr(0x2456);

        assert_eq!(1, cpu.sp);
        assert_eq!(0x0456, cpu.ip);
        assert_eq!(0x1111, cpu.stack[0]);
        assert_eq!(0x1234 + 2, cpu.stack[1]);
        assert_eq!(0x3333, cpu.stack[2]);
    }

    #[test]
    fn instr_se_imm() {
        let mut cpu = CPU::new();
        cpu.ip = 0x0100;
        cpu.registers[1] = 0x12;

        cpu.run_instr(0x3113);
        assert_eq!(0x0102, cpu.ip);

        cpu.run_instr(0x3112);
        assert_eq!(0x0106, cpu.ip);
    }

    #[test]
    fn instr_sne_imm() {
        let mut cpu = CPU::new();
        cpu.ip = 0x0100;
        cpu.registers[1] = 0x12;

        cpu.run_instr(0x4113);
        assert_eq!(0x0104, cpu.ip);

        cpu.run_instr(0x4112);
        assert_eq!(0x0106, cpu.ip);
    }

    #[test]
    fn instr_se_reg() {
        let mut cpu = CPU::new();
        cpu.ip = 0x0100;
        cpu.registers[1] = 0x12;
        cpu.registers[2] = 0x12;
        cpu.registers[3] = 0x13;

        cpu.run_instr(0x5130);
        assert_eq!(0x0102, cpu.ip);

        cpu.run_instr(0x5120);
        assert_eq!(0x0106, cpu.ip);
    }

    #[test]
    fn instr_ld_imm() {
        let mut cpu = CPU::new();
        cpu.run_instr(0x6123);
        assert_eq!(0x23, cpu.registers[1]);
    }

    #[test]
    fn instr_add_imm() {
        let mut cpu = CPU::new();
        cpu.registers[1] = 0x11;
        cpu.run_instr(0x7122);
        assert_eq!(0x33, cpu.registers[1]);
    }

    #[test]
    fn instr_ld_reg() {
        let mut cpu = CPU::new();
        cpu.registers[1] = 0x11;
        cpu.registers[2] = 0x22;
        cpu.run_instr(0x8120);
        assert_eq!(0x22, cpu.registers[1]);
    }

    #[test]
    fn instr_or() {
        let mut cpu = CPU::new();
        cpu.registers[1] = 0b00110011;
        cpu.registers[2] = 0b00001111;
        cpu.run_instr(0x8121);
        assert_eq!(0b00111111, cpu.registers[1]);
    }

    #[test]
    fn instr_and() {
        let mut cpu = CPU::new();
        cpu.registers[1] = 0b00110011;
        cpu.registers[2] = 0b00001111;
        cpu.run_instr(0x8122);
        assert_eq!(0b00000011, cpu.registers[1]);
    }

    #[test]
    fn instr_xor() {
        let mut cpu = CPU::new();
        cpu.registers[1] = 0b00110011;
        cpu.registers[2] = 0b00001111;
        cpu.run_instr(0x8123);
        assert_eq!(0b00111100, cpu.registers[1]);
    }

    #[test]
    fn instr_add() {
        let mut cpu = CPU::new();
        cpu.registers[1] = 250;
        cpu.registers[2] = 5;

        cpu.run_instr(0x8124);
        assert_eq!(255, cpu.registers[1]);
        assert_eq!(0, cpu.registers[0xF]);

        cpu.run_instr(0x8124);
        assert_eq!(4, cpu.registers[1]);
        assert_eq!(1, cpu.registers[0xF]);
    }

    #[test]
    fn instr_sub() {
        let mut cpu = CPU::new();
        cpu.registers[1] = 7;
        cpu.registers[2] = 5;

        cpu.run_instr(0x8125);
        assert_eq!(2, cpu.registers[1]);
        assert_eq!(1, cpu.registers[0xF]);

        cpu.run_instr(0x8125);
        assert_eq!(253, cpu.registers[1]);
        assert_eq!(0, cpu.registers[0xF]);
    }

    #[test]
    fn instr_shr() {
        let mut cpu = CPU::new();
        cpu.registers[1] = 0b01011010;

        cpu.run_instr(0x8106);
        assert_eq!(0b00101101, cpu.registers[1]);
        assert_eq!(0, cpu.registers[0xF]);

        cpu.run_instr(0x8106);
        assert_eq!(0b00010110, cpu.registers[1]);
        assert_eq!(1, cpu.registers[0xF]);
    }

    #[test]
    fn instr_sub_i() {
        let mut cpu = CPU::new();
        cpu.registers[1] = 5;
        cpu.registers[2] = 7;

        cpu.run_instr(0x8127);
        assert_eq!(2, cpu.registers[1]);
        assert_eq!(1, cpu.registers[0xF]);

        cpu.registers[2] = 1;
        cpu.run_instr(0x8127);
        assert_eq!(255, cpu.registers[1]);
        assert_eq!(0, cpu.registers[0xF]);
    }

    #[test]
    fn instr_shl() {
        let mut cpu = CPU::new();
        cpu.registers[1] = 0b01011010;

        cpu.run_instr(0x810E);
        assert_eq!(0b10110100, cpu.registers[1]);
        assert_eq!(0, cpu.registers[0xF]);

        cpu.run_instr(0x810E);
        assert_eq!(0b01101000, cpu.registers[1]);
        assert_eq!(1, cpu.registers[0xF]);
    }

    #[test]
    fn instr_sne_reg() {
        let mut cpu = CPU::new();
        cpu.ip = 0x0100;
        cpu.registers[1] = 0x12;
        cpu.registers[2] = 0x12;
        cpu.registers[3] = 0x13;

        cpu.run_instr(0x9130);
        assert_eq!(0x0104, cpu.ip);

        cpu.run_instr(0x9120);
        assert_eq!(0x0106, cpu.ip);
    }

    #[test]
    fn instr_ld_addr() {
        let mut cpu = CPU::new();
        cpu.run_instr(0xA123);
        assert_eq!(0x0123, cpu.addr_reg);
    }

    #[test]
    fn instr_jmp_reg() {
        let mut cpu = CPU::new();
        cpu.ip = 0x1234;
        cpu.sp = 1;
        cpu.registers[0] = 0x22;

        cpu.run_instr(0xB111);

        assert_eq!(1, cpu.sp);
        assert_eq!(0x0133, cpu.ip);
    }

    #[test]
    fn instr_rnd() {
        let mut cpu = CPU::new();
        cpu.rng = Box::new(rand::rngs::mock::StepRng::new(0b00111100, 0));
        cpu.run_instr(0xC1F0);
        assert_eq!(0b00110000, cpu.registers[1]);
    }

    #[test]
    fn instr_ld_vx_dt() {
        let mut cpu = CPU::new();
        cpu.delay_timer.set_timeout(100);
        cpu.run_instr(0xF507);
        assert!(cpu.delay_timer.get_timeout() > 98);
        assert!(cpu.delay_timer.get_timeout() <= 100);
        assert!(cpu.registers[5] > 98);
        assert!(cpu.registers[5] <= 100);
    }

    #[test]
    fn instr_ld_dt_vx() {
        let mut cpu = CPU::new();
        cpu.registers[5] = 100;
        cpu.run_instr(0xF515);
        assert!(cpu.delay_timer.get_timeout() > 98);
        assert!(cpu.delay_timer.get_timeout() <= 100);
    }

    #[test]
    fn instr_ld_st_vx() {
        let mut cpu = CPU::new();
        cpu.registers[5] = 100;
        cpu.run_instr(0xF518);
        assert!(cpu.sound_timer.get_timeout() > 98);
        assert!(cpu.sound_timer.get_timeout() <= 100);
    }

    #[test]
    fn instr_add_addr() {
        let mut cpu = CPU::new();
        cpu.addr_reg = 0x9821;
        cpu.registers[5] = 0x56;
        cpu.run_instr(0xF51E);
        assert_eq!(0x9877, cpu.addr_reg);
    }
}
