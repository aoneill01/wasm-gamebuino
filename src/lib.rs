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
    LslImm { rs: u8, rd: u8, offset: u8 },
    LslReg { rs: u8, rd: u8 },
    LsrImm { rs: u8, rd: u8, offset: u8 },
    LsrReg { rs: u8, rd: u8 },
    AsrImm { rs: u8, rd: u8, offset: u8 },
    AsrReg { rs: u8, rd: u8 },
    AddReg { rs: u8, rd: u8, rn: u8 },
    AddImm { rs: u8, rd: u8, offset: u8 },
    AddSp { rd: u8, offset: u32 },
    AddPc { rd: u8, offset: u32 },
    Adc { rs: u8, rd: u8 },
    SubReg { rs: u8, rd: u8, rn: u8 },
    SubImm { rs: u8, rd: u8, offset: u8 },
    Sbc { rs: u8, rd: u8 },
    Neg { rs: u8, rd: u8 },
    Mul { rs: u8, rd: u8 },
    MovImm { rd: u8, offset: u8 },
    MovReg { rs: u8, rd: u8 },
    Mvn { rs: u8, rd: u8 },
    CmpImm { rd: u8, offset: u8 },
    CmpReg { rs: u8, rd: u8 },
    Cmn { rs: u8, rd: u8 },
    Tst { rs: u8, rd: u8 },
    And { rs: u8, rd: u8 },
    Bic { rs: u8, rd: u8 },
    Eor { rs: u8, rd: u8 },
    Oor { rs: u8, rd: u8 },
    Bx { rs: u8 },
    Blx { rm: u8 },
    LdrPc { rd: u8, immediate_value: u32 },
    LdrReg { rb: u8, ro: u8, rd: u8 },
    LdrbReg { rb: u8, ro: u8, rd: u8 },
    LdrImm { rb: u8, offset: u32, rd: u8 },
    LdrbImm { rb: u8, offset: u32, rd: u8 },
    Ldsb { rb: u8, ro: u8, rd: u8 },
    LdrhReg { rb: u8, ro: u8, rd: u8 },
    LdrhImm { rb: u8, offset: u32, rd: u8 },
    Ldsh { rb: u8, ro: u8, rd: u8 },
    Ldmia { rb: u8, rlist: u8 },
    StrReg { rb: u8, ro: u8, rd: u8 },
    StrbReg { rb: u8, ro: u8, rd: u8 },
    StrImm { rb: u8, offset: u32, rd: u8 },
    StrbImm { rb: u8, offset: u32, rd: u8 },
    StrhReg { rb: u8, ro: u8, rd: u8 },
    StrhImm { rb: u8, offset: u32, rd: u8 },
    Stmia { rb: u8, rlist: u8 },
    Sxth { rd: u8, rm: u8 },
    Sxtb { rd: u8, rm: u8 },
    Uxth { rd: u8, rm: u8 },
    Uxtb { rd: u8, rm: u8 },
    Rev { rd: u8, rm: u8 },
    Rev16 { rd: u8, rm: u8 },
    Push { rlist: u8, lr: bool },
    Pop { rlist: u8, pc: bool },
    Beq { offset: u32 },
    Bne { offset: u32 },
    Bcs { offset: u32 },
    Bcc { offset: u32 },
    Bmi { offset: u32 },
    Bpl { offset: u32 },
    Bvs { offset: u32 },
    Bcv { offset: u32 },
    Bhi { offset: u32 },
    Bls { offset: u32 },
    Bge { offset: u32 },
    Blt { offset: u32 },
    Bgt { offset: u32 },
    Ble { offset: u32 },
    B { offset: u32 },
    Bl { offset: u32 },
    Dmb,
    NotImplemented,
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
    flash: [u8; 0x40000],
    sram: [u8; 0x8000],
    tick_count: u64,
}

const PC_INDEX: usize = 15;
const LR_INDEX: u8 = 14;
const SP_INDEX: u8 = 13;

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
            flash: [0xff; 0x40000],
            sram: [0xff; 0x8000],
            tick_count: 0,
        };
        result.load_sample_instructions();
        result
    }

    pub fn dummy(&self) {
        log!(
            "It works! {:?}\nreg: {:?}",
            &self.instructions,
            &self.registers
        );
    }

    pub fn step(&mut self) {
        let instruction = *self.instructions.get(0).unwrap();
        self.execute_instruction(instruction);
        self.increment_pc();
    }

    pub fn debug_instruction(&mut self, instruction: u16) {
        self.execute_instruction(Gamebuino::parse_instruction(instruction, 0xffff));
    }

    pub fn run(&mut self, steps: usize) {
        for _ in 0..steps {
            self.step();
        }
    }

    fn increment_pc(&mut self) {
        self.tick_count += 1;
        self.registers[PC_INDEX] += 1;
    }

    fn push_stack(&mut self, value: u32) {
        self.set_register(SP_INDEX, self.read_register(SP_INDEX) - 4);
        self.write_word(self.read_register(SP_INDEX), value);
    }

    fn pop_stack(&mut self, register: u8) {
        self.set_register(register, self.fetch_word(self.read_register(SP_INDEX)));
        self.set_register(SP_INDEX, self.read_register(SP_INDEX) + 4);
    }

    fn fetch_word(&self, address: u32) -> u32 {
        let addr = address as usize;
        if addr < 0x20000000 {
            let addr = addr % 0x40000;
            self.flash[addr] as u32
                | (self.flash[addr + 1] as u32) << 8
                | (self.flash[addr + 2] as u32) << 16
                | (self.flash[addr + 3] as u32) << 24
        } else if addr < 0x40000000 {
            let addr = (addr - 0x20000000) % 0x8000;
            self.sram[addr] as u32
                | (self.sram[addr + 1] as u32) << 8
                | (self.sram[addr + 2] as u32) << 16
                | (self.sram[addr + 3] as u32) << 24
        } else {
            0
        }
    }

    fn fetch_half_word(&self, address: u32) -> u16 {
        let addr = address as usize;
        if addr < 0x20000000 {
            let addr = addr % 0x40000;
            self.flash[addr] as u16 | (self.flash[addr + 1] as u16) << 8
        } else if addr < 0x40000000 {
            let addr = (addr - 0x20000000) % 0x8000;
            self.sram[addr] as u16 | (self.sram[addr + 1] as u16) << 8
        } else {
            0
        }
    }

    fn fetch_byte(&self, address: u32) -> u8 {
        let addr = address as usize;
        if addr < 0x20000000 {
            let addr = addr % 0x40000;
            self.flash[addr]
        } else if addr < 0x40000000 {
            let addr = (addr - 0x20000000) % 0x8000;
            self.sram[addr]
        } else {
            0
        }
    }

    fn write_word(&mut self, address: u32, value: u32) {
        let addr = address as usize;
        if addr < 0x20000000 {
            // do nothing; not supporting writing to flash
        } else if addr < 0x40000000 {
            let addr = (addr - 0x20000000) % 0x8000;
            self.sram[addr] = (value & 0xff) as u8;
            self.sram[addr + 1] = ((value >> 8) & 0xff) as u8;
            self.sram[addr + 2] = ((value >> 16) & 0xff) as u8;
            self.sram[addr + 3] = ((value >> 24) & 0xff) as u8;
        }
    }

    fn write_half_word(&mut self, address: u32, value: u32) {
        let addr = address as usize;
        if addr < 0x20000000 {
            // do nothing; not supporting writing to flash
        } else if addr < 0x40000000 {
            let addr = (addr - 0x20000000) % 0x8000;
            self.sram[addr] = (value & 0xff) as u8;
            self.sram[addr + 1] = ((value >> 8) & 0xff) as u8;
        }
    }

    fn write_byte(&mut self, address: u32, value: u32) {
        let addr = address as usize;
        if addr < 0x20000000 {
            // do nothing; not supporting writing to flash
        } else if addr < 0x40000000 {
            let addr = (addr - 0x20000000) % 0x8000;
            self.sram[addr] = value as u8;
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

    fn add_and_set_condition(&mut self, n1: u32, n2: u32, carry: u32) -> u32 {
        let (result, overflow1) = n1.overflowing_add(n2);
        let (result, overflow2) = result.overflowing_add(carry);
        self.cond_reg.c = overflow1 | overflow2;
        self.cond_reg.z = result == 0;
        self.cond_reg.n = (result & 0x80000000) != 0;
        self.cond_reg.v = ((n1 & 0x80000000) == (n2 & 0x80000000))
            && ((n1 & 0x80000000) != (result & 0x80000000));
        result
    }

    fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::LslImm { rs, rd, offset } => {
                let original = self.read_register(rs);
                let result = original << offset;
                self.set_register(rd, result);
                self.cond_reg.c = original & (1 << offset) != 0;
                self.set_nz(result);
            }
            Instruction::LslReg { rs, rd } => {
                let offset = self.read_register(rs);
                let original = self.read_register(rd);
                let result = original << offset;
                self.set_register(rd, result);
                self.cond_reg.c = original & (1 << offset) != 0;
                self.set_nz(result);
            }
            Instruction::LsrImm { rs, rd, offset } => {
                let original = self.read_register(rs);
                let result = original >> offset;
                self.set_register(rd, result);
                self.cond_reg.c = original & (1 << (32 - offset)) != 0;
                self.set_nz(result);
            }
            Instruction::LsrReg { rs, rd } => {
                let offset = self.read_register(rs);
                let original = self.read_register(rd);
                let result = original >> offset;
                self.set_register(rd, result);
                self.cond_reg.c = original & (1 << (32 - offset)) != 0;
                self.set_nz(result);
            }
            Instruction::AsrImm { rs, rd, offset } => {
                let original = self.read_register(rs) as i32;
                let result = (original >> offset) as u32;
                self.set_register(rd, result);
                self.cond_reg.c = original & (1 << (offset - 1)) != 0;
                self.set_nz(result);
            }
            Instruction::AsrReg { rs, rd } => {
                let offset = self.read_register(rs);
                let original = self.read_register(rd) as i32;
                let result = (original >> offset) as u32;
                self.set_register(rd, result);
                self.cond_reg.c = original & (1 << (offset - 1)) != 0;
                self.set_nz(result);
            }
            Instruction::AddReg { rs, rd, rn } => {
                let result =
                    self.add_and_set_condition(self.read_register(rs), self.read_register(rn), 0);
                self.set_register(rd, result);
            }
            Instruction::AddImm { rs, rd, offset } => {
                let result = self.add_and_set_condition(self.read_register(rs), offset as u32, 0);
                self.set_register(rd, result);
            }
            Instruction::AddSp { rd, offset } => {
                self.set_register(rd, self.read_register(SP_INDEX) + offset);
            }
            Instruction::AddPc { rd, offset } => {
                self.set_register(rd, (self.read_register(PC_INDEX as u8) & !0b11) + offset);
            }
            Instruction::Adc { rs, rd } => {
                let result = self.add_and_set_condition(
                    self.read_register(rs),
                    self.read_register(rd),
                    if self.cond_reg.c { 1 } else { 0 },
                );
                self.set_register(rd, result);
            }
            Instruction::SubReg { rs, rd, rn } => {
                let result =
                    self.add_and_set_condition(self.read_register(rs), !self.read_register(rn), 1);
                self.set_register(rd, result);
            }
            Instruction::SubImm { rs, rd, offset } => {
                let result =
                    self.add_and_set_condition(self.read_register(rs), !(offset as u32), 1);
                self.set_register(rd, result);
            }
            Instruction::Sbc { rs, rd } => {
                let result = self.add_and_set_condition(
                    self.read_register(rs),
                    !self.read_register(rd),
                    if self.cond_reg.c { 1 } else { 0 },
                );
                self.set_register(rd, result);
            }
            Instruction::Neg { rs, rd } => {
                let result = self.add_and_set_condition(0, !self.read_register(rs), 1);
                self.set_register(rd, result);
            }
            Instruction::Mul { rs, rd } => {
                let (result, overflow) = self
                    .read_register(rd)
                    .overflowing_mul(self.read_register(rs));
                self.set_register(rd, result);
                self.set_nz(result);
                self.cond_reg.c = overflow;
            }
            Instruction::MovImm { rd, offset } => {
                self.set_register(rd, offset as u32);
                self.set_nz(offset as u32);
            }
            Instruction::MovReg { rs, rd } => {
                self.set_register(rd, self.read_register(rs));
                if rd as usize == PC_INDEX {
                    self.increment_pc();
                }
            }
            Instruction::Mvn { rs, rd } => {
                let result = !self.read_register(rs);
                self.set_register(rd, result);
                self.set_nz(result);
            }
            Instruction::CmpImm { rd, offset } => {
                self.add_and_set_condition(self.read_register(rd), !(offset as u32), 1);
            }
            Instruction::CmpReg { rd, rs } => {
                self.add_and_set_condition(self.read_register(rd), !self.read_register(rs), 1);
            }
            Instruction::Cmn { rd, rs } => {
                self.add_and_set_condition(self.read_register(rd), self.read_register(rs), 0);
            }
            Instruction::Tst { rs, rd } => {
                let result = self.read_register(rd) & self.read_register(rs);
                self.set_nz(result);
            }
            Instruction::And { rs, rd } => {
                let result = self.read_register(rd) & self.read_register(rs);
                self.set_register(rd, result);
                self.set_nz(result);
            }
            Instruction::Bic { rs, rd } => {
                let result = self.read_register(rd) & !self.read_register(rs);
                self.set_register(rd, result);
                self.set_nz(result);
            }
            Instruction::Eor { rs, rd } => {
                let result = self.read_register(rd) ^ self.read_register(rs);
                self.set_register(rd, result);
                self.set_nz(result);
            }
            Instruction::Oor { rs, rd } => {
                let result = self.read_register(rd) | self.read_register(rs);
                self.set_register(rd, result);
                self.set_nz(result);
            }
            Instruction::Bx { rs } => {
                self.set_register(PC_INDEX as u8, self.read_register(rs) & !1);
                self.increment_pc();
            }
            Instruction::Blx { rm } => {
                self.set_register(LR_INDEX, (self.read_register(PC_INDEX as u8) - 2) | 1);
                self.set_register(PC_INDEX as u8, self.read_register(rm) & !1);
                self.increment_pc();
            }
            Instruction::LdrPc {
                rd,
                immediate_value,
            } => {
                let result =
                    self.fetch_word((self.read_register(PC_INDEX as u8) & !0b11) + immediate_value);
                self.set_register(rd, result);
            }
            Instruction::LdrReg { rb, ro, rd } => {
                let result = self.fetch_word(self.read_register(rb) + self.read_register(ro));
                self.set_register(rd, result);
            }
            Instruction::LdrbReg { rb, ro, rd } => {
                let result = self.fetch_byte(self.read_register(rb) + self.read_register(ro));
                self.set_register(rd, result as u32);
            }
            Instruction::LdrImm { rb, offset, rd } => {
                self.set_register(rd, self.fetch_word(self.read_register(rb) + offset));
            }
            Instruction::LdrbImm { rb, offset, rd } => {
                self.set_register(rd, self.fetch_byte(self.read_register(rb) + offset) as u32);
            }
            Instruction::Ldsb { rb, ro, rd } => {
                let mut result = self.fetch_byte(self.read_register(rb) + self.read_register(ro));
                if result & 0x80 != 0 {
                    result |= !0xff;
                }
                self.set_register(rd, result as u32);
            }
            Instruction::LdrhReg { rb, ro, rd } => {
                let result = self.fetch_half_word(self.read_register(rb) + self.read_register(ro));
                self.set_register(rd, result as u32);
            }
            Instruction::LdrhImm { rb, offset, rd } => {
                self.set_register(
                    rd,
                    0xffff & (self.fetch_half_word(self.read_register(rb) + offset) as u32),
                );
            }
            Instruction::Ldsh { rb, ro, rd } => {
                let mut result =
                    self.fetch_half_word(self.read_register(rb) + self.read_register(ro));
                if result & 0x8000 != 0 {
                    result |= !0xffff;
                }
                self.set_register(rd, result as u32);
            }
            Instruction::Ldmia { rb, rlist } => {
                let mut addr = self.read_register(rb);
                for i in 0..8 {
                    if rlist & (1 << i) != 0 {
                        self.set_register(i, self.fetch_word(addr));
                        addr += 4;
                    }
                }
                self.set_register(rb, addr);
            }
            Instruction::StrReg { rb, ro, rd } => {
                let address = self.read_register(rb) + self.read_register(ro);
                self.write_word(address, self.read_register(rd));
            }
            Instruction::StrbReg { rb, ro, rd } => {
                let address = self.read_register(rb) + self.read_register(ro);
                self.write_byte(address, self.read_register(rd));
            }
            Instruction::StrImm { rb, offset, rd } => {
                self.write_word(self.read_register(rb) + offset, self.read_register(rd))
            }
            Instruction::StrbImm { rb, offset, rd } => {
                self.write_byte(self.read_register(rb) + offset, self.read_register(rd))
            }
            Instruction::StrhReg { rb, ro, rd } => {
                let address = self.read_register(rb) + self.read_register(ro);
                self.write_half_word(address, self.read_register(rd));
            }
            Instruction::StrhImm { rb, offset, rd } => {
                self.write_half_word(self.read_register(rb) + offset, self.read_register(rd))
            }
            Instruction::Stmia { rb, rlist } => {
                let mut addr = self.read_register(rb);
                for i in 0..8 {
                    if rlist & (1 << i) != 0 {
                        self.write_word(addr, self.read_register(i));
                        addr += 4;
                    }
                }
                self.set_register(rb, addr);
            }
            Instruction::Sxth { rd, rm } => {
                let mut result = self.read_register(rm) & 0xffff;
                if (result & 0x8000) != 0 {
                    result = (!0xffff) | result;
                }
                self.set_register(rd, result);
            }
            Instruction::Sxtb { rd, rm } => {
                let mut result = self.read_register(rm) & 0xff;
                if (result & 0x80) != 0 {
                    result = (!0xff) | result;
                }
                self.set_register(rd, result);
            }
            Instruction::Uxth { rd, rm } => {
                let result = self.read_register(rm) & 0xffff;
                self.set_register(rd, result);
            }
            Instruction::Uxtb { rd, rm } => {
                let result = self.read_register(rm) & 0xff;
                self.set_register(rd, result);
            }
            Instruction::Rev { rd, rm } => {
                let rm_val = self.read_register(rm);
                let result = ((0xff000000 & rm_val) >> 24)
                    | ((0x00ff0000 & rm_val) >> 8)
                    | ((0x0000ff00 & rm_val) << 8)
                    | ((0x000000ff & rm_val) << 24);
                self.set_register(rd, result);
            }
            Instruction::Rev16 { rd, rm } => {
                let rm_val = self.read_register(rm);
                let result = ((0xff00ff00 & rm_val) >> 8) | ((0x00ff00ff & rm_val) << 8);
                self.set_register(rd, result);
            }
            Instruction::Push { rlist, lr } => {
                if lr {
                    self.push_stack(self.read_register(LR_INDEX));
                }
                for i in (0..8).rev() {
                    if rlist & (1 << i) != 0 {
                        self.push_stack(self.read_register(i));
                    }
                }
            }
            Instruction::Pop { rlist, pc } => {
                for i in 0..8 {
                    if rlist & (1 << i) != 0 {
                        self.pop_stack(i);
                    }
                }
                if pc {
                    self.pop_stack(PC_INDEX as u8);
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) & !1);
                    self.increment_pc();
                }
            }
            Instruction::Beq { offset } => {
                if self.cond_reg.z {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bne { offset } => {
                if !self.cond_reg.z {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bcs { offset } => {
                if self.cond_reg.c {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bcc { offset } => {
                if !self.cond_reg.c {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bmi { offset } => {
                if self.cond_reg.n {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bpl { offset } => {
                if !self.cond_reg.n {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bvs { offset } => {
                if self.cond_reg.v {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bcv { offset } => {
                if !self.cond_reg.v {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bhi { offset } => {
                if self.cond_reg.c && !self.cond_reg.z {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bls { offset } => {
                if !self.cond_reg.c || self.cond_reg.z {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bge { offset } => {
                if self.cond_reg.n == self.cond_reg.v {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Blt { offset } => {
                if self.cond_reg.n != self.cond_reg.v {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bgt { offset } => {
                if !self.cond_reg.z && (self.cond_reg.n == self.cond_reg.v) {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Ble { offset } => {
                if self.cond_reg.z || (self.cond_reg.n != self.cond_reg.v) {
                    self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                    self.increment_pc();
                }
            }
            Instruction::B { offset } => {
                self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                self.increment_pc();
            }
            Instruction::Bl { offset } => {
                self.set_register(PC_INDEX as u8, self.read_register(PC_INDEX as u8) + offset);
                self.set_register(LR_INDEX, self.read_register(PC_INDEX as u8) - 1);
                self.increment_pc();
                self.increment_pc();
            }
            Instruction::Dmb => {
                self.increment_pc();
            }
            Instruction::NotImplemented => {}
        }
    }

    fn load_sample_instructions(&mut self) {
        let data: Vec<u16> = vec![
            0b0000000100001010,
            0b0001000100001011,
            0b0001110001000000,
            0b0010000000000000,
            0b0100110000001000,
        ];
        for instruction in data
            .iter()
            .map(|i| Gamebuino::parse_instruction(*i, 0xffff))
        {
            self.instructions.push(instruction);
        }
    }

    fn parse_instruction(instruction: u16, following_instruction: u16) -> Instruction {
        if instruction & 0b1110000000000000 == 0b0000000000000000 {
            let rs = ((instruction & 0b0000000000111000) >> 3) as u8;
            let rd = ((instruction & 0b0000000000000111) >> 0) as u8;
            if (instruction & 0b0001100000000000) != 0b0001100000000000 {
                let opcode = (instruction & 0b0001100000000000) >> 11;
                let offset = ((instruction & 0b0000011111000000) >> 6) as u8;
                match opcode {
                    0 => Instruction::LslImm { rs, rd, offset },
                    1 => Instruction::LsrImm { rs, rd, offset },
                    2 => Instruction::AsrImm { rs, rd, offset },
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
        } else if (instruction & 0b1111110000000000) == 0b0100000000000000 {
            let opcode = (instruction & 0b0000001111000000) >> 6;
            let rs = ((instruction & 0b0000000000111000) >> 3) as u8;
            let rd = (instruction & 0b0000000000000111) as u8;
            match opcode {
                0b0000 => Instruction::And { rd, rs },
                0b0001 => Instruction::Eor { rd, rs },
                0b0010 => Instruction::LslReg { rd, rs },
                0b0011 => Instruction::LsrReg { rd, rs },
                0b0100 => Instruction::AsrReg { rd, rs },
                0b0101 => Instruction::Adc { rd, rs },
                0b0110 => Instruction::Sbc { rd, rs },
                // TODO ror
                0b1000 => Instruction::Tst { rd, rs },
                0b1001 => Instruction::Neg { rd, rs },
                0b1010 => Instruction::CmpReg { rd, rs },
                0b1011 => Instruction::Cmn { rd, rs },
                0b1100 => Instruction::Oor { rd, rs },
                0b1101 => Instruction::Mul { rd, rs },
                0b1110 => Instruction::Bic { rd, rs },
                0b1111 => Instruction::Mvn { rd, rs },
                _ => panic!("Unexpected opcode {}", opcode),
            }
        } else if (instruction & 0b1111110000000000) == 0b0100010000000000 {
            let op_h1_h2 = (instruction & 0b0000001111000000) >> 6;
            let rs_hs = ((instruction & 0b0000000000111000) >> 3) as u8;
            let rd_hd = (instruction & 0b0000000000000111) as u8;
            let rm = ((instruction & 0b0000000001111000) >> 3) as u8;
            match op_h1_h2 {
                0b0001 => Instruction::AddReg {
                    rd: rd_hd,
                    rs: rs_hs + 8,
                    rn: rd_hd,
                },
                0b0010 => Instruction::AddReg {
                    rd: rd_hd + 8,
                    rs: rs_hs,
                    rn: rd_hd + 8,
                },
                0b0011 => Instruction::AddReg {
                    rd: rd_hd + 8,
                    rs: rs_hs + 8,
                    rn: rd_hd + 8,
                },
                0b0101 => Instruction::CmpReg {
                    rs: rs_hs + 8,
                    rd: rd_hd,
                },
                0b0110 => Instruction::CmpReg {
                    rs: rs_hs,
                    rd: rd_hd + 8,
                },
                0b0111 => Instruction::CmpReg {
                    rs: rs_hs + 8,
                    rd: rd_hd + 8,
                },
                0b1000 => Instruction::MovReg {
                    rs: rs_hs,
                    rd: rd_hd,
                },
                0b1001 => Instruction::MovReg {
                    rs: rs_hs + 8,
                    rd: rd_hd,
                },
                0b1010 => Instruction::MovReg {
                    rs: rs_hs,
                    rd: rd_hd + 8,
                },
                0b1011 => Instruction::MovReg {
                    rs: rs_hs + 8,
                    rd: rd_hd + 8,
                },
                0b1100 => Instruction::Bx { rs: rs_hs },
                0b1101 => Instruction::Bx { rs: rs_hs + 8 },
                0b1110 | 0b1111 => Instruction::Blx { rm },
                _ => panic!("Unexpected opcode {}", op_h1_h2),
            }
        } else if (instruction & 0b1111100000000000) == 0b0100100000000000 {
            let rd = ((instruction & 0b0000011100000000) >> 8) as u8;
            let immediate_value = ((instruction & 0xff) << 2) as u32;
            Instruction::LdrPc {
                rd,
                immediate_value,
            }
        } else if (instruction & 0b1111001000000000) == 0b0101000000000000 {
            let lb = (instruction & 0b0000110000000000) >> 10;
            let ro = ((instruction & 0b0000000111000000) >> 6) as u8;
            let rb = ((instruction & 0b0000000000111000) >> 3) as u8;
            let rd = (instruction & 0b0000000000000111) as u8;
            match lb {
                0b00 => Instruction::StrReg { rb, ro, rd },
                0b01 => Instruction::StrbReg { rb, ro, rd },
                0b10 => Instruction::LdrReg { rb, ro, rd },
                0b11 => Instruction::LdrbReg { rb, ro, rd },
                _ => panic!("Unexpected lb {}", lb),
            }
        } else if (instruction & 0b1111001000000000) == 0b0101001000000000 {
            let hs = (instruction & 0b0000110000000000) >> 10;
            let ro = ((instruction & 0b0000000111000000) >> 6) as u8;
            let rb = ((instruction & 0b0000000000111000) >> 3) as u8;
            let rd = (instruction & 0b0000000000000111) as u8;
            match hs {
                0b00 => Instruction::StrhReg { rb, ro, rd },
                0b01 => Instruction::Ldsb { rb, ro, rd },
                0b10 => Instruction::LdrhReg { rb, ro, rd },
                0b11 => Instruction::Ldsh { rb, ro, rd },
                _ => panic!("Unexpected hs {}", hs),
            }
        } else if (instruction & 0b1110000000000000) == 0b0110000000000000 {
            let bl = (instruction & 0b0001100000000000) >> 11;
            let offset = ((instruction & 0b0000011111000000) >> 6) as u32;
            let rb = ((instruction & 0b0000000000111000) >> 3) as u8;
            let rd = (instruction & 0b0000000000000111) as u8;
            match bl {
                0b00 => Instruction::StrImm {
                    rb,
                    offset: offset << 2,
                    rd,
                },
                0b01 => Instruction::LdrImm {
                    rb,
                    offset: offset << 2,
                    rd,
                },
                0b10 => Instruction::StrbImm {
                    rb,
                    offset: offset,
                    rd,
                },
                0b11 => Instruction::LdrbImm {
                    rb,
                    offset: offset,
                    rd,
                },
                _ => panic!("Unexpected bl {}", bl),
            }
        } else if (instruction & 0b1111000000000000) == 0b1000000000000000 {
            let l = (instruction & 0b0000100000000000) != 0;
            let offset = ((instruction & 0b0000011111000000) >> 6) as u32;
            let rb = ((instruction & 0b0000000000111000) >> 3) as u8;
            let rd = (instruction & 0b0000000000000111) as u8;
            if l {
                Instruction::LdrhImm {
                    rb,
                    offset: offset << 1,
                    rd,
                }
            } else {
                Instruction::StrhImm {
                    rb,
                    offset: offset << 1,
                    rd,
                }
            }
        } else if (instruction & 0b1111000000000000) == 0b1001000000000000 {
            let l = (instruction & 0b0000100000000000) != 0;
            let rd = ((instruction & 0b0000011100000000) >> 8) as u8;
            let offset = ((instruction & 0b0000000011111111) << 2) as u32;
            if l {
                Instruction::LdrImm {
                    rb: SP_INDEX,
                    offset,
                    rd,
                }
            } else {
                Instruction::StrImm {
                    rb: SP_INDEX,
                    offset,
                    rd,
                }
            }
        } else if (instruction & 0b1111000000000000) == 0b1010000000000000 {
            let sp = (instruction & 0b0000100000000000) != 0;
            let rd = ((instruction & 0b0000011100000000) >> 8) as u8;
            let offset = ((instruction & 0b0000000011111111) << 2) as u32;
            if sp {
                Instruction::AddSp { offset, rd }
            } else {
                Instruction::AddPc { offset, rd }
            }
        } else if (instruction & 0b1111111100000000) == 0b1011000000000000 {
            let negative = (instruction & 0b0000000010000000) != 0;
            let offset =
                if negative { -1 } else { 1 } * ((instruction & 0b0000000001111111) << 2) as i32;
            Instruction::AddSp {
                offset: offset as u32,
                rd: SP_INDEX,
            }
        } else if (instruction & 0b1111111100000000) == 0b1011001000000000 {
            let opcode = (instruction & 0b0000000011000000) >> 6;
            let rm = ((instruction & 0b0000000000111000) >> 3) as u8;
            let rd = (instruction & 0b0000000000000111) as u8;
            match opcode {
                0b00 => Instruction::Sxth { rd, rm },
                0b01 => Instruction::Sxtb { rd, rm },
                0b10 => Instruction::Uxth { rd, rm },
                0b11 => Instruction::Uxtb { rd, rm },
                _ => panic!("Unexpected opcode {}", opcode),
            }
        } else if (instruction & 0b1111111100000000) == 0b1011101000000000 {
            let opcode = (instruction & 0b0000000011000000) >> 6;
            let rm = ((instruction & 0b0000000000111000) >> 3) as u8;
            let rd = (instruction & 0b0000000000000111) as u8;
            match opcode {
                // TODO revsh
                0b00 => Instruction::Rev { rd, rm },
                0b01 => Instruction::Rev16 { rd, rm },
                _ => panic!("Unexpected opcode {}", opcode),
            }
        } else if (instruction & 0b1111111111101000) == 0b1011011001100000 {
            // cps
            Instruction::NotImplemented
        } else if (instruction & 0b1111011000000000) == 0b1011010000000000 {
            let l = (instruction & 0b0000100000000000) != 0;
            let r = (instruction & 0b0000000100000000) != 0;
            let rlist = 0xff & instruction as u8;
            if !l {
                Instruction::Push { rlist, lr: r }
            } else {
                Instruction::Pop { rlist, pc: r }
            }
        } else if (instruction & 0b1111000000000000) == 0b1100000000000000 {
            let l = (instruction & 0b0000100000000000) != 0;
            let rb = ((instruction & 0b0000011100000000) >> 8) as u8;
            let rlist = 0xff & instruction as u8;
            if l {
                Instruction::Ldmia { rb, rlist }
            } else {
                Instruction::Stmia { rb, rlist }
            }
        } else if (instruction & 0b1111000000000000) == 0b1101000000000000 {
            let condition = (instruction & 0b0000111100000000) >> 8;
            let mut offset = (instruction & 0b0000000011111111) as u32;
            // Handle negative
            if offset & 0b10000000 != 0 {
                offset |= !0b11111111;
            }
            offset = offset << 1;
            match condition {
                0b0000 => Instruction::Beq { offset },
                0b0001 => Instruction::Bne { offset },
                0b0010 => Instruction::Bcs { offset },
                0b0011 => Instruction::Bcc { offset },
                0b0100 => Instruction::Bmi { offset },
                0b0101 => Instruction::Bpl { offset },
                0b0110 => Instruction::Bvs { offset },
                0b0111 => Instruction::Bcv { offset },
                0b1000 => Instruction::Bhi { offset },
                0b1001 => Instruction::Bls { offset },
                0b1010 => Instruction::Bge { offset },
                0b1011 => Instruction::Blt { offset },
                0b1100 => Instruction::Bgt { offset },
                0b1101 => Instruction::Ble { offset },
                _ => panic!("Unexpected condition {}", condition),
            }
        } else if (instruction & 0b1111100000000000) == 0b1110000000000000 {
            let mut offset = (instruction & 0b0000000011111111) as u32;
            // Handle negative
            if offset & 0b10000000 != 0 {
                offset |= !0b11111111;
            }
            offset = offset << 1;
            Instruction::B { offset }
        } else if (instruction & 0b1111100000000000) == 0b1111000000000000
            && (following_instruction & 0b1111100000000000) == 0b1111100000000000
        {
            let mut offset1 = (instruction & 0b0000011111111111) as u32;
            if (offset1 & 0b0000010000000000) != 0 {
                offset1 |= !0b0000011111111111;
            }
            offset1 = offset1 << 12;
            let offset2 = ((following_instruction & 0b0000011111111111) << 1) as u32;
            Instruction::Bl {
                offset: offset1 + offset2,
            }
        } else if (instruction & 0b1111111111100000) == 0b1111001111100000
            && (following_instruction & 0b1101000000000000) == 0b1000000000000000
        {
            // mrs
            Instruction::NotImplemented
        } else if instruction == 0xF3BF {
            Instruction::Dmb
        }
        // TODO swi
        else {
            Instruction::NotImplemented
        }
    }
}
