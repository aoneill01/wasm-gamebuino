use crate::register::{PortRegisters, SercomRegisters};

pub struct St7735 {
    x_start: u8,
    x_end: u8,
    y_start: u8,
    y_end: u8,
    x: u8,
    y: u8,
    arg_index: u8,
    last_command: u8,
    tmp_data: u8,
    image_data: [u32; St7735::WIDTH * St7735::HEIGHT],
}

impl St7735 {
    const CASET: u8 = 0x2a; // Column address set command
    const RASET: u8 = 0x2b; // Row address set command
    const RAMWR: u8 = 0x2c; // Memory write comman
    const WIDTH: usize = 160; // Screen width in pixels
    const HEIGHT: usize = 128; // Screen height in pixels

    pub fn new() -> St7735 {
        St7735 {
            x_start: 0,
            x_end: 0,
            y_start: 0,
            y_end: 0,
            x: 0,
            y: 0,
            arg_index: 0,
            last_command: 0,
            tmp_data: 0,
            image_data: [0; St7735::WIDTH * St7735::HEIGHT],
        }
    }

    pub fn image_pointer(&self) -> *const u32 {
        self.image_data.as_ptr()
    }

    pub fn byte_received(&mut self, value: u8, porta: &PortRegisters, portb: &PortRegisters) {
        if porta.out_value & (1 << 22) != 0 {
            return;
        }
        if portb.out_value & 0b100000000000000000000000 != 0 {
            match self.last_command {
                St7735::RAMWR => {
                    if self.arg_index % 2 == 0 {
                        self.tmp_data = value;
                    } else {
                        let pixel_data = ((self.tmp_data as u32) << 8) | value as u32;
                        let r = (0b1111100000000000 & pixel_data) >> 8;
                        let g = (0b0000011111100000 & pixel_data) >> 3;
                        let b = (0b0000000000011111 & pixel_data) << 3;
                        let color = (255 << 24) | // alpha
                                    (b   << 16) | // blue
                                    (g   <<  8) | // green
                                     r; // red
                        let base_index = self.y as usize * St7735::WIDTH + self.x as usize;

                        if base_index < St7735::WIDTH * St7735::HEIGHT {
                            self.image_data[base_index] = color;
                        }

                        self.x += 1;
                        if self.x > self.x_end {
                            self.x = self.x_start;
                            self.y += 1;
                            if self.y > self.y_end {
                                self.y = self.y_end;
                            }
                        }
                    }
                }
                St7735::CASET => {
                    if self.arg_index == 1 {
                        self.x_start = value;
                        self.x = value;
                    } else if self.arg_index == 3 {
                        self.x_end = value;
                    }
                }
                St7735::RASET => {
                    if self.arg_index == 1 {
                        self.y_start = value;
                        self.y = value;
                    } else if self.arg_index == 3 {
                        self.y_end = value;
                    }
                }
                _ => {}
            }
            self.arg_index += 1;
        } else {
            self.last_command = value;
            self.arg_index = 0;
        }
    }
}

pub struct Buttons {
    pub button_data: u8,
}

impl Buttons {
    pub fn new() -> Buttons {
        Buttons { button_data: 0xff }
    }

    pub fn byte_received(
        &mut self,
        _value: u8,
        portb: &PortRegisters,
        sercom4: &mut SercomRegisters,
    ) {
        if (portb.out_value & (1 << 3)) != 0 {
            return;
        }

        sercom4.data = self.button_data;
    }
}
