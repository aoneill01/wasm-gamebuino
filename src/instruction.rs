#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    LslImm {
        rs: u8,
        rd: u8,
        offset: u8,
    },
    LslReg {
        rs: u8,
        rd: u8,
    },
    LsrImm {
        rs: u8,
        rd: u8,
        offset: u8,
    },
    LsrReg {
        rs: u8,
        rd: u8,
    },
    AsrImm {
        rs: u8,
        rd: u8,
        offset: u8,
    },
    AsrReg {
        rs: u8,
        rd: u8,
    },
    AddReg {
        rs: u8,
        rd: u8,
        rn: u8,
    },
    AddImm {
        rs: u8,
        rd: u8,
        offset: u8,
    },
    AddSp {
        rd: u8,
        offset: u32,
    },
    AddPc {
        rd: u8,
        offset: u32,
    },
    Adc {
        rs: u8,
        rd: u8,
    },
    SubReg {
        rs: u8,
        rd: u8,
        rn: u8,
    },
    SubImm {
        rs: u8,
        rd: u8,
        offset: u8,
    },
    Sbc {
        rs: u8,
        rd: u8,
    },
    Neg {
        rs: u8,
        rd: u8,
    },
    Mul {
        rs: u8,
        rd: u8,
    },
    MovImm {
        rd: u8,
        offset: u8,
    },
    MovReg {
        rs: u8,
        rd: u8,
    },
    Mvn {
        rs: u8,
        rd: u8,
    },
    CmpImm {
        rd: u8,
        offset: u8,
    },
    CmpReg {
        rs: u8,
        rd: u8,
    },
    Cmn {
        rs: u8,
        rd: u8,
    },
    Tst {
        rs: u8,
        rd: u8,
    },
    And {
        rs: u8,
        rd: u8,
    },
    Bic {
        rs: u8,
        rd: u8,
    },
    Eor {
        rs: u8,
        rd: u8,
    },
    Oor {
        rs: u8,
        rd: u8,
    },
    Bx {
        rs: u8,
    },
    Blx {
        rm: u8,
    },
    LdrPc {
        rd: u8,
        immediate_value: u32,
    },
    LdrReg {
        rb: u8,
        ro: u8,
        rd: u8,
    },
    LdrbReg {
        rb: u8,
        ro: u8,
        rd: u8,
    },
    LdrImm {
        rb: u8,
        offset: u32,
        rd: u8,
    },
    LdrbImm {
        rb: u8,
        offset: u32,
        rd: u8,
    },
    Ldsb {
        rb: u8,
        ro: u8,
        rd: u8,
    },
    LdrhReg {
        rb: u8,
        ro: u8,
        rd: u8,
    },
    LdrhImm {
        rb: u8,
        offset: u32,
        rd: u8,
    },
    Ldsh {
        rb: u8,
        ro: u8,
        rd: u8,
    },
    Ldmia {
        rb: u8,
        rlist: u8,
    },
    StrReg {
        rb: u8,
        ro: u8,
        rd: u8,
    },
    StrbReg {
        rb: u8,
        ro: u8,
        rd: u8,
    },
    StrImm {
        rb: u8,
        offset: u32,
        rd: u8,
    },
    StrbImm {
        rb: u8,
        offset: u32,
        rd: u8,
    },
    StrhReg {
        rb: u8,
        ro: u8,
        rd: u8,
    },
    StrhImm {
        rb: u8,
        offset: u32,
        rd: u8,
    },
    Stmia {
        rb: u8,
        rlist: u8,
    },
    Sxth {
        rd: u8,
        rm: u8,
    },
    Sxtb {
        rd: u8,
        rm: u8,
    },
    Uxth {
        rd: u8,
        rm: u8,
    },
    Uxtb {
        rd: u8,
        rm: u8,
    },
    Rev {
        rd: u8,
        rm: u8,
    },
    Rev16 {
        rd: u8,
        rm: u8,
    },
    Push {
        rlist: u8,
        lr: bool,
    },
    Pop {
        rlist: u8,
        pc: bool,
    },
    Beq {
        offset: u32,
    },
    Bne {
        offset: u32,
    },
    Bcs {
        offset: u32,
    },
    Bcc {
        offset: u32,
    },
    Bmi {
        offset: u32,
    },
    Bpl {
        offset: u32,
    },
    Bvs {
        offset: u32,
    },
    Bcv {
        offset: u32,
    },
    Bhi {
        offset: u32,
    },
    Bls {
        offset: u32,
    },
    Bge {
        offset: u32,
    },
    Blt {
        offset: u32,
    },
    Bgt {
        offset: u32,
    },
    Ble {
        offset: u32,
    },
    B {
        offset: u32,
    },
    Bl {
        offset1: u32,
        offset2: u32,
        first: bool,
    },
    Dmb,
    NotImplemented,
}

pub fn parse_instruction(instruction: u16, following_instruction: u16) -> Instruction {
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
                _ => Instruction::NotImplemented,
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
                _ => Instruction::NotImplemented,
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
            _ => Instruction::NotImplemented,
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
            _ => Instruction::NotImplemented,
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
            _ => Instruction::NotImplemented,
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
            _ => Instruction::NotImplemented,
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
            _ => Instruction::NotImplemented,
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
            _ => Instruction::NotImplemented,
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
                rb: crate::SP_INDEX,
                offset,
                rd,
            }
        } else {
            Instruction::StrImm {
                rb: crate::SP_INDEX,
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
            rd: crate::SP_INDEX,
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
            _ => Instruction::NotImplemented,
        }
    } else if (instruction & 0b1111111100000000) == 0b1011101000000000 {
        let opcode = (instruction & 0b0000000011000000) >> 6;
        let rm = ((instruction & 0b0000000000111000) >> 3) as u8;
        let rd = (instruction & 0b0000000000000111) as u8;
        match opcode {
            // TODO revsh
            0b00 => Instruction::Rev { rd, rm },
            0b01 => Instruction::Rev16 { rd, rm },
            _ => Instruction::NotImplemented,
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
            _ => Instruction::NotImplemented,
        }
    } else if (instruction & 0b1111100000000000) == 0b1110000000000000 {
        let mut offset = (instruction & 0b0000011111111111) as u32;
        // Handle negative
        if offset & 0b10000000000 != 0 {
            offset |= !0b11111111111;
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
            offset1,
            offset2,
            first: true,
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
