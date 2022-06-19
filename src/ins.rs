use crate::error::*;

type Reg = usize;

#[derive(Debug)]
pub enum Ins {
    Sys(u16),
    Cls,
    Ret,
    Jp(u16),
    Call(u16),
    SeVB(Reg, u8),
    SneVB(Reg, u8),
    SeVV(Reg, Reg),
    LdVB(Reg, u8),
    AddVB(Reg, u8),
    LdVV(Reg, Reg),
    Or(Reg, Reg),
    And(Reg, Reg),
    Xor(Reg, Reg),
    AddVV(Reg, Reg),
    Sub(Reg, Reg),
    Shr(Reg),
    Subn(Reg, Reg),
    Shl(Reg),
    SneVV(Reg, Reg),
    LdI(u16),
    JpV0(u16),
    Rnd(Reg, u8),
    Drw(Reg, Reg, u8),
    Skp(Reg),
    Sknp(Reg),
    LdVDt(Reg),
    LdVK(Reg),
    LdDtV(Reg),
    LdStV(Reg),
    AddI(Reg),
    LdFV(Reg),
    LdBV(Reg),
    LdIlocV(Reg),
    LdVIloc(Reg),
}

impl Ins {
    pub fn decode(opcode: u16) -> Result<Self> {
        match (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            opcode & 0x000F,
        ) {
            (0, 0, 0xE, 0) => Ok(Self::Cls),
            (0, 0, 0xE, 0xE) => Ok(Self::Ret),
            (0, _, _, _) => Ok(Self::Sys(opcode & 0xFFF)),
            (1, _, _, _) => Ok(Self::Jp(opcode & 0x0FFF)),
            (2, _, _, _) => Ok(Self::Call(opcode & 0x0FFF)),
            (3, x, _, _) => Ok(Self::SeVB(x.into(), opcode.to_le_bytes()[0])),
            (4, x, _, _) => Ok(Self::SneVB(x.into(), opcode.to_le_bytes()[0])),
            (5, x, y, 0) => Ok(Self::SeVV(x.into(), y.into())),
            (6, x, _, _) => Ok(Self::LdVB(x.into(), opcode.to_le_bytes()[0])),
            (7, x, _, _) => Ok(Self::AddVB(x.into(), opcode.to_le_bytes()[0])),
            (8, x, y, 0) => Ok(Self::LdVV(x.into(), y.into())),
            (8, x, y, 1) => Ok(Self::Or(x.into(), y.into())),
            (8, x, y, 2) => Ok(Self::And(x.into(), y.into())),
            (8, x, y, 3) => Ok(Self::Xor(x.into(), y.into())),
            (8, x, y, 4) => Ok(Self::AddVV(x.into(), y.into())),
            (8, x, y, 5) => Ok(Self::Sub(x.into(), y.into())),
            (8, x, _, 6) => Ok(Self::Shr(x.into())),
            (8, x, y, 7) => Ok(Self::Subn(x.into(), y.into())),
            (8, x, _, 0xE) => Ok(Self::Shl(x.into())),
            (9, x, y, 0) => Ok(Self::SneVV(x.into(), y.into())),
            (0xA, _, _, _) => Ok(Self::LdI(opcode & 0x0FFF)),
            (0xB, _, _, _) => Ok(Self::JpV0(opcode & 0x0FFF)),
            (0xC, x, _, _) => Ok(Self::Rnd(x.into(), opcode.to_le_bytes()[0])),
            (0xD, x, y, n) => Ok(Self::Drw(x.into(), y.into(), n.to_le_bytes()[0])),
            (0xE, x, 9, 0xE) => Ok(Self::Skp(x.into())),
            (0xE, x, 0xA, 1) => Ok(Self::Sknp(x.into())),
            (0xF, x, 0, 7) => Ok(Self::LdVDt(x.into())),
            (0xF, x, 0, 0xA) => Ok(Self::LdVK(x.into())),
            (0xF, x, 1, 5) => Ok(Self::LdDtV(x.into())),
            (0xF, x, 1, 8) => Ok(Self::LdStV(x.into())),
            (0xF, x, 1, 0xE) => Ok(Self::AddI(x.into())),
            (0xF, x, 2, 9) => Ok(Self::LdFV(x.into())),
            (0xF, x, 3, 3) => Ok(Self::LdBV(x.into())),
            (0xF, x, 5, 5) => Ok(Self::LdIlocV(x.into())),
            (0xF, x, 6, 5) => Ok(Self::LdVIloc(x.into())),
            _ => Err(Error::Opcode(opcode)),
        }
    }
}