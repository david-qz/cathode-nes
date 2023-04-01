use std::{cell::Cell, fmt::Debug};

#[derive(Debug, Clone, Copy)]
pub enum Mnemonic {
    ADC,
    ALR,
    ANC,
    AND,
    ARR,
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
    DCP,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    ISC,
    JAM,
    JMP,
    JSR,
    LAS,
    LAX,
    LDA,
    LDX,
    LDY,
    LSR,
    LXA,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    RLA,
    ROL,
    ROR,
    RRA,
    RTI,
    RTS,
    SAX,
    SBC,
    SBX,
    SEC,
    SED,
    SEI,
    SHA,
    SHX,
    SHY,
    SLO,
    SRE,
    STA,
    STX,
    STY,
    TAS,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    XAA,
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
    pub illegal: bool,
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

    pub fn length(&self) -> u8 {
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

    pub fn mnemonic(&self) -> Mnemonic {
        self.disassembly().mnemonic
    }

    pub fn addressing_mode(&self) -> AddressingMode {
        self.disassembly().addressing_mode
    }

    pub fn illegal(&self) -> bool {
        self.disassembly().illegal
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
        let disassembly =
            |mnemonic: Mnemonic, addressing_mode: AddressingMode, illegal: bool| Disassembly {
                mnemonic,
                addressing_mode,
                illegal,
            };

        match self.opcode {
            0x00 => disassembly(Mnemonic::BRK, AddressingMode::Implied, false),
            0x01 => disassembly(Mnemonic::ORA, AddressingMode::IndirectX, false),
            0x02 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0x03 => disassembly(Mnemonic::SLO, AddressingMode::IndirectX, true),
            0x04 => disassembly(Mnemonic::NOP, AddressingMode::ZeroPage, true),
            0x05 => disassembly(Mnemonic::ORA, AddressingMode::ZeroPage, false),
            0x06 => disassembly(Mnemonic::ASL, AddressingMode::ZeroPage, false),
            0x07 => disassembly(Mnemonic::SLO, AddressingMode::ZeroPage, true),
            0x08 => disassembly(Mnemonic::PHP, AddressingMode::Implied, false),
            0x09 => disassembly(Mnemonic::ORA, AddressingMode::Immediate, false),
            0x0A => disassembly(Mnemonic::ASL, AddressingMode::Accumulator, false),
            0x0B => disassembly(Mnemonic::ANC, AddressingMode::Immediate, true),
            0x0C => disassembly(Mnemonic::NOP, AddressingMode::Implied, true),
            0x0D => disassembly(Mnemonic::ORA, AddressingMode::Absolute, false),
            0x0E => disassembly(Mnemonic::ASL, AddressingMode::Absolute, false),
            0x0F => disassembly(Mnemonic::SLO, AddressingMode::Absolute, true),

            0x10 => disassembly(Mnemonic::BPL, AddressingMode::Relative, false),
            0x11 => disassembly(Mnemonic::ORA, AddressingMode::IndirectY, false),
            0x12 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0x13 => disassembly(Mnemonic::SLO, AddressingMode::IndirectY, true),
            0x14 => disassembly(Mnemonic::NOP, AddressingMode::ZeroPageX, true),
            0x15 => disassembly(Mnemonic::ORA, AddressingMode::ZeroPageX, false),
            0x16 => disassembly(Mnemonic::ASL, AddressingMode::ZeroPageX, false),
            0x17 => disassembly(Mnemonic::SLO, AddressingMode::ZeroPageX, true),
            0x18 => disassembly(Mnemonic::CLC, AddressingMode::Implied, false),
            0x19 => disassembly(Mnemonic::ORA, AddressingMode::AbsoluteY, false),
            0x1A => disassembly(Mnemonic::NOP, AddressingMode::Implied, true),
            0x1B => disassembly(Mnemonic::SLO, AddressingMode::AbsoluteY, true),
            0x1C => disassembly(Mnemonic::NOP, AddressingMode::AbsoluteX, true),
            0x1D => disassembly(Mnemonic::ORA, AddressingMode::AbsoluteX, false),
            0x1E => disassembly(Mnemonic::ASL, AddressingMode::AbsoluteX, false),
            0x1F => disassembly(Mnemonic::SLO, AddressingMode::AbsoluteX, true),
            0x20 => disassembly(Mnemonic::JSR, AddressingMode::Absolute, false),
            0x21 => disassembly(Mnemonic::AND, AddressingMode::IndirectX, false),
            0x22 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0x23 => disassembly(Mnemonic::RLA, AddressingMode::IndirectX, true),
            0x24 => disassembly(Mnemonic::BIT, AddressingMode::ZeroPage, false),
            0x25 => disassembly(Mnemonic::AND, AddressingMode::ZeroPage, false),
            0x26 => disassembly(Mnemonic::ROL, AddressingMode::ZeroPage, false),
            0x27 => disassembly(Mnemonic::RLA, AddressingMode::ZeroPage, true),
            0x28 => disassembly(Mnemonic::PLP, AddressingMode::Implied, false),
            0x29 => disassembly(Mnemonic::AND, AddressingMode::Immediate, false),
            0x2A => disassembly(Mnemonic::ROL, AddressingMode::Accumulator, false),
            0x2B => disassembly(Mnemonic::ANC, AddressingMode::Immediate, true),
            0x2C => disassembly(Mnemonic::BIT, AddressingMode::Absolute, false),
            0x2D => disassembly(Mnemonic::AND, AddressingMode::Absolute, false),
            0x2E => disassembly(Mnemonic::ROL, AddressingMode::Absolute, false),
            0x2F => disassembly(Mnemonic::RLA, AddressingMode::Absolute, true),

            0x30 => disassembly(Mnemonic::BMI, AddressingMode::Relative, false),
            0x31 => disassembly(Mnemonic::AND, AddressingMode::IndirectY, false),
            0x32 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0x33 => disassembly(Mnemonic::RLA, AddressingMode::IndirectY, true),
            0x34 => disassembly(Mnemonic::NOP, AddressingMode::ZeroPageX, true),
            0x35 => disassembly(Mnemonic::AND, AddressingMode::ZeroPageX, false),
            0x36 => disassembly(Mnemonic::ROL, AddressingMode::ZeroPageX, false),
            0x37 => disassembly(Mnemonic::RLA, AddressingMode::ZeroPageX, true),
            0x38 => disassembly(Mnemonic::SEC, AddressingMode::Implied, false),
            0x39 => disassembly(Mnemonic::AND, AddressingMode::AbsoluteY, false),
            0x3A => disassembly(Mnemonic::NOP, AddressingMode::Implied, true),
            0x3B => disassembly(Mnemonic::RLA, AddressingMode::AbsoluteY, true),
            0x3C => disassembly(Mnemonic::NOP, AddressingMode::AbsoluteX, true),
            0x3D => disassembly(Mnemonic::AND, AddressingMode::AbsoluteX, false),
            0x3E => disassembly(Mnemonic::ROL, AddressingMode::AbsoluteX, false),
            0x3F => disassembly(Mnemonic::RLA, AddressingMode::AbsoluteX, true),

            0x40 => disassembly(Mnemonic::RTI, AddressingMode::Implied, false),
            0x41 => disassembly(Mnemonic::EOR, AddressingMode::IndirectX, false),
            0x42 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0x43 => disassembly(Mnemonic::SRE, AddressingMode::IndirectX, true),
            0x44 => disassembly(Mnemonic::NOP, AddressingMode::ZeroPage, true),
            0x45 => disassembly(Mnemonic::EOR, AddressingMode::ZeroPage, false),
            0x46 => disassembly(Mnemonic::LSR, AddressingMode::ZeroPage, false),
            0x47 => disassembly(Mnemonic::SRE, AddressingMode::ZeroPage, true),
            0x48 => disassembly(Mnemonic::PHA, AddressingMode::Implied, false),
            0x49 => disassembly(Mnemonic::EOR, AddressingMode::Immediate, false),
            0x4A => disassembly(Mnemonic::LSR, AddressingMode::Accumulator, false),
            0x4B => disassembly(Mnemonic::ALR, AddressingMode::Immediate, true),
            0x4C => disassembly(Mnemonic::JMP, AddressingMode::Absolute, false),
            0x4D => disassembly(Mnemonic::EOR, AddressingMode::Absolute, false),
            0x4E => disassembly(Mnemonic::LSR, AddressingMode::Absolute, false),
            0x4F => disassembly(Mnemonic::SRE, AddressingMode::Absolute, true),

            0x50 => disassembly(Mnemonic::BVC, AddressingMode::Relative, false),
            0x51 => disassembly(Mnemonic::EOR, AddressingMode::IndirectY, false),
            0x52 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0x53 => disassembly(Mnemonic::SRE, AddressingMode::IndirectY, true),
            0x54 => disassembly(Mnemonic::NOP, AddressingMode::ZeroPageX, true),
            0x55 => disassembly(Mnemonic::EOR, AddressingMode::ZeroPageX, false),
            0x56 => disassembly(Mnemonic::LSR, AddressingMode::ZeroPageX, false),
            0x57 => disassembly(Mnemonic::SRE, AddressingMode::ZeroPageX, true),
            0x58 => disassembly(Mnemonic::CLI, AddressingMode::Implied, false),
            0x59 => disassembly(Mnemonic::EOR, AddressingMode::AbsoluteY, false),
            0x5A => disassembly(Mnemonic::NOP, AddressingMode::Implied, true),
            0x5B => disassembly(Mnemonic::SRE, AddressingMode::AbsoluteY, true),
            0x5C => disassembly(Mnemonic::NOP, AddressingMode::AbsoluteX, true),
            0x5D => disassembly(Mnemonic::EOR, AddressingMode::AbsoluteX, false),
            0x5E => disassembly(Mnemonic::LSR, AddressingMode::AbsoluteX, false),
            0x5F => disassembly(Mnemonic::SRE, AddressingMode::AbsoluteX, true),

            0x60 => disassembly(Mnemonic::RTS, AddressingMode::Implied, false),
            0x61 => disassembly(Mnemonic::ADC, AddressingMode::IndirectX, false),
            0x62 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0x63 => disassembly(Mnemonic::RRA, AddressingMode::IndirectX, true),
            0x64 => disassembly(Mnemonic::NOP, AddressingMode::ZeroPage, true),
            0x65 => disassembly(Mnemonic::ADC, AddressingMode::ZeroPage, false),
            0x66 => disassembly(Mnemonic::ROR, AddressingMode::ZeroPage, false),
            0x67 => disassembly(Mnemonic::RRA, AddressingMode::ZeroPage, true),
            0x68 => disassembly(Mnemonic::PLA, AddressingMode::Implied, false),
            0x69 => disassembly(Mnemonic::ADC, AddressingMode::Immediate, false),
            0x6A => disassembly(Mnemonic::ROR, AddressingMode::Accumulator, false),
            0x6B => disassembly(Mnemonic::ARR, AddressingMode::Immediate, true),
            0x6C => disassembly(Mnemonic::JMP, AddressingMode::Indirect, false),
            0x6D => disassembly(Mnemonic::ADC, AddressingMode::Absolute, false),
            0x6E => disassembly(Mnemonic::ROR, AddressingMode::Absolute, false),
            0x6F => disassembly(Mnemonic::RRA, AddressingMode::Absolute, true),

            0x70 => disassembly(Mnemonic::BVS, AddressingMode::Relative, false),
            0x71 => disassembly(Mnemonic::ADC, AddressingMode::IndirectY, false),
            0x72 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0x73 => disassembly(Mnemonic::RRA, AddressingMode::IndirectY, true),
            0x74 => disassembly(Mnemonic::NOP, AddressingMode::ZeroPageX, true),
            0x75 => disassembly(Mnemonic::ADC, AddressingMode::ZeroPageX, false),
            0x76 => disassembly(Mnemonic::ROR, AddressingMode::ZeroPageX, false),
            0x77 => disassembly(Mnemonic::RRA, AddressingMode::ZeroPageX, true),
            0x78 => disassembly(Mnemonic::SEI, AddressingMode::Implied, false),
            0x79 => disassembly(Mnemonic::ADC, AddressingMode::AbsoluteY, false),
            0x7A => disassembly(Mnemonic::NOP, AddressingMode::Implied, true),
            0x7B => disassembly(Mnemonic::RRA, AddressingMode::AbsoluteY, true),
            0x7C => disassembly(Mnemonic::NOP, AddressingMode::AbsoluteY, true),
            0x7D => disassembly(Mnemonic::ADC, AddressingMode::AbsoluteX, false),
            0x7E => disassembly(Mnemonic::ROR, AddressingMode::AbsoluteX, false),
            0x7F => disassembly(Mnemonic::RRA, AddressingMode::AbsoluteX, true),

            0x80 => disassembly(Mnemonic::NOP, AddressingMode::Immediate, true),
            0x81 => disassembly(Mnemonic::STA, AddressingMode::IndirectX, false),
            0x82 => disassembly(Mnemonic::NOP, AddressingMode::Immediate, true),
            0x83 => disassembly(Mnemonic::SAX, AddressingMode::IndirectX, true),
            0x84 => disassembly(Mnemonic::STY, AddressingMode::ZeroPage, false),
            0x85 => disassembly(Mnemonic::STA, AddressingMode::ZeroPage, false),
            0x86 => disassembly(Mnemonic::STX, AddressingMode::ZeroPage, false),
            0x87 => disassembly(Mnemonic::SAX, AddressingMode::ZeroPage, true),
            0x88 => disassembly(Mnemonic::DEY, AddressingMode::Implied, false),
            0x89 => disassembly(Mnemonic::NOP, AddressingMode::Immediate, true),
            0x8A => disassembly(Mnemonic::TXA, AddressingMode::Implied, false),
            0x8B => disassembly(Mnemonic::XAA, AddressingMode::Immediate, true),
            0x8C => disassembly(Mnemonic::STY, AddressingMode::Absolute, false),
            0x8D => disassembly(Mnemonic::STA, AddressingMode::Absolute, false),
            0x8E => disassembly(Mnemonic::STX, AddressingMode::Absolute, false),
            0x8F => disassembly(Mnemonic::SAX, AddressingMode::Absolute, true),

            0x90 => disassembly(Mnemonic::BCC, AddressingMode::Relative, false),
            0x91 => disassembly(Mnemonic::STA, AddressingMode::IndirectY, false),
            0x92 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0x93 => disassembly(Mnemonic::SHA, AddressingMode::IndirectY, true),
            0x94 => disassembly(Mnemonic::STY, AddressingMode::ZeroPageX, false),
            0x95 => disassembly(Mnemonic::STA, AddressingMode::ZeroPageX, false),
            0x96 => disassembly(Mnemonic::STX, AddressingMode::ZeroPageY, false),
            0x97 => disassembly(Mnemonic::SAX, AddressingMode::ZeroPageY, true),
            0x98 => disassembly(Mnemonic::TYA, AddressingMode::Implied, false),
            0x99 => disassembly(Mnemonic::STA, AddressingMode::AbsoluteX, false),
            0x9A => disassembly(Mnemonic::TXS, AddressingMode::Implied, false),
            0x9B => disassembly(Mnemonic::TAS, AddressingMode::AbsoluteY, true),
            0x9C => disassembly(Mnemonic::SHY, AddressingMode::AbsoluteX, true),
            0x9D => disassembly(Mnemonic::STA, AddressingMode::AbsoluteX, false),
            0x9E => disassembly(Mnemonic::SHX, AddressingMode::AbsoluteY, true),
            0x9F => disassembly(Mnemonic::SHA, AddressingMode::AbsoluteY, true),

            0xA0 => disassembly(Mnemonic::LDY, AddressingMode::Immediate, false),
            0xA1 => disassembly(Mnemonic::LDA, AddressingMode::IndirectX, false),
            0xA2 => disassembly(Mnemonic::LDX, AddressingMode::Immediate, false),
            0xA3 => disassembly(Mnemonic::LAX, AddressingMode::IndirectX, true),
            0xA4 => disassembly(Mnemonic::LDY, AddressingMode::ZeroPage, false),
            0xA5 => disassembly(Mnemonic::LDA, AddressingMode::ZeroPage, false),
            0xA6 => disassembly(Mnemonic::LDX, AddressingMode::ZeroPage, false),
            0xA7 => disassembly(Mnemonic::LAX, AddressingMode::ZeroPage, true),
            0xA8 => disassembly(Mnemonic::TAY, AddressingMode::Implied, false),
            0xA9 => disassembly(Mnemonic::LDA, AddressingMode::Immediate, false),
            0xAA => disassembly(Mnemonic::TAX, AddressingMode::Implied, false),
            0xAB => disassembly(Mnemonic::LXA, AddressingMode::Immediate, true),
            0xAC => disassembly(Mnemonic::LDY, AddressingMode::Absolute, false),
            0xAD => disassembly(Mnemonic::LDA, AddressingMode::Absolute, false),
            0xAE => disassembly(Mnemonic::LDX, AddressingMode::Absolute, false),
            0xAF => disassembly(Mnemonic::LAX, AddressingMode::Absolute, true),

            0xB0 => disassembly(Mnemonic::BCS, AddressingMode::Relative, false),
            0xB1 => disassembly(Mnemonic::LDA, AddressingMode::IndirectY, false),
            0xB2 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0xB3 => disassembly(Mnemonic::LAX, AddressingMode::IndirectY, true),
            0xB4 => disassembly(Mnemonic::LDY, AddressingMode::ZeroPageX, false),
            0xB5 => disassembly(Mnemonic::LDA, AddressingMode::ZeroPageX, false),
            0xB6 => disassembly(Mnemonic::LDX, AddressingMode::ZeroPageY, false),
            0xB7 => disassembly(Mnemonic::LAX, AddressingMode::ZeroPageY, true),
            0xB8 => disassembly(Mnemonic::CLV, AddressingMode::Implied, false),
            0xB9 => disassembly(Mnemonic::LDA, AddressingMode::AbsoluteY, false),
            0xBA => disassembly(Mnemonic::TSX, AddressingMode::Implied, false),
            0xBB => disassembly(Mnemonic::LAS, AddressingMode::AbsoluteY, true),
            0xBC => disassembly(Mnemonic::LDY, AddressingMode::AbsoluteX, false),
            0xBD => disassembly(Mnemonic::LDA, AddressingMode::AbsoluteX, false),
            0xBE => disassembly(Mnemonic::LDX, AddressingMode::AbsoluteY, false),
            0xBF => disassembly(Mnemonic::LAX, AddressingMode::AbsoluteY, true),

            0xC0 => disassembly(Mnemonic::CPY, AddressingMode::Immediate, false),
            0xC1 => disassembly(Mnemonic::CMP, AddressingMode::IndirectX, false),
            0xC2 => disassembly(Mnemonic::NOP, AddressingMode::Immediate, true),
            0xC3 => disassembly(Mnemonic::DCP, AddressingMode::IndirectX, true),
            0xC4 => disassembly(Mnemonic::CPY, AddressingMode::ZeroPage, false),
            0xC5 => disassembly(Mnemonic::CMP, AddressingMode::ZeroPage, false),
            0xC6 => disassembly(Mnemonic::DEC, AddressingMode::ZeroPage, false),
            0xC7 => disassembly(Mnemonic::DCP, AddressingMode::ZeroPage, true),
            0xC8 => disassembly(Mnemonic::INY, AddressingMode::Implied, false),
            0xC9 => disassembly(Mnemonic::CMP, AddressingMode::Immediate, false),
            0xCA => disassembly(Mnemonic::DEX, AddressingMode::Implied, false),
            0xCB => disassembly(Mnemonic::SBX, AddressingMode::Immediate, true),
            0xCC => disassembly(Mnemonic::CPY, AddressingMode::Absolute, false),
            0xCD => disassembly(Mnemonic::CMP, AddressingMode::Absolute, false),
            0xCE => disassembly(Mnemonic::DEC, AddressingMode::Absolute, false),
            0xCF => disassembly(Mnemonic::DCP, AddressingMode::Absolute, true),

            0xD0 => disassembly(Mnemonic::BNE, AddressingMode::Relative, false),
            0xD1 => disassembly(Mnemonic::CMP, AddressingMode::IndirectY, false),
            0xD2 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0xD3 => disassembly(Mnemonic::DCP, AddressingMode::IndirectY, true),
            0xD4 => disassembly(Mnemonic::NOP, AddressingMode::ZeroPageX, true),
            0xD5 => disassembly(Mnemonic::CMP, AddressingMode::ZeroPageX, false),
            0xD6 => disassembly(Mnemonic::DEC, AddressingMode::ZeroPageX, false),
            0xD7 => disassembly(Mnemonic::DCP, AddressingMode::ZeroPageX, true),
            0xD8 => disassembly(Mnemonic::CLD, AddressingMode::Implied, false),
            0xD9 => disassembly(Mnemonic::CMP, AddressingMode::AbsoluteY, false),
            0xDA => disassembly(Mnemonic::NOP, AddressingMode::Implied, true),
            0xDB => disassembly(Mnemonic::DCP, AddressingMode::AbsoluteY, true),
            0xDC => disassembly(Mnemonic::NOP, AddressingMode::AbsoluteX, true),
            0xDD => disassembly(Mnemonic::CMP, AddressingMode::AbsoluteX, false),
            0xDE => disassembly(Mnemonic::DEC, AddressingMode::AbsoluteX, false),
            0xDF => disassembly(Mnemonic::DCP, AddressingMode::AbsoluteX, true),

            0xE0 => disassembly(Mnemonic::CPX, AddressingMode::Immediate, false),
            0xE1 => disassembly(Mnemonic::SBC, AddressingMode::IndirectX, false),
            0xE2 => disassembly(Mnemonic::NOP, AddressingMode::Immediate, true),
            0xE3 => disassembly(Mnemonic::ISC, AddressingMode::IndirectX, true),
            0xE4 => disassembly(Mnemonic::CPX, AddressingMode::ZeroPage, false),
            0xE5 => disassembly(Mnemonic::SBC, AddressingMode::ZeroPage, false),
            0xE6 => disassembly(Mnemonic::INC, AddressingMode::ZeroPage, false),
            0xE7 => disassembly(Mnemonic::ISC, AddressingMode::ZeroPage, true),
            0xE8 => disassembly(Mnemonic::INX, AddressingMode::Implied, false),
            0xE9 => disassembly(Mnemonic::SBC, AddressingMode::Immediate, false),
            0xEA => disassembly(Mnemonic::NOP, AddressingMode::Implied, false),
            0xEB => disassembly(Mnemonic::SBC, AddressingMode::Immediate, true),
            0xEC => disassembly(Mnemonic::CPX, AddressingMode::Absolute, false),
            0xED => disassembly(Mnemonic::SBC, AddressingMode::Absolute, false),
            0xEE => disassembly(Mnemonic::INC, AddressingMode::Absolute, false),
            0xEF => disassembly(Mnemonic::ISC, AddressingMode::Absolute, true),

            0xF0 => disassembly(Mnemonic::BEQ, AddressingMode::Relative, false),
            0xF1 => disassembly(Mnemonic::SBC, AddressingMode::IndirectY, false),
            0xF2 => disassembly(Mnemonic::JAM, AddressingMode::Implied, true),
            0xF3 => disassembly(Mnemonic::ISC, AddressingMode::IndirectY, true),
            0xF4 => disassembly(Mnemonic::NOP, AddressingMode::ZeroPageX, true),
            0xF5 => disassembly(Mnemonic::SBC, AddressingMode::ZeroPageX, false),
            0xF6 => disassembly(Mnemonic::INC, AddressingMode::ZeroPageX, false),
            0xF7 => disassembly(Mnemonic::ISC, AddressingMode::ZeroPageX, true),
            0xF8 => disassembly(Mnemonic::SED, AddressingMode::Implied, false),
            0xF9 => disassembly(Mnemonic::SBC, AddressingMode::AbsoluteY, false),
            0xFA => disassembly(Mnemonic::NOP, AddressingMode::Implied, true),
            0xFB => disassembly(Mnemonic::ISC, AddressingMode::AbsoluteY, true),
            0xFC => disassembly(Mnemonic::NOP, AddressingMode::AbsoluteX, true),
            0xFD => disassembly(Mnemonic::SBC, AddressingMode::AbsoluteX, false),
            0xFE => disassembly(Mnemonic::INC, AddressingMode::AbsoluteX, false),
            0xFF => disassembly(Mnemonic::ISC, AddressingMode::AbsoluteX, true),
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
            illegal,
        } = self.disassembly();

        let raw_bytes = match self.length() {
            1 => format!("{:02X}", opcode),
            2 => format!("{:02X} {:02X}", opcode, operand1),
            3 => format!("{:02X} {:02X} {:02X}", opcode, operand1, operand2),
            _ => unreachable!(),
        };

        let legality = if illegal { "*" } else { " " };

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

        f.pad(&format!("{:<8} {}{}", raw_bytes, legality, disassembly))
    }
}
