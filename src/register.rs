use crate::Gamebuino;

#[derive(Debug)]
pub struct CondRegister {
    pub n: bool,
    pub z: bool,
    pub v: bool,
    pub c: bool,
}

impl CondRegister {
    pub fn to_word(&self) -> u32 {
        (if self.c { 1 } else { 0 })
            | (if self.n { 2 } else { 0 })
            | (if self.v { 4 } else { 0 })
            | (if self.z { 8 } else { 0 })
    }

    pub fn from_word(&mut self, val: u32) {
        self.c = if val & 1 != 0 { true } else { false };
        self.n = if val & 2 != 0 { true } else { false };
        self.v = if val & 4 != 0 { true } else { false };
        self.z = if val & 8 != 0 { true } else { false };
    }
}

pub trait Peripheral {
    fn handle_write_word(&mut self, offset: u32, value: u32, gamebuino: &mut Gamebuino);
    fn handle_write_byte(&mut self, offset: u32, value: u8, gamebuino: &mut Gamebuino);
    fn handle_read_word(&self, offset: u32) -> u32;
    fn handle_read_byte(&self, offset: u32) -> u8;
}

#[derive(Clone, Copy)]
pub struct DmacRegisters {
    base_address: u32,
    wrb_address: u32,
    descriptor: u32,
    selected_channel_id: u8,
}

impl DmacRegisters {
    const BASEADDR_OFFSET: u32 = 0x34;
    const WRBADDR_OFFSET: u32 = 0x38;
    const CHID_OFFSET: u32 = 0x3f;
    const CHCTRLA_OFFSET: u32 = 0x40;
    const CHINTFLAG_OFFSET: u32 = 0x4e;
    pub const DMAC_START_ADDR: u32 = 0x41004800;
    pub const DMAC_END_ADDR: u32 = DmacRegisters::DMAC_START_ADDR + DmacRegisters::CHINTFLAG_OFFSET;

    pub fn new() -> DmacRegisters {
        DmacRegisters {
            base_address: 0,
            wrb_address: 0,
            descriptor: 0,
            selected_channel_id: 0,
        }
    }
}

impl Peripheral for DmacRegisters {
    fn handle_write_word(&mut self, offset: u32, value: u32, _gamebuino: &mut Gamebuino) {
        // log!("dmac write word {:x}", offset);
        match offset {
            DmacRegisters::BASEADDR_OFFSET => {
                self.base_address = value;
            }
            DmacRegisters::WRBADDR_OFFSET => {
                self.wrb_address = value;
            }
            _ => {}
        }
    }

    fn handle_write_byte(&mut self, offset: u32, value: u8, gamebuino: &mut Gamebuino) {
        match offset {
            DmacRegisters::CHID_OFFSET => {
                self.selected_channel_id = value;
            }
            DmacRegisters::CHCTRLA_OFFSET => {
                if value == 0b10 {
                    if self.descriptor == 0 {
                        self.descriptor =
                            self.base_address + self.selected_channel_id as u32 * 0x10;
                    }

                    let _btctrl = gamebuino.fetch_half_word(self.descriptor);
                    let btcnt = gamebuino.fetch_half_word(self.descriptor + 0x02) as u32;
                    let srcaddr = gamebuino.fetch_word(self.descriptor + 0x04);
                    let dstaddr = gamebuino.fetch_word(self.descriptor + 0x08);
                    let descaddr = gamebuino.fetch_word(self.descriptor + 0x0C);

                    for i in 0..btcnt {
                        gamebuino
                            .write_byte(dstaddr, gamebuino.fetch_byte(srcaddr + i - btcnt) as u32);
                    }

                    self.descriptor = descaddr;

                    gamebuino.dmac_interrupt();
                }
            }
            _ => {}
        }
    }

    fn handle_read_word(&self, offset: u32) -> u32 {
        match offset {
            DmacRegisters::CHINTFLAG_OFFSET => 0b010, // TCMPL
            _ => 0,
        }
    }

    fn handle_read_byte(&self, offset: u32) -> u8 {
        match offset {
            DmacRegisters::CHINTFLAG_OFFSET => 0b010, // TCMPL
            _ => 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PortRegisters {
    pub out_value: u32,
    in_value: u32,
    dir_value: u32,
}

impl PortRegisters {
    const DIR_OFFSET: u32 = 0x00;
    const DIRCLR_OFFSET: u32 = 0x04;
    const DIRSET_OFFSET: u32 = 0x08;
    const DIRTGL_OFFSET: u32 = 0x0C;
    const OUT_OFFSET: u32 = 0x10;
    const OUTCLR_OFFSET: u32 = 0x14;
    const OUTSET_OFFSET: u32 = 0x18;
    const OUTTGL_OFFSET: u32 = 0x1C;
    const IN_OFFSET: u32 = 0x20;
    pub const PORTA_START_ADDR: u32 = 0x41004400;
    pub const PORTA_END_ADDR: u32 = PortRegisters::PORTA_START_ADDR + PortRegisters::IN_OFFSET;
    pub const PORTB_START_ADDR: u32 = 0x41004480;
    pub const PORTB_END_ADDR: u32 = PortRegisters::PORTB_START_ADDR + PortRegisters::IN_OFFSET;

    pub fn new() -> PortRegisters {
        PortRegisters {
            out_value: 0,
            in_value: 0,
            dir_value: 0,
        }
    }
}

impl Peripheral for PortRegisters {
    fn handle_write_word(&mut self, offset: u32, value: u32, _gamebuino: &mut Gamebuino) {
        match offset {
            PortRegisters::OUT_OFFSET => {
                self.out_value = value;
            }
            PortRegisters::OUTCLR_OFFSET => {
                let new_value = self.out_value & !value;
                self.out_value = new_value;
            }
            PortRegisters::OUTSET_OFFSET => {
                let new_value = self.out_value | value;
                self.out_value = new_value;
            }
            PortRegisters::OUTTGL_OFFSET => {
                let new_value = self.out_value ^ value;
                self.out_value = new_value;
            }
            PortRegisters::DIR_OFFSET => {
                self.dir_value ^= value;
            }
            PortRegisters::DIRCLR_OFFSET => {
                self.dir_value &= !value;
            }
            PortRegisters::DIRSET_OFFSET => {
                self.dir_value |= value;
            }
            PortRegisters::DIRTGL_OFFSET => {
                self.dir_value ^= value;
            }
            _ => {}
        }
    }
    fn handle_write_byte(&mut self, _offset: u32, _value: u8, _gamebuino: &mut Gamebuino) {}

    fn handle_read_word(&self, offset: u32) -> u32 {
        match offset {
            PortRegisters::OUT_OFFSET => self.out_value,
            PortRegisters::OUTCLR_OFFSET => self.out_value,
            PortRegisters::OUTSET_OFFSET => self.out_value,
            PortRegisters::OUTTGL_OFFSET => self.out_value,
            PortRegisters::IN_OFFSET => self.in_value,
            PortRegisters::DIR_OFFSET => self.dir_value,
            PortRegisters::DIRCLR_OFFSET => self.dir_value,
            PortRegisters::DIRSET_OFFSET => self.dir_value,
            PortRegisters::DIRTGL_OFFSET => self.dir_value,
            _ => 0,
        }
    }

    fn handle_read_byte(&self, _offset: u32) -> u8 {
        0
    }
}

#[derive(Clone, Copy)]
pub struct SercomRegisters {
    pub data: u8,
    pub sent: Option<u8>,
}

impl SercomRegisters {
    const INTFLAG_OFFSET: u32 = 0x18;
    const DATA_OFFSET: u32 = 0x28;
    const SERCOM0_ADDR: u32 = 0x42000800;
    pub const SERCOM4_START_ADDR: u32 = SercomRegisters::SERCOM0_ADDR + 4 * 0x400;
    pub const SERCOM4_END_ADDR: u32 =
        SercomRegisters::SERCOM4_START_ADDR + SercomRegisters::DATA_OFFSET;
    pub const SERCOM5_START_ADDR: u32 = SercomRegisters::SERCOM0_ADDR + 5 * 0x400;
    pub const SERCOM5_END_ADDR: u32 =
        SercomRegisters::SERCOM5_START_ADDR + SercomRegisters::DATA_OFFSET;

    pub fn new() -> SercomRegisters {
        SercomRegisters {
            data: 0x80,
            sent: None,
        }
    }
}

impl Peripheral for SercomRegisters {
    fn handle_write_word(&mut self, offset: u32, value: u32, _gamebuino: &mut Gamebuino) {
        match offset {
            SercomRegisters::DATA_OFFSET => {
                self.data = 0x80;
                self.sent = Some(value as u8);
            }
            _ => {}
        }
    }

    fn handle_write_byte(&mut self, offset: u32, value: u8, _gamebuino: &mut Gamebuino) {
        match offset {
            SercomRegisters::DATA_OFFSET => {
                self.data = 0x80;
                self.sent = Some(value);
            }
            _ => {}
        }
    }

    fn handle_read_word(&self, offset: u32) -> u32 {
        match offset {
            SercomRegisters::INTFLAG_OFFSET => 0b00000111 as u32, // RXC, TXC, DRE
            SercomRegisters::DATA_OFFSET => self.data as u32,
            _ => 0,
        }
    }

    fn handle_read_byte(&self, offset: u32) -> u8 {
        match offset {
            SercomRegisters::INTFLAG_OFFSET => 0b00000111, // RXC, TXC, DRE
            SercomRegisters::DATA_OFFSET => self.data,
            _ => 0,
        }
    }
}

pub struct TcRegisters {
}

impl TcRegisters {
    const TC5_ADDRESS: u32 = 0x42003400;
    pub const TC5_CC_ADDRESS: u32 = TcRegisters::TC5_ADDRESS + 0x18;
}