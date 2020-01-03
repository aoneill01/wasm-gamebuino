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

#[derive(Debug)]
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

#[wasm_bindgen]
pub struct Gamebuino {
    instructions: Vec<Instruction>,
}

#[wasm_bindgen]
impl Gamebuino {
    pub fn new() -> Gamebuino {
        utils::set_panic_hook();
        let mut result = Gamebuino {
            instructions: Vec::new(),
        };
        result.load_sample_instructions();
        result
    }

    pub fn dummy(&self) {
        log!("It works! {:?}", &self.instructions);
    }

    fn load_sample_instructions(&mut self) {
        let data: Vec<u16> = vec![0b0000000100001010, 0b0001000100001011];
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
        } else {
            panic!("Unimplemented instruction!");
        }
    }
}
