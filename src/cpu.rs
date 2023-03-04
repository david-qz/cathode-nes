use crate::bus::Bus16;

enum AddressingMode {
    Accumulator,
    Immediate,
    Absolute,
    ZeroPage,
    IndexedZeroPageX,
    IndexedZeroPageY,
    IndexedAbsoluteX,
    IndexedAbsoluteY,
    IndexedIndirectX,
    IndexedIndirectY,
    AbsoluteIndirect,
}

/// A MOS 6502 CPU
pub struct CPU {
    a: u8,
    x: u8,
    y: u8,

    pc: u16,
    s: u8,

    carry: bool,
    zero: bool,
    irq_disable: bool,
    decimal_mode: bool,
    brk_command: bool,
    overflow: bool,
    negative: bool,

    should_run_reset_procedure: bool,
}

impl CPU {
    const RESET_VECTOR: u16 = 0xFFFA;

    // The 6502 uses a descending stack.
    const STACK_BOTTOM: u16 = 0x01FF;
    const STACK_TOP: u16 = 0x0100;

    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0,
            carry: false,
            zero: false,
            irq_disable: false,
            decimal_mode: false,
            brk_command: false,
            overflow: false,
            negative: false,
            should_run_reset_procedure: true,
        }
    }

    pub fn reset(&mut self, bus: &mut dyn Bus16) {
        self.pc = bus.read_word(Self::RESET_VECTOR);
        self.s = 0xFF;
    }

    pub fn clock(&mut self, bus: &mut dyn Bus16) {
        if self.should_run_reset_procedure {
            self.reset(bus);
            return;
        }

        let opcode = bus.read_byte(self.pc);

        match opcode {
            // ADC
            0x69 => self.adc(AddressingMode::Immediate),
            0x6D => self.adc(AddressingMode::Absolute),
            0x65 => self.adc(AddressingMode::ZeroPage),
            0x61 => self.adc(AddressingMode::IndexedIndirectX),
            0x71 => self.adc(AddressingMode::IndexedIndirectY),
            0x75 => self.adc(AddressingMode::IndexedZeroPageX),
            0x7D => self.adc(AddressingMode::IndexedAbsoluteX),
            0x79 => self.adc(AddressingMode::IndexedAbsoluteY),
            // AND
            0x29 => self.and(AddressingMode::Immediate),
            0x2D => self.and(AddressingMode::Absolute),
            0x25 => self.and(AddressingMode::ZeroPage),
            0x21 => self.and(AddressingMode::IndexedIndirectX),
            0x31 => self.and(AddressingMode::IndexedIndirectY),
            0x35 => self.and(AddressingMode::IndexedZeroPageX),
            0x3D => self.and(AddressingMode::IndexedAbsoluteX),
            0x39 => self.and(AddressingMode::IndexedAbsoluteY),
            // ASL
            0x0E => self.asl(AddressingMode::Absolute),
            0x06 => self.asl(AddressingMode::ZeroPage),
            0x0A => self.asl(AddressingMode::Accumulator),
            0x16 => self.asl(AddressingMode::IndexedZeroPageX),
            0x1E => self.asl(AddressingMode::IndexedAbsoluteX),
            // BCC
            0x90 => self.bcc(),
            // BCS
            0xB0 => self.bcs(),
            // BEQ
            0xF0 => self.beq(),
            // BIT
            0x2C => self.bit(AddressingMode::Absolute),
            0x24 => self.bit(AddressingMode::ZeroPage),
            // BMI
            0x30 => self.bmi(),
            // BNE
            0xD0 => self.bne(),
            // BPL
            0x10 => self.bpl(),
            // BRK
            0x00 => self.brk(),
            // BVC
            0x50 => self.bvc(),
            // BVS
            0x70 => self.bvs(),
            // CLC
            0x18 => self.clc(),
            // CLD
            0xD8 => self.cld(),
            // CLI
            0x58 => self.cli(),
            // CLV
            0xB8 => self.clv(),
            //CMP
            0xC9 => self.cmp(AddressingMode::Immediate),
            0xCD => self.cmp(AddressingMode::Absolute),
            0xC5 => self.cmp(AddressingMode::ZeroPage),
            0xC1 => self.cmp(AddressingMode::IndexedIndirectX),
            0xD1 => self.cmp(AddressingMode::IndexedIndirectY),
            0xD5 => self.cmp(AddressingMode::IndexedZeroPageX),
            0xDD => self.cmp(AddressingMode::IndexedAbsoluteX),
            0xD9 => self.cmp(AddressingMode::IndexedAbsoluteY),
            // CPX
            0xE0 => self.cpx(AddressingMode::Immediate),
            0xEC => self.cpx(AddressingMode::Absolute),
            0xE4 => self.cpx(AddressingMode::ZeroPage),
            // CPY
            0xC0 => self.cpy(AddressingMode::Immediate),
            0xCC => self.cpy(AddressingMode::Absolute),
            0xC4 => self.cpy(AddressingMode::ZeroPage),
            // DEC
            0xCE => self.dec(AddressingMode::Absolute),
            0xC6 => self.dec(AddressingMode::ZeroPage),
            0xD6 => self.dec(AddressingMode::IndexedZeroPageX),
            0xDE => self.dec(AddressingMode::IndexedAbsoluteX),
            // DEX
            0xCA => self.dex(),
            // DEY
            0x88 => self.dey(),
            // EOR
            0x49 => self.eor(AddressingMode::Immediate),
            0x4D => self.eor(AddressingMode::Absolute),
            0x45 => self.eor(AddressingMode::ZeroPage),
            0x41 => self.eor(AddressingMode::IndexedIndirectX),
            0x51 => self.eor(AddressingMode::IndexedIndirectY),
            0x55 => self.eor(AddressingMode::IndexedZeroPageX),
            0x5D => self.eor(AddressingMode::IndexedAbsoluteX),
            0x59 => self.eor(AddressingMode::IndexedAbsoluteY),
            // INC
            0xEE => self.inc(AddressingMode::Absolute),
            0xE6 => self.inc(AddressingMode::ZeroPage),
            0xF6 => self.inc(AddressingMode::IndexedZeroPageX),
            0xFE => self.inc(AddressingMode::IndexedAbsoluteX),
            // INX
            0xE8 => self.inx(),
            // INY
            0xC8 => self.iny(),
            // JMP
            0x4C => self.jmp(AddressingMode::ZeroPage),
            0x6C => self.jmp(AddressingMode::AbsoluteIndirect),
            // JSR
            0x20 => self.jsr(),
            // LDA
            0xA9 => self.lda(AddressingMode::Immediate),
            0xAD => self.lda(AddressingMode::Absolute),
            0xA5 => self.lda(AddressingMode::ZeroPage),
            0xA1 => self.lda(AddressingMode::IndexedIndirectX),
            0xB1 => self.lda(AddressingMode::IndexedIndirectY),
            0xB5 => self.lda(AddressingMode::IndexedZeroPageX),
            0xBD => self.lda(AddressingMode::IndexedAbsoluteX),
            0xB9 => self.lda(AddressingMode::IndexedAbsoluteY),
            // LDX
            0xA2 => self.ldx(AddressingMode::Immediate),
            0xAE => self.ldx(AddressingMode::Absolute),
            0xA6 => self.ldx(AddressingMode::ZeroPage),
            0xBE => self.ldx(AddressingMode::IndexedAbsoluteY),
            0xB6 => self.ldx(AddressingMode::IndexedZeroPageY),
            // LDY
            0xA0 => self.ldy(AddressingMode::Immediate),
            0xAC => self.ldy(AddressingMode::Absolute),
            0xA4 => self.ldy(AddressingMode::ZeroPage),
            0xB4 => self.ldy(AddressingMode::IndexedZeroPageX),
            0xBC => self.ldy(AddressingMode::IndexedAbsoluteX),
            // LSR
            0x4E => self.lsr(AddressingMode::Absolute),
            0x46 => self.lsr(AddressingMode::ZeroPage),
            0x4A => self.lsr(AddressingMode::Accumulator),
            0x56 => self.lsr(AddressingMode::IndexedZeroPageX),
            0x5E => self.lsr(AddressingMode::IndexedAbsoluteX),
            // NOP
            0xEA => (),
            // ORA
            0x09 => self.ora(AddressingMode::Immediate),
            0x0D => self.ora(AddressingMode::Absolute),
            0x05 => self.ora(AddressingMode::ZeroPage),
            0x01 => self.ora(AddressingMode::IndexedIndirectX),
            0x11 => self.ora(AddressingMode::IndexedIndirectY),
            0x15 => self.ora(AddressingMode::IndexedZeroPageX),
            0x1D => self.ora(AddressingMode::IndexedAbsoluteX),
            0x19 => self.ora(AddressingMode::IndexedAbsoluteY),
            // PHA
            0x48 => self.pha(),
            // PHP
            0x08 => self.php(),
            // PLA
            0x68 => self.pla(),
            // PLP
            0x28 => self.plp(),
            // ROL
            0x2E => self.rol(AddressingMode::Absolute),
            0x26 => self.rol(AddressingMode::ZeroPage),
            0x2A => self.rol(AddressingMode::Accumulator),
            0x36 => self.rol(AddressingMode::IndexedZeroPageX),
            0x3E => self.rol(AddressingMode::IndexedAbsoluteX),
            // ROR
            0x6E => self.ror(AddressingMode::Absolute),
            0x66 => self.ror(AddressingMode::ZeroPage),
            0x6A => self.ror(AddressingMode::Accumulator),
            0x76 => self.ror(AddressingMode::IndexedZeroPageX),
            0x7E => self.ror(AddressingMode::IndexedAbsoluteX),
            // RTI
            0x40 => self.rti(),
            // RTS
            0x60 => self.rts(),
            // SBC
            0xE9 => self.sbc(AddressingMode::Immediate),
            0xED => self.sbc(AddressingMode::Absolute),
            0xE5 => self.sbc(AddressingMode::ZeroPage),
            0xE1 => self.sbc(AddressingMode::IndexedIndirectX),
            0xF1 => self.sbc(AddressingMode::IndexedIndirectY),
            0xF5 => self.sbc(AddressingMode::IndexedZeroPageX),
            0xFD => self.sbc(AddressingMode::IndexedAbsoluteX),
            0xF9 => self.sbc(AddressingMode::IndexedAbsoluteY),
            // SEC
            0x38 => self.sec(),
            // SED
            0xF8 => self.sed(),
            // SEI
            0x78 => self.sei(),
            // STA
            0x8D => self.sta(AddressingMode::Absolute),
            0x85 => self.sta(AddressingMode::ZeroPage),
            0x81 => self.sta(AddressingMode::IndexedIndirectX),
            0x91 => self.sta(AddressingMode::IndexedIndirectY),
            0x95 => self.sta(AddressingMode::IndexedZeroPageX),
            0x9D => self.sta(AddressingMode::IndexedAbsoluteX),
            0x99 => self.sta(AddressingMode::IndexedAbsoluteY),
            // STX
            0x8E => self.stx(AddressingMode::Absolute),
            0x86 => self.stx(AddressingMode::ZeroPage),
            0x96 => self.stx(AddressingMode::IndexedZeroPageY),
            // STY
            0x8C => self.sty(AddressingMode::Absolute),
            0x84 => self.sty(AddressingMode::ZeroPage),
            0x94 => self.sty(AddressingMode::IndexedZeroPageX),
            // TAX
            0xAA => self.tax(),
            // TAY
            0xA8 => self.tay(),
            // TSX
            0xBA => self.tsx(),
            // TXA
            0x8A => self.txa(),
            // TXS
            0x9A => self.txs(),
            // TYA
            0x98 => self.tya(),
            _ => panic!("Unknown opcode: {}", opcode),
        }
    }

    fn adc(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'ADC'");
    }

    fn and(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'AND'");
    }

    fn asl(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'AND'");
    }

    fn bcc(&mut self) {
        panic!("Unimplemented opcode 'BCC'");
    }

    fn bcs(&mut self) {
        panic!("Unimplemented opcode 'BCS'");
    }

    fn beq(&mut self) {
        panic!("Unimplemented opcode 'BCS'");
    }

    fn bit(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'BIT'");
    }

    fn bmi(&mut self) {
        panic!("Unimplemented opcode 'BMI'");
    }

    fn bne(&mut self) {
        panic!("Unimplemented opcode 'BNE'");
    }

    fn bpl(&mut self) {
        panic!("Unimplemented opcode 'BPL'");
    }

    fn brk(&mut self) {
        panic!("Unimplemented opcode 'BRK'");
    }

    fn bvc(&mut self) {
        panic!("Unimplemented opcode 'BVC'");
    }

    fn bvs(&mut self) {
        panic!("Unimplemented opcode 'BVS'");
    }

    fn clc(&mut self) {
        panic!("Unimplemented opcode 'CLC'");
    }

    fn cld(&mut self) {
        panic!("Unimplemented opcode 'CLD'");
    }

    fn cli(&mut self) {
        panic!("Unimplemented opcode 'CLI'");
    }

    fn clv(&mut self) {
        panic!("Unimplemented opcode 'CLV'");
    }

    fn cmp(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'CMP'");
    }

    fn cpx(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'CPX'");
    }

    fn cpy(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'CPY'");
    }

    fn dec(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'DEC'");
    }

    fn dex(&mut self) {
        panic!("Unimplemented opcode 'DEX'");
    }

    fn dey(&mut self) {
        panic!("Unimplemented opcode 'DEY'");
    }

    fn eor(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'EOR'");
    }

    fn inc(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'INC'");
    }

    fn inx(&mut self) {
        panic!("Unimplemented opcode 'INX'");
    }

    fn iny(&mut self) {
        panic!("Unimplemented opcode 'INY'");
    }

    fn jmp(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'JMP'");
    }

    fn jsr(&mut self) {
        panic!("Unimplemented opcode 'JSR'");
    }

    fn lda(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'LDA'");
    }

    fn ldx(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'LDX'");
    }

    fn ldy(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'LDY'");
    }

    fn lsr(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'LSR'");
    }

    fn ora(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'ORA'");
    }

    fn pha(&mut self) {
        panic!("Unimplemented opcode 'PHA'");
    }

    fn php(&mut self) {
        panic!("Unimplemented opcode 'PHP'");
    }

    fn pla(&mut self) {
        panic!("Unimplemented opcode 'PLA'");
    }

    fn plp(&mut self) {
        panic!("Unimplemented opcode 'PLP'");
    }

    fn rol(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'ROL'");
    }

    fn ror(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'ROR'");
    }

    fn rti(&mut self) {
        panic!("Unimplemented opcode 'RTI'");
    }

    fn rts(&mut self) {
        panic!("Unimplemented opcode 'RTS'");
    }

    fn sbc(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'SBC'");
    }

    fn sec(&mut self) {
        panic!("Unimplemented opcode 'SEC'");
    }

    fn sed(&mut self) {
        panic!("Unimplemented opcode 'SED'");
    }

    fn sei(&mut self) {
        panic!("Unimplemented opcode 'SEI'");
    }

    fn sta(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'STA'");
    }

    fn stx(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'STX'");
    }

    fn sty(&mut self, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'STY'");
    }

    fn tax(&mut self) {
        panic!("Unimplemented opcode 'TAX'");
    }

    fn tay(&mut self) {
        panic!("Unimplemented opcode 'TAY'");
    }

    fn tsx(&mut self) {
        panic!("Unimplemented opcode 'TSX'");
    }

    fn txa(&mut self) {
        panic!("Unimplemented opcode 'TXA'");
    }

    fn txs(&mut self) {
        panic!("Unimplemented opcode 'TXS'");
    }

    fn tya(&mut self) {
        panic!("Unimplemented opcode 'TYA'");
    }
}
