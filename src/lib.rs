mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern crate web_sys;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[derive(Debug, Copy, Clone)]
enum Instruction {
    Lsl { rs: u8, rd: u8, offset: u8 },
    Lsr { rs: u8, rd: u8, offset: u8 },
    Asr { rs: u8, rd: u8, offset: u8 },
    AddReg { rs: u8, rd: u8, rn: u8 },
    AddImm { rs: u8, rd: u8, offset: u8 },
    SubReg { rs: u8, rd: u8, rn: u8 },
    SubImm { rs: u8, rd: u8, offset: u8 },
    MovImm { rd: u8, offset: u8 },
    CmpImm { rd: u8, offset: u8 },
}

struct CondRegister {
    n: bool,
    z: bool,
    v: bool,
    c: bool,
}

#[wasm_bindgen]
pub struct Gamebuino {
    instructions: Vec<Instruction>,
    cond_reg: CondRegister,
    registers: [u32; 16],
}

#[wasm_bindgen]
impl Gamebuino {
    pub fn new() -> Gamebuino {
        utils::set_panic_hook();
        let mut result = Gamebuino {
            instructions: Vec::new(),
            cond_reg: CondRegister {
                n: false,
                z: false,
                v: false,
                c: false,
            },
            registers: [0; 16],
        };
        result.load_sample_instructions();
        result
    }

    pub fn dummy(&self) {
        log!("It works! {:?}", &self.instructions);
    }

    pub fn step(&mut self) {
        let instruction = *self.instructions.get(1).unwrap();
        self.execute_instruction(instruction);
    }

    pub fn run(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    fn read_register(&self, register: u8) -> u32 {
        self.registers[register as usize]
    }

    fn set_register(&mut self, register: u8, value: u32) {
        self.registers[register as usize] = value;
    }

    fn set_nz(&mut self, value: u32) {
        self.cond_reg.n = (value & 0x80000000) != 0;
        self.cond_reg.z = value == 0;
    }

    fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Lsl { rs, rd, offset } => {
                let original = self.read_register(rs);
                let result = original << offset;
                self.set_register(rd, result);
                self.cond_reg.c = original & (1 << offset) != 0;
                self.set_nz(result);
            }
            Instruction::Lsr { rs, rd, offset } => {
                let original = self.read_register(rs);
                let result = original >> offset;
                self.set_register(rd, result);
                self.cond_reg.c = original & (1 << (32 - offset)) != 0;
                self.set_nz(result);
            }
            Instruction::Asr { rs, rd, offset } => {
                let original = self.read_register(rs) as i32;
                let result = (original >> offset) as u32;
                self.set_register(rd, result);
                self.cond_reg.c = original & (1 << (offset - 1)) != 0;
                self.set_nz(result);
            }
            _ => panic!("Unimplemented instruction!"),
        }
    }

    fn load_sample_instructions(&mut self) {
        let data: Vec<u16> = vec![0b0000000100001010, 0b0001000100001011, 0b0010000000000000];
        for instruction in data.iter().map(|i| Gamebuino::parse_instruction(*i)) {
            self.instructions.push(instruction);
        }
    }

    fn parse_instruction(instruction: u16) -> Instruction {
        if instruction & 0b1110000000000000 == 0b0000000000000000 {
            let rs = ((instruction & 0b0000000000111000) >> 3) as u8;
            let rd = ((instruction & 0b0000000000000111) >> 0) as u8;
            if (instruction & 0b0001100000000000) != 0b0001100000000000 {
                let opcode = (instruction & 0b0001100000000000) >> 11;
                let offset = ((instruction & 0b0000011111000000) >> 6) as u8;
                match opcode {
                    0 => Instruction::Lsl { rs, rd, offset },
                    1 => Instruction::Lsr { rs, rd, offset },
                    2 => Instruction::Asr { rs, rd, offset },
                    _ => panic!("Unexpected opcode {}", opcode),
                }
            } else {
                let opcode = (instruction & 0b0000011000000000) >> 9;
                let rn_offset = ((instruction & 0b0000000111000000) >> 6) as u8;
                match opcode {
                    0b00 => Instruction::AddReg {
                        rs,
                        rd,
                        rn: rn_offset,
                    },
                    0b10 => Instruction::AddImm {
                        rs,
                        rd,
                        offset: rn_offset,
                    },
                    0b01 => Instruction::SubReg {
                        rs,
                        rd,
                        rn: rn_offset,
                    },
                    0b11 => Instruction::SubImm {
                        rs,
                        rd,
                        offset: rn_offset,
                    },
                    _ => panic!("Unexpected opcode {}", opcode),
                }
            }
        } else if instruction & 0b1110000000000000 == 0b0010000000000000 {
            let opcode = (instruction & 0b0001100000000000) >> 11;
            let rd = ((instruction & 0b0000011100000000) >> 8) as u8;
            let offset = (instruction & 0b11111111) as u8;
            match opcode {
                0b00 => Instruction::MovImm { rd, offset },
                0b01 => Instruction::CmpImm { rd, offset },
                0b10 => Instruction::AddImm { rs: rd, rd, offset },
                0b11 => Instruction::SubImm { rs: rd, rd, offset },
                _ => panic!("Unexpected opcode {}", opcode),
            }
        } else {
            panic!("Unimplemented instruction!");
        }
    }
}
