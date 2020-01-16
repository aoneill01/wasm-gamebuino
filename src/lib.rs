mod input_output;
mod instruction;
mod register;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern crate js_sys;
extern crate web_sys;

// macro_rules! log {
//     ( $( $t:tt )* ) => {
//         web_sys::console::log_1(&format!( $( $t )* ).into());
//     }
// }

use input_output::{Buttons, St7735};
use instruction::Instruction;
use register::{CondRegister, DmacRegisters, Peripheral, PortRegisters, SercomRegisters};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Gamebuino {
    instructions: Vec<Instruction>,
    cond_reg: CondRegister,
    registers: [u32; 16],
    flash: [u8; 0x40000],
    sram: [u8; 0x8000],
    tick_count: u64,
    program_offset: u32,
    systic_vector: u32,
    systick_trigger: isize,
    dmac_vector: u32,
    dmac_interrupt: bool,
    dmac_registers: DmacRegisters,
    porta_registers: PortRegisters,
    portb_registers: PortRegisters,
    sercom4: SercomRegisters,
    sercom5: SercomRegisters,
    screen: St7735,
    buttons: Buttons,
    // log: bool,
}

const PC_INDEX: u8 = 15;
const LR_INDEX: u8 = 14;
const SP_INDEX: u8 = 13;
const SYSTICK_COUNTDOWN: isize = 20000;

#[wasm_bindgen]
impl Gamebuino {
    pub fn new() -> Gamebuino {
        crate::utils::set_panic_hook();
        Gamebuino {
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
            program_offset: 0,
            systic_vector: 0,
            systick_trigger: SYSTICK_COUNTDOWN,
            dmac_vector: 0,
            dmac_interrupt: false,
            dmac_registers: DmacRegisters::new(),
            porta_registers: PortRegisters::new(),
            portb_registers: PortRegisters::new(),
            sercom4: SercomRegisters::new(),
            sercom5: SercomRegisters::new(),
            screen: St7735::new(),
            buttons: Buttons::new(),
            // log: false,
        }
    }

    // pub fn enable_logging(&mut self) {
    //     self.log = true;
    // }

    pub fn get_register(&self, i: usize) -> u32 {
        self.registers[i]
    }

    pub fn get_tick_count(&self) -> u32 {
        self.tick_count as u32
    }

    pub fn load_program(&mut self, contents: &[u8], offset: u32) {
        self.program_offset = offset;
        self.instructions.clear();

        for (i, val) in contents.iter().enumerate() {
            self.flash[i + offset as usize] = *val;
        }

        let mut skip_next = false;
        for i in 0..(contents.len() / 2) as u32 {
            if skip_next {
                skip_next = false;
                continue;
            }

            let instruction = self.fetch_half_word(offset + i * 2);
            let following_instruction = self.fetch_half_word(offset + (i + 1) * 2);
            let parsed = instruction::parse_instruction(instruction, following_instruction);
            self.instructions.push(parsed);

            if let Instruction::Bl {
                offset1,
                offset2,
                first: _,
            } = parsed
            {
                skip_next = true;
                self.instructions.push(Instruction::Bl {
                    offset1,
                    offset2,
                    first: false,
                });
            }
        }

        self.reset();
    }

    fn reset(&mut self) {
        self.set_register(SP_INDEX, self.fetch_word(0x0000 + self.program_offset));
        self.set_register(LR_INDEX, 0xffffffff);
        self.set_register(PC_INDEX, self.fetch_word(0x0004 + self.program_offset) & !1);
        self.increment_pc();
        self.systic_vector = self.fetch_word(0x003C + self.program_offset) & !1;
        self.dmac_vector = self.fetch_word(0x0058 + self.program_offset) & !1;
    }

    pub fn step(&mut self) {
        if self.dmac_interrupt {
            self.dmac_interrupt = false;
            self.push_stack(self.cond_reg.to_word());
            self.push_stack(self.read_register(PC_INDEX));
            self.push_stack(self.read_register(LR_INDEX));
            self.push_stack(self.read_register(12));
            self.push_stack(self.read_register(3));
            self.push_stack(self.read_register(2));
            self.push_stack(self.read_register(1));
            self.push_stack(self.read_register(0));
            self.set_register(PC_INDEX, self.dmac_vector);
            self.set_register(LR_INDEX, 0xfffffff9);
            self.increment_pc();
        } else if self.systick_trigger <= 0 {
            self.systick_trigger = SYSTICK_COUNTDOWN;
            self.push_stack(self.cond_reg.to_word());
            self.push_stack(self.read_register(PC_INDEX));
            self.push_stack(self.read_register(LR_INDEX));
            self.push_stack(self.read_register(12));
            self.push_stack(self.read_register(3));
            self.push_stack(self.read_register(2));
            self.push_stack(self.read_register(1));
            self.push_stack(self.read_register(0));
            self.set_register(PC_INDEX, self.systic_vector);
            self.set_register(LR_INDEX, 0xfffffff9);
            self.increment_pc();
        }

        let mut addr = self.read_register(PC_INDEX) - 2;

        while addr == 0xfffffff8 {
            self.pop_stack(0);
            self.pop_stack(1);
            self.pop_stack(2);
            self.pop_stack(3);
            self.pop_stack(12);
            self.pop_stack(LR_INDEX);
            self.pop_stack(PC_INDEX);
            let cnvz = self.fetch_word(self.read_register(SP_INDEX));
            self.cond_reg.from_word(cnvz);
            self.set_register(SP_INDEX, self.read_register(SP_INDEX) + 4);
            addr = self.read_register(PC_INDEX) - 2;
        }

        let instruction = *self
            .instructions
            .get(((addr - self.program_offset) >> 1) as usize)
            .unwrap();
        // if self.log {
        //     log!(
        //         "addr: {:04x}, instr: {:016b}, {:?}",
        //         addr,
        //         self.fetch_half_word(addr),
        //         instruction
        //     );
        // }
        self.increment_pc();
        self.execute_instruction(instruction);
        // if self.log {
        //     log!("flags: {:?}\nregs: {:?}", self.cond_reg, self.registers);
        // }
    }

    pub fn run(&mut self, steps: usize, button_data: u8) {
        self.buttons.button_data = button_data;

        let goal = self.tick_count + steps as u64;
        while self.tick_count < goal {
            self.step();
        }
    }

    pub fn screen_data(&self) -> *const u32 {
        self.screen.screen_data()
    }

    fn increment_pc(&mut self) {
        self.tick_count += 1;
        self.systick_trigger -= 1;
        self.registers[PC_INDEX as usize] += 2;
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
            if addr >= 0x40000 {
                return 0;
            }
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
        } else if addr < 0x60000000 {
            let addr = addr as u32;
            match addr {
                0x4000080c => 0b11010010, // hack for PCLKSR
                DmacRegisters::DMAC_START_ADDR..=DmacRegisters::DMAC_END_ADDR => self
                    .dmac_registers
                    .handle_read_word(addr - DmacRegisters::DMAC_START_ADDR),
                PortRegisters::PORTA_START_ADDR..=PortRegisters::PORTA_END_ADDR => self
                    .porta_registers
                    .handle_read_word(addr - PortRegisters::PORTA_START_ADDR),
                PortRegisters::PORTB_START_ADDR..=PortRegisters::PORTB_END_ADDR => self
                    .portb_registers
                    .handle_read_word(addr - PortRegisters::PORTB_START_ADDR),
                SercomRegisters::SERCOM4_START_ADDR..=SercomRegisters::SERCOM4_END_ADDR => self
                    .sercom4
                    .handle_read_word(addr - SercomRegisters::SERCOM4_START_ADDR),
                SercomRegisters::SERCOM5_START_ADDR..=SercomRegisters::SERCOM5_END_ADDR => self
                    .sercom5
                    .handle_read_word(addr - SercomRegisters::SERCOM5_START_ADDR),
                _ => 0,
            }
        } else {
            0
        }
    }

    fn fetch_half_word(&self, address: u32) -> u16 {
        let addr = address as usize;
        if addr < 0x20000000 {
            if addr >= 0x40000 {
                return 0;
            }
            self.flash[addr] as u16 | (self.flash[addr + 1] as u16) << 8
        } else if addr < 0x40000000 {
            let addr = (addr - 0x20000000) % 0x8000;
            self.sram[addr] as u16 | (self.sram[addr + 1] as u16) << 8
        } else if addr < 0x60000000 {
            match addr {
                // hack for ADC RESULT
                0x4200401A => (js_sys::Math::random() * (0xffff as f64)).floor() as u16,
                _ => 0,
            }
        } else {
            0
        }
    }

    fn fetch_byte(&self, address: u32) -> u8 {
        let addr = address as usize;
        if addr < 0x20000000 {
            if addr >= 0x40000 {
                return 0;
            }
            self.flash[addr]
        } else if addr < 0x40000000 {
            let addr = (addr - 0x20000000) % 0x8000;
            self.sram[addr]
        } else if addr < 0x60000000 {
            let addr = addr as u32;
            match addr {
                0x42004018 => 1, // hack for ADC INTFLAG RESRDY
                DmacRegisters::DMAC_START_ADDR..=DmacRegisters::DMAC_END_ADDR => self
                    .dmac_registers
                    .handle_read_byte(addr - DmacRegisters::DMAC_START_ADDR),
                SercomRegisters::SERCOM4_START_ADDR..=SercomRegisters::SERCOM4_END_ADDR => self
                    .sercom4
                    .handle_read_byte(addr - SercomRegisters::SERCOM4_START_ADDR),
                SercomRegisters::SERCOM5_START_ADDR..=SercomRegisters::SERCOM5_END_ADDR => self
                    .sercom5
                    .handle_read_byte(addr - SercomRegisters::SERCOM5_START_ADDR),
                _ => 0,
            }
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
        } else if addr < 0x60000000 {
            let addr = addr as u32;
            match addr {
                DmacRegisters::DMAC_START_ADDR..=DmacRegisters::DMAC_END_ADDR => {
                    let mut copied = self.dmac_registers;
                    copied.handle_write_word(addr - DmacRegisters::DMAC_START_ADDR, value, self);
                    self.dmac_registers = copied;
                }
                PortRegisters::PORTA_START_ADDR..=PortRegisters::PORTA_END_ADDR => {
                    let mut copied = self.porta_registers;
                    copied.handle_write_word(addr - PortRegisters::PORTA_START_ADDR, value, self);
                    self.porta_registers = copied;
                    // TODO port listeners
                }
                PortRegisters::PORTB_START_ADDR..=PortRegisters::PORTB_END_ADDR => {
                    let mut copied = self.portb_registers;
                    copied.handle_write_word(addr - PortRegisters::PORTB_START_ADDR, value, self);
                    self.portb_registers = copied;
                    // TODO port listeners
                }
                SercomRegisters::SERCOM4_START_ADDR..=SercomRegisters::SERCOM4_END_ADDR => {
                    let mut copied = self.sercom4;
                    copied.handle_write_word(
                        addr - SercomRegisters::SERCOM4_START_ADDR,
                        value,
                        self,
                    );
                    self.sercom4 = copied;
                    if let Some(value) = self.sercom4.sent {
                        self.screen.byte_received(
                            value,
                            &self.porta_registers,
                            &self.portb_registers,
                        );
                        self.buttons
                            .byte_received(value, &self.portb_registers, &mut self.sercom4);
                    }
                    self.sercom4.sent = None;
                }
                SercomRegisters::SERCOM5_START_ADDR..=SercomRegisters::SERCOM5_END_ADDR => {
                    let mut copied = self.sercom5;
                    copied.handle_write_word(
                        addr - SercomRegisters::SERCOM5_START_ADDR,
                        value,
                        self,
                    );
                    self.sercom5 = copied;
                    // TODO sercom listeners
                    self.sercom5.sent = None;
                }
                _ => {}
            }
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
        } else if addr < 0x60000000 {
            let addr = addr as u32;
            match addr {
                DmacRegisters::DMAC_START_ADDR..=DmacRegisters::DMAC_END_ADDR => {
                    let mut copied = self.dmac_registers;
                    copied.handle_write_byte(
                        addr - DmacRegisters::DMAC_START_ADDR,
                        value as u8,
                        self,
                    );
                    self.dmac_registers = copied;
                }
                SercomRegisters::SERCOM4_START_ADDR..=SercomRegisters::SERCOM4_END_ADDR => {
                    let mut copied = self.sercom4;
                    copied.handle_write_byte(
                        addr - SercomRegisters::SERCOM4_START_ADDR,
                        value as u8,
                        self,
                    );
                    self.sercom4 = copied;
                    if let Some(value) = self.sercom4.sent {
                        self.screen.byte_received(
                            value,
                            &self.porta_registers,
                            &self.portb_registers,
                        );
                        self.buttons
                            .byte_received(value, &self.portb_registers, &mut self.sercom4);
                    }
                    self.sercom4.sent = None;
                }
                SercomRegisters::SERCOM5_START_ADDR..=SercomRegisters::SERCOM5_END_ADDR => {
                    let mut copied = self.sercom5;
                    copied.handle_write_byte(
                        addr - SercomRegisters::SERCOM5_START_ADDR,
                        value as u8,
                        self,
                    );
                    self.sercom5 = copied;
                    // TODO sercom listeners
                    self.sercom5.sent = None;
                }
                _ => {}
            }
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

    fn dmac_interrupt(&mut self) {
        self.dmac_interrupt = true;
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
                self.set_register(rd, (self.read_register(PC_INDEX) & !0b11) + offset);
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
                    self.read_register(rd),
                    !self.read_register(rs),
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
                if rd == PC_INDEX {
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
                self.set_register(PC_INDEX, self.read_register(rs) & !1);
                self.increment_pc();
            }
            Instruction::Blx { rm } => {
                self.set_register(LR_INDEX, (self.read_register(PC_INDEX) - 2) | 1);
                self.set_register(PC_INDEX, self.read_register(rm) & !1);
                self.increment_pc();
            }
            Instruction::LdrPc {
                rd,
                immediate_value,
            } => {
                let result =
                    self.fetch_word((self.read_register(PC_INDEX) & !0b11) + immediate_value);
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
                let mut result =
                    self.fetch_byte(self.read_register(rb) + self.read_register(ro)) as u32;
                if result & 0x80 != 0 {
                    result |= !0xff;
                }
                self.set_register(rd, result);
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
                    self.fetch_half_word(self.read_register(rb) + self.read_register(ro)) as u32;
                if result & 0x8000 != 0 {
                    result |= !0xffff;
                }
                self.set_register(rd, result);
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
                    self.pop_stack(PC_INDEX);
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) & !1);
                    self.increment_pc();
                }
            }
            Instruction::Beq { offset } => {
                if self.cond_reg.z {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bne { offset } => {
                if !self.cond_reg.z {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bcs { offset } => {
                if self.cond_reg.c {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bcc { offset } => {
                if !self.cond_reg.c {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bmi { offset } => {
                if self.cond_reg.n {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bpl { offset } => {
                if !self.cond_reg.n {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bvs { offset } => {
                if self.cond_reg.v {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bcv { offset } => {
                if !self.cond_reg.v {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bhi { offset } => {
                if self.cond_reg.c && !self.cond_reg.z {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bls { offset } => {
                if !self.cond_reg.c || self.cond_reg.z {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bge { offset } => {
                if self.cond_reg.n == self.cond_reg.v {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Blt { offset } => {
                if self.cond_reg.n != self.cond_reg.v {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Bgt { offset } => {
                if !self.cond_reg.z && (self.cond_reg.n == self.cond_reg.v) {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::Ble { offset } => {
                if self.cond_reg.z || (self.cond_reg.n != self.cond_reg.v) {
                    self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                    self.increment_pc();
                }
            }
            Instruction::B { offset } => {
                self.set_register(PC_INDEX, self.read_register(PC_INDEX) + offset);
                self.increment_pc();
            }
            Instruction::Bl {
                offset1,
                offset2,
                first,
            } => {
                if first {
                    self.set_register(LR_INDEX, self.read_register(PC_INDEX) + offset1);
                } else {
                    let next_instruction = self.read_register(PC_INDEX) - 2;
                    self.set_register(PC_INDEX, self.read_register(LR_INDEX) + offset2);
                    self.set_register(LR_INDEX, next_instruction | 1);
                    self.increment_pc();
                }
            }
            Instruction::Dmb => {
                self.increment_pc();
            }
            Instruction::NotImplemented => {}
        }
    }
}
