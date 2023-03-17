use std::{cell::Cell, fmt::Debug};

#[derive(Debug, Clone, Copy)]
pub enum Mnemonic {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    UNK,
}

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
}

pub struct Instruction {
    pub opcode: u8,
    pub operand1: u8,
    pub operand2: u8,
    disassembly: Cell<Option<Disassembly>>,
}

#[derive(Clone, Copy)]
pub struct Disassembly {
    pub mnemonic: Mnemonic,
    pub addressing_mode: AddressingMode,
}

impl Instruction {
    pub fn new(opcode: u8, operand1: u8, operand2: u8) -> Self {
        Self {
            opcode,
            operand1,
            operand2,
            disassembly: Cell::new(None),
        }
    }

    fn length(&self) -> u8 {
        match self.disassembly().addressing_mode {
            AddressingMode::Implied => 1,
            AddressingMode::Accumulator => 1,
            AddressingMode::Immediate => 2,
            AddressingMode::Absolute => 3,
            AddressingMode::AbsoluteX => 3,
            AddressingMode::AbsoluteY => 3,
            AddressingMode::Indirect => 3,
            AddressingMode::IndirectX => 2,
            AddressingMode::IndirectY => 2,
            AddressingMode::ZeroPage => 2,
            AddressingMode::ZeroPageX => 2,
            AddressingMode::ZeroPageY => 2,
            AddressingMode::Relative => 2,
        }
    }

    fn disassembly(&self) -> Disassembly {
        match self.disassembly.get() {
            Some(disassembly) => disassembly,
            None => {
                let disassembly = self.disassemble();
                self.disassembly.set(Some(disassembly));
                disassembly
            }
        }
    }

    fn disassemble(&self) -> Disassembly {
        let disassembly = |mnemonic: Mnemonic, addressing_mode: AddressingMode| Disassembly {
            mnemonic,
            addressing_mode,
        };
        let unknown = || disassembly(Mnemonic::UNK, AddressingMode::Implied);

        match self.opcode {
            0x00 => disassembly(Mnemonic::BRK, AddressingMode::Implied),
            0x01 => disassembly(Mnemonic::ORA, AddressingMode::IndirectX),
            0x02 => unknown(),
            0x03 => unknown(),
            0x04 => unknown(),
            0x05 => disassembly(Mnemonic::ORA, AddressingMode::ZeroPage),
            0x06 => disassembly(Mnemonic::ASL, AddressingMode::ZeroPage),
            0x07 => unknown(),
            0x08 => disassembly(Mnemonic::PHP, AddressingMode::Implied),
            0x09 => disassembly(Mnemonic::ORA, AddressingMode::Immediate),
            0x0A => disassembly(Mnemonic::ASL, AddressingMode::Accumulator),
            0x0B => unknown(),
            0x0C => unknown(),
            0x0D => disassembly(Mnemonic::ORA, AddressingMode::Absolute),
            0x0E => disassembly(Mnemonic::ASL, AddressingMode::Absolute),
            0x0F => unknown(),

            0x10 => disassembly(Mnemonic::BPL, AddressingMode::Relative),
            0x11 => disassembly(Mnemonic::ORA, AddressingMode::IndirectY),
            0x12 => unknown(),
            0x13 => unknown(),
            0x14 => unknown(),
            0x15 => disassembly(Mnemonic::ORA, AddressingMode::ZeroPageX),
            0x16 => disassembly(Mnemonic::ASL, AddressingMode::ZeroPageX),
            0x17 => unknown(),
            0x18 => disassembly(Mnemonic::CLC, AddressingMode::Implied),
            0x19 => disassembly(Mnemonic::ORA, AddressingMode::AbsoluteY),
            0x1A => unknown(),
            0x1B => unknown(),
            0x1C => unknown(),
            0x1D => disassembly(Mnemonic::ORA, AddressingMode::AbsoluteX),
            0x1E => disassembly(Mnemonic::ASL, AddressingMode::AbsoluteX),
            0x1F => unknown(),
            0x20 => disassembly(Mnemonic::JSR, AddressingMode::Absolute),
            0x21 => disassembly(Mnemonic::AND, AddressingMode::IndirectX),
            0x22 => unknown(),
            0x23 => unknown(),
            0x24 => disassembly(Mnemonic::BIT, AddressingMode::ZeroPage),
            0x25 => disassembly(Mnemonic::AND, AddressingMode::ZeroPage),
            0x26 => disassembly(Mnemonic::ROL, AddressingMode::ZeroPage),
            0x27 => unknown(),
            0x28 => disassembly(Mnemonic::PLP, AddressingMode::Implied),
            0x29 => disassembly(Mnemonic::AND, AddressingMode::Immediate),
            0x2A => disassembly(Mnemonic::ROL, AddressingMode::Accumulator),
            0x2B => unknown(),
            0x2C => disassembly(Mnemonic::BIT, AddressingMode::Absolute),
            0x2D => disassembly(Mnemonic::AND, AddressingMode::Absolute),
            0x2E => disassembly(Mnemonic::ROL, AddressingMode::Absolute),
            0x2F => unknown(),

            0x30 => disassembly(Mnemonic::BMI, AddressingMode::Relative),
            0x31 => disassembly(Mnemonic::AND, AddressingMode::IndirectY),
            0x32 => unknown(),
            0x33 => unknown(),
            0x34 => unknown(),
            0x35 => disassembly(Mnemonic::AND, AddressingMode::ZeroPageX),
            0x36 => disassembly(Mnemonic::ROL, AddressingMode::ZeroPageX),
            0x37 => unknown(),
            0x38 => disassembly(Mnemonic::SEC, AddressingMode::Implied),
            0x39 => disassembly(Mnemonic::AND, AddressingMode::AbsoluteY),
            0x3A => unknown(),
            0x3B => unknown(),
            0x3C => unknown(),
            0x3D => disassembly(Mnemonic::AND, AddressingMode::AbsoluteX),
            0x3E => disassembly(Mnemonic::ROL, AddressingMode::AbsoluteX),
            0x3F => unknown(),

            0x40 => disassembly(Mnemonic::RTI, AddressingMode::Implied),
            0x41 => disassembly(Mnemonic::EOR, AddressingMode::IndirectX),
            0x42 => unknown(),
            0x43 => unknown(),
            0x44 => unknown(),
            0x45 => disassembly(Mnemonic::EOR, AddressingMode::ZeroPage),
            0x46 => disassembly(Mnemonic::LSR, AddressingMode::ZeroPage),
            0x47 => unknown(),
            0x48 => disassembly(Mnemonic::PHA, AddressingMode::Implied),
            0x49 => disassembly(Mnemonic::EOR, AddressingMode::Immediate),
            0x4A => disassembly(Mnemonic::LSR, AddressingMode::Accumulator),
            0x4B => unknown(),
            0x4C => disassembly(Mnemonic::JMP, AddressingMode::Absolute),
            0x4D => disassembly(Mnemonic::EOR, AddressingMode::Absolute),
            0x4E => disassembly(Mnemonic::LSR, AddressingMode::Absolute),
            0x4F => unknown(),

            0x50 => disassembly(Mnemonic::BVC, AddressingMode::Relative),
            0x51 => disassembly(Mnemonic::EOR, AddressingMode::IndirectY),
            0x52 => unknown(),
            0x53 => unknown(),
            0x54 => unknown(),
            0x55 => disassembly(Mnemonic::EOR, AddressingMode::ZeroPageX),
            0x56 => disassembly(Mnemonic::LSR, AddressingMode::ZeroPageX),
            0x57 => unknown(),
            0x58 => disassembly(Mnemonic::CLI, AddressingMode::Implied),
            0x59 => disassembly(Mnemonic::EOR, AddressingMode::AbsoluteY),
            0x5A => unknown(),
            0x5B => unknown(),
            0x5C => unknown(),
            0x5D => disassembly(Mnemonic::EOR, AddressingMode::AbsoluteX),
            0x5E => disassembly(Mnemonic::LSR, AddressingMode::AbsoluteX),
            0x5F => unknown(),

            0x60 => disassembly(Mnemonic::RTS, AddressingMode::Implied),
            0x61 => disassembly(Mnemonic::ADC, AddressingMode::IndirectX),
            0x62 => unknown(),
            0x63 => unknown(),
            0x64 => unknown(),
            0x65 => disassembly(Mnemonic::ADC, AddressingMode::ZeroPage),
            0x66 => disassembly(Mnemonic::ROR, AddressingMode::ZeroPage),
            0x67 => unknown(),
            0x68 => disassembly(Mnemonic::PLA, AddressingMode::Implied),
            0x69 => disassembly(Mnemonic::ADC, AddressingMode::Immediate),
            0x6A => disassembly(Mnemonic::ROR, AddressingMode::Accumulator),
            0x6B => unknown(),
            0x6C => disassembly(Mnemonic::JMP, AddressingMode::Indirect),
            0x6D => disassembly(Mnemonic::ADC, AddressingMode::Absolute),
            0x6E => disassembly(Mnemonic::ROR, AddressingMode::Absolute),
            0x6F => unknown(),

            0x70 => disassembly(Mnemonic::BVS, AddressingMode::Relative),
            0x71 => disassembly(Mnemonic::ADC, AddressingMode::IndirectY),
            0x72 => unknown(),
            0x73 => unknown(),
            0x74 => unknown(),
            0x75 => disassembly(Mnemonic::ADC, AddressingMode::ZeroPageX),
            0x76 => disassembly(Mnemonic::ROR, AddressingMode::ZeroPageX),
            0x77 => unknown(),
            0x78 => disassembly(Mnemonic::SEI, AddressingMode::Implied),
            0x79 => disassembly(Mnemonic::ADC, AddressingMode::AbsoluteY),
            0x7A => unknown(),
            0x7B => unknown(),
            0x7C => unknown(),
            0x7D => disassembly(Mnemonic::ADC, AddressingMode::AbsoluteX),
            0x7E => disassembly(Mnemonic::ROR, AddressingMode::AbsoluteX),
            0x7F => unknown(),

            0x80 => unknown(),
            0x81 => disassembly(Mnemonic::STA, AddressingMode::IndirectX),
            0x82 => unknown(),
            0x83 => unknown(),
            0x84 => disassembly(Mnemonic::STY, AddressingMode::ZeroPage),
            0x85 => disassembly(Mnemonic::STA, AddressingMode::ZeroPage),
            0x86 => disassembly(Mnemonic::STX, AddressingMode::ZeroPage),
            0x87 => unknown(),
            0x88 => disassembly(Mnemonic::DEY, AddressingMode::Implied),
            0x89 => unknown(),
            0x8A => disassembly(Mnemonic::TXA, AddressingMode::Implied),
            0x8B => unknown(),
            0x8C => disassembly(Mnemonic::STY, AddressingMode::Absolute),
            0x8D => disassembly(Mnemonic::STA, AddressingMode::Absolute),
            0x8E => disassembly(Mnemonic::STX, AddressingMode::Absolute),
            0x8F => unknown(),

            0x90 => disassembly(Mnemonic::BCC, AddressingMode::Relative),
            0x91 => disassembly(Mnemonic::STA, AddressingMode::IndirectY),
            0x92 => unknown(),
            0x93 => unknown(),
            0x94 => disassembly(Mnemonic::STY, AddressingMode::ZeroPageX),
            0x95 => disassembly(Mnemonic::STA, AddressingMode::ZeroPageX),
            0x96 => disassembly(Mnemonic::STX, AddressingMode::ZeroPageY),
            0x97 => unknown(),
            0x98 => disassembly(Mnemonic::TYA, AddressingMode::Implied),
            0x99 => disassembly(Mnemonic::STA, AddressingMode::AbsoluteX),
            0x9A => disassembly(Mnemonic::TXS, AddressingMode::Implied),
            0x9B => unknown(),
            0x9C => unknown(),
            0x9D => disassembly(Mnemonic::STA, AddressingMode::AbsoluteX),
            0x9E => unknown(),
            0x9F => unknown(),

            0xA0 => disassembly(Mnemonic::LDY, AddressingMode::Immediate),
            0xA1 => disassembly(Mnemonic::LDA, AddressingMode::IndirectX),
            0xA2 => disassembly(Mnemonic::LDX, AddressingMode::Immediate),
            0xA3 => unknown(),
            0xA4 => disassembly(Mnemonic::LDY, AddressingMode::ZeroPage),
            0xA5 => disassembly(Mnemonic::LDA, AddressingMode::ZeroPage),
            0xA6 => disassembly(Mnemonic::LDX, AddressingMode::ZeroPage),
            0xA7 => unknown(),
            0xA8 => disassembly(Mnemonic::TAY, AddressingMode::Implied),
            0xA9 => disassembly(Mnemonic::LDA, AddressingMode::Immediate),
            0xAA => disassembly(Mnemonic::TAX, AddressingMode::Implied),
            0xAB => unknown(),
            0xAC => disassembly(Mnemonic::LDY, AddressingMode::Absolute),
            0xAD => disassembly(Mnemonic::LDA, AddressingMode::Absolute),
            0xAE => disassembly(Mnemonic::LDX, AddressingMode::Absolute),
            0xAF => unknown(),

            0xB0 => disassembly(Mnemonic::BCS, AddressingMode::Relative),
            0xB1 => disassembly(Mnemonic::LDA, AddressingMode::IndirectY),
            0xB2 => unknown(),
            0xB3 => unknown(),
            0xB4 => disassembly(Mnemonic::LDY, AddressingMode::ZeroPageX),
            0xB5 => disassembly(Mnemonic::LDA, AddressingMode::ZeroPageX),
            0xB6 => disassembly(Mnemonic::LDX, AddressingMode::ZeroPageY),
            0xB7 => unknown(),
            0xB8 => disassembly(Mnemonic::CLV, AddressingMode::Implied),
            0xB9 => disassembly(Mnemonic::LDA, AddressingMode::AbsoluteY),
            0xBA => disassembly(Mnemonic::TSX, AddressingMode::Implied),
            0xBB => unknown(),
            0xBC => disassembly(Mnemonic::LDY, AddressingMode::AbsoluteX),
            0xBD => disassembly(Mnemonic::LDA, AddressingMode::AbsoluteX),
            0xBE => disassembly(Mnemonic::LDX, AddressingMode::AbsoluteY),
            0xBF => unknown(),

            0xC0 => disassembly(Mnemonic::CPY, AddressingMode::Immediate),
            0xC1 => disassembly(Mnemonic::CMP, AddressingMode::IndirectX),
            0xC2 => unknown(),
            0xC3 => unknown(),
            0xC4 => disassembly(Mnemonic::CPY, AddressingMode::ZeroPage),
            0xC5 => disassembly(Mnemonic::CMP, AddressingMode::ZeroPage),
            0xC6 => disassembly(Mnemonic::DEC, AddressingMode::ZeroPage),
            0xC7 => unknown(),
            0xC8 => disassembly(Mnemonic::INY, AddressingMode::Implied),
            0xC9 => disassembly(Mnemonic::CMP, AddressingMode::Immediate),
            0xCA => disassembly(Mnemonic::DEX, AddressingMode::Implied),
            0xCB => unknown(),
            0xCC => disassembly(Mnemonic::CPY, AddressingMode::Absolute),
            0xCD => disassembly(Mnemonic::CMP, AddressingMode::Absolute),
            0xCE => disassembly(Mnemonic::DEC, AddressingMode::Absolute),
            0xCF => unknown(),

            0xD0 => disassembly(Mnemonic::BNE, AddressingMode::Relative),
            0xD1 => disassembly(Mnemonic::CMP, AddressingMode::IndirectY),
            0xD2 => unknown(),
            0xD3 => unknown(),
            0xD4 => unknown(),
            0xD5 => disassembly(Mnemonic::CMP, AddressingMode::ZeroPageX),
            0xD6 => disassembly(Mnemonic::DEC, AddressingMode::ZeroPageX),
            0xD7 => unknown(),
            0xD8 => disassembly(Mnemonic::CLD, AddressingMode::Implied),
            0xD9 => disassembly(Mnemonic::CMP, AddressingMode::AbsoluteY),
            0xDA => unknown(),
            0xDB => unknown(),
            0xDC => unknown(),
            0xDD => disassembly(Mnemonic::CMP, AddressingMode::AbsoluteX),
            0xDE => disassembly(Mnemonic::DEC, AddressingMode::AbsoluteX),
            0xDF => unknown(),

            0xE0 => disassembly(Mnemonic::CPX, AddressingMode::Immediate),
            0xE1 => disassembly(Mnemonic::SBC, AddressingMode::IndirectX),
            0xE2 => unknown(),
            0xE3 => unknown(),
            0xE4 => disassembly(Mnemonic::CPX, AddressingMode::ZeroPage),
            0xE5 => disassembly(Mnemonic::SBC, AddressingMode::ZeroPage),
            0xE6 => disassembly(Mnemonic::INC, AddressingMode::ZeroPage),
            0xE7 => unknown(),
            0xE8 => disassembly(Mnemonic::INX, AddressingMode::Implied),
            0xE9 => disassembly(Mnemonic::SBC, AddressingMode::Immediate),
            0xEA => disassembly(Mnemonic::NOP, AddressingMode::Implied),
            0xEB => unknown(),
            0xEC => disassembly(Mnemonic::CPX, AddressingMode::Absolute),
            0xED => disassembly(Mnemonic::SBC, AddressingMode::Absolute),
            0xEE => disassembly(Mnemonic::INC, AddressingMode::Absolute),
            0xEF => unknown(),

            0xF0 => disassembly(Mnemonic::BEQ, AddressingMode::Relative),
            0xF1 => disassembly(Mnemonic::SBC, AddressingMode::IndirectY),
            0xF2 => unknown(),
            0xF3 => unknown(),
            0xF4 => unknown(),
            0xF5 => disassembly(Mnemonic::SBC, AddressingMode::ZeroPageX),
            0xF6 => disassembly(Mnemonic::INC, AddressingMode::ZeroPageX),
            0xF7 => unknown(),
            0xF8 => disassembly(Mnemonic::SED, AddressingMode::Implied),
            0xF9 => disassembly(Mnemonic::SBC, AddressingMode::AbsoluteY),
            0xFA => unknown(),
            0xFB => unknown(),
            0xFC => unknown(),
            0xFD => disassembly(Mnemonic::SBC, AddressingMode::AbsoluteX),
            0xFE => disassembly(Mnemonic::INC, AddressingMode::AbsoluteX),
            0xFF => unknown(),
        }
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;
        std::fmt::Display::fmt(&self, f)?;
        write!(f, " }}")
    }
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        match self.length() {
            1 => self.opcode == other.opcode,
            2 => self.opcode == other.opcode && self.operand1 == other.operand1,
            3 => {
                self.opcode == other.opcode
                    && self.operand1 == other.operand1
                    && self.operand2 == other.operand2
            }
            _ => unreachable!(),
        }
    }
}
impl Eq for Instruction {}

impl std::fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Instruction {
            opcode,
            operand1,
            operand2,
            ..
        } = self;
        let Disassembly {
            mnemonic,
            addressing_mode,
        } = self.disassembly();

        let raw_bytes = match self.length() {
            1 => format!("{:02X}", opcode),
            2 => format!("{:02X} {:02X}", opcode, operand1),
            3 => format!("{:02X} {:02X} {:02X}", opcode, operand1, operand2),
            _ => unreachable!(),
        };

        use AddressingMode::*;
        let disassembly = match addressing_mode {
            Implied => format!("{}", mnemonic),
            Accumulator => format!("{} A", mnemonic),
            Immediate => format!("{} #${:02X}", mnemonic, operand1),
            Absolute => format!("{} ${:02X}{:02X}", mnemonic, operand2, operand1),
            AbsoluteX => format!("{} ${:02X}{:02X},X", mnemonic, operand2, operand1),
            AbsoluteY => format!("{} ${:02X}{:02X},Y", mnemonic, operand2, operand2),
            Indirect => format!("{} (${:02X}{:02X})", mnemonic, operand2, operand1),
            IndirectX => format!("{} (${:02X},X)", mnemonic, operand1),
            IndirectY => format!("{} (${:02X}),Y", mnemonic, operand1),
            ZeroPage => format!("{} ${:02X}", mnemonic, operand1),
            ZeroPageX => format!("{} ${:02X},X", mnemonic, operand1),
            ZeroPageY => format!("{} ${:02X},Y", mnemonic, operand1),
            Relative => format!("{} ${:02X}", mnemonic, operand1),
        };

        f.pad(&format!("{:<8}  {}", raw_bytes, disassembly))
    }
}
