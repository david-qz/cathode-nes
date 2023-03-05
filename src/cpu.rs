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
    Relative,
    Implied,
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

    fn resolve_address(&mut self, bus: &mut dyn Bus16, addressing_mode: &AddressingMode) -> u16 {
        match addressing_mode {
            AddressingMode::Immediate => self.pc + 1,
            AddressingMode::Absolute => bus.read_word(self.pc + 1),
            AddressingMode::ZeroPage => bus.read_byte(self.pc + 1) as u16,
            AddressingMode::IndexedZeroPageX => {
                let base_address_zero_page = bus.read_byte(self.pc + 1);
                base_address_zero_page.wrapping_add(self.x) as u16
            }
            AddressingMode::IndexedZeroPageY => {
                let base_address_zero_page = bus.read_byte(self.pc + 1);
                base_address_zero_page.wrapping_add(self.y) as u16
            }
            AddressingMode::IndexedAbsoluteX => {
                let base_address = bus.read_word(self.pc + 1);
                base_address.wrapping_add(self.x as u16)
            }
            AddressingMode::IndexedAbsoluteY => {
                let base_address = bus.read_word(self.pc + 1);
                base_address.wrapping_add(self.y as u16)
            }
            AddressingMode::IndexedIndirectX => {
                let indirect_base_address_zero_page = bus.read_byte(self.pc + 1);
                let indirect_address = indirect_base_address_zero_page.wrapping_add(self.x) as u16;
                bus.read_word(indirect_address)
            }
            AddressingMode::IndexedIndirectY => {
                let indirect_address_zero_page = bus.read_byte(self.pc + 1);
                let base_address = bus.read_word(indirect_address_zero_page as u16);
                base_address.wrapping_add(self.y as u16)
            }
            AddressingMode::AbsoluteIndirect => {
                let indirect_address = bus.read_word(self.pc + 1);
                bus.read_word(indirect_address)
            }
            AddressingMode::Relative => {
                let offset: i8 = unsafe { std::mem::transmute(bus.read_byte(self.pc + 1)) };
                self.pc.wrapping_add_signed(offset as i16)
            }
            AddressingMode::Accumulator => {
                panic!("Attempt to resolve address of accumulator register!")
            }
            AddressingMode::Implied => {
                panic!("Attempt to resolve address in implied addressing mode!")
            }
        }
    }

    fn instruction_length(addressing_mode: &AddressingMode) -> u16 {
        match addressing_mode {
            AddressingMode::Immediate => 2,
            AddressingMode::Absolute => 3,
            AddressingMode::ZeroPage => 2,
            AddressingMode::Accumulator => 1,
            AddressingMode::IndexedZeroPageX => 2,
            AddressingMode::IndexedZeroPageY => 2,
            AddressingMode::IndexedIndirectX => 2,
            AddressingMode::IndexedIndirectY => 2,
            AddressingMode::IndexedAbsoluteX => 3,
            AddressingMode::IndexedAbsoluteY => 3,
            AddressingMode::AbsoluteIndirect => 3,
            AddressingMode::Relative => 2,
            AddressingMode::Implied => 1,
        }
    }

    pub fn clock(&mut self, bus: &mut dyn Bus16) {
        if self.should_run_reset_procedure {
            self.reset(bus);
            return;
        }

        let opcode = bus.read_byte(self.pc);

        match opcode {
            // ADC
            0x69 => self.adc(bus, AddressingMode::Immediate),
            0x6D => self.adc(bus, AddressingMode::Absolute),
            0x65 => self.adc(bus, AddressingMode::ZeroPage),
            0x61 => self.adc(bus, AddressingMode::IndexedIndirectX),
            0x71 => self.adc(bus, AddressingMode::IndexedIndirectY),
            0x75 => self.adc(bus, AddressingMode::IndexedZeroPageX),
            0x7D => self.adc(bus, AddressingMode::IndexedAbsoluteX),
            0x79 => self.adc(bus, AddressingMode::IndexedAbsoluteY),
            // AND
            0x29 => self.and(bus, AddressingMode::Immediate),
            0x2D => self.and(bus, AddressingMode::Absolute),
            0x25 => self.and(bus, AddressingMode::ZeroPage),
            0x21 => self.and(bus, AddressingMode::IndexedIndirectX),
            0x31 => self.and(bus, AddressingMode::IndexedIndirectY),
            0x35 => self.and(bus, AddressingMode::IndexedZeroPageX),
            0x3D => self.and(bus, AddressingMode::IndexedAbsoluteX),
            0x39 => self.and(bus, AddressingMode::IndexedAbsoluteY),
            // ASL
            0x0E => self.asl(bus, AddressingMode::Absolute),
            0x06 => self.asl(bus, AddressingMode::ZeroPage),
            0x0A => self.asl(bus, AddressingMode::Accumulator),
            0x16 => self.asl(bus, AddressingMode::IndexedZeroPageX),
            0x1E => self.asl(bus, AddressingMode::IndexedAbsoluteX),
            // BCC
            0x90 => self.bcc(),
            // BCS
            0xB0 => self.bcs(),
            // BEQ
            0xF0 => self.beq(),
            // BIT
            0x2C => self.bit(bus, AddressingMode::Absolute),
            0x24 => self.bit(bus, AddressingMode::ZeroPage),
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
            0xC9 => self.cmp(bus, AddressingMode::Immediate),
            0xCD => self.cmp(bus, AddressingMode::Absolute),
            0xC5 => self.cmp(bus, AddressingMode::ZeroPage),
            0xC1 => self.cmp(bus, AddressingMode::IndexedIndirectX),
            0xD1 => self.cmp(bus, AddressingMode::IndexedIndirectY),
            0xD5 => self.cmp(bus, AddressingMode::IndexedZeroPageX),
            0xDD => self.cmp(bus, AddressingMode::IndexedAbsoluteX),
            0xD9 => self.cmp(bus, AddressingMode::IndexedAbsoluteY),
            // CPX
            0xE0 => self.cpx(bus, AddressingMode::Immediate),
            0xEC => self.cpx(bus, AddressingMode::Absolute),
            0xE4 => self.cpx(bus, AddressingMode::ZeroPage),
            // CPY
            0xC0 => self.cpy(bus, AddressingMode::Immediate),
            0xCC => self.cpy(bus, AddressingMode::Absolute),
            0xC4 => self.cpy(bus, AddressingMode::ZeroPage),
            // DEC
            0xCE => self.dec(bus, AddressingMode::Absolute),
            0xC6 => self.dec(bus, AddressingMode::ZeroPage),
            0xD6 => self.dec(bus, AddressingMode::IndexedZeroPageX),
            0xDE => self.dec(bus, AddressingMode::IndexedAbsoluteX),
            // DEX
            0xCA => self.dex(),
            // DEY
            0x88 => self.dey(),
            // EOR
            0x49 => self.eor(bus, AddressingMode::Immediate),
            0x4D => self.eor(bus, AddressingMode::Absolute),
            0x45 => self.eor(bus, AddressingMode::ZeroPage),
            0x41 => self.eor(bus, AddressingMode::IndexedIndirectX),
            0x51 => self.eor(bus, AddressingMode::IndexedIndirectY),
            0x55 => self.eor(bus, AddressingMode::IndexedZeroPageX),
            0x5D => self.eor(bus, AddressingMode::IndexedAbsoluteX),
            0x59 => self.eor(bus, AddressingMode::IndexedAbsoluteY),
            // INC
            0xEE => self.inc(bus, AddressingMode::Absolute),
            0xE6 => self.inc(bus, AddressingMode::ZeroPage),
            0xF6 => self.inc(bus, AddressingMode::IndexedZeroPageX),
            0xFE => self.inc(bus, AddressingMode::IndexedAbsoluteX),
            // INX
            0xE8 => self.inx(),
            // INY
            0xC8 => self.iny(),
            // JMP
            0x4C => self.jmp(bus, AddressingMode::ZeroPage),
            0x6C => self.jmp(bus, AddressingMode::AbsoluteIndirect),
            // JSR
            0x20 => self.jsr(),
            // LDA
            0xA9 => self.lda(bus, AddressingMode::Immediate),
            0xAD => self.lda(bus, AddressingMode::Absolute),
            0xA5 => self.lda(bus, AddressingMode::ZeroPage),
            0xA1 => self.lda(bus, AddressingMode::IndexedIndirectX),
            0xB1 => self.lda(bus, AddressingMode::IndexedIndirectY),
            0xB5 => self.lda(bus, AddressingMode::IndexedZeroPageX),
            0xBD => self.lda(bus, AddressingMode::IndexedAbsoluteX),
            0xB9 => self.lda(bus, AddressingMode::IndexedAbsoluteY),
            // LDX
            0xA2 => self.ldx(bus, AddressingMode::Immediate),
            0xAE => self.ldx(bus, AddressingMode::Absolute),
            0xA6 => self.ldx(bus, AddressingMode::ZeroPage),
            0xBE => self.ldx(bus, AddressingMode::IndexedAbsoluteY),
            0xB6 => self.ldx(bus, AddressingMode::IndexedZeroPageY),
            // LDY
            0xA0 => self.ldy(bus, AddressingMode::Immediate),
            0xAC => self.ldy(bus, AddressingMode::Absolute),
            0xA4 => self.ldy(bus, AddressingMode::ZeroPage),
            0xB4 => self.ldy(bus, AddressingMode::IndexedZeroPageX),
            0xBC => self.ldy(bus, AddressingMode::IndexedAbsoluteX),
            // LSR
            0x4E => self.lsr(bus, AddressingMode::Absolute),
            0x46 => self.lsr(bus, AddressingMode::ZeroPage),
            0x4A => self.lsr(bus, AddressingMode::Accumulator),
            0x56 => self.lsr(bus, AddressingMode::IndexedZeroPageX),
            0x5E => self.lsr(bus, AddressingMode::IndexedAbsoluteX),
            // NOP
            0xEA => (),
            // ORA
            0x09 => self.ora(bus, AddressingMode::Immediate),
            0x0D => self.ora(bus, AddressingMode::Absolute),
            0x05 => self.ora(bus, AddressingMode::ZeroPage),
            0x01 => self.ora(bus, AddressingMode::IndexedIndirectX),
            0x11 => self.ora(bus, AddressingMode::IndexedIndirectY),
            0x15 => self.ora(bus, AddressingMode::IndexedZeroPageX),
            0x1D => self.ora(bus, AddressingMode::IndexedAbsoluteX),
            0x19 => self.ora(bus, AddressingMode::IndexedAbsoluteY),
            // PHA
            0x48 => self.pha(),
            // PHP
            0x08 => self.php(),
            // PLA
            0x68 => self.pla(),
            // PLP
            0x28 => self.plp(),
            // ROL
            0x2E => self.rol(bus, AddressingMode::Absolute),
            0x26 => self.rol(bus, AddressingMode::ZeroPage),
            0x2A => self.rol(bus, AddressingMode::Accumulator),
            0x36 => self.rol(bus, AddressingMode::IndexedZeroPageX),
            0x3E => self.rol(bus, AddressingMode::IndexedAbsoluteX),
            // ROR
            0x6E => self.ror(bus, AddressingMode::Absolute),
            0x66 => self.ror(bus, AddressingMode::ZeroPage),
            0x6A => self.ror(bus, AddressingMode::Accumulator),
            0x76 => self.ror(bus, AddressingMode::IndexedZeroPageX),
            0x7E => self.ror(bus, AddressingMode::IndexedAbsoluteX),
            // RTI
            0x40 => self.rti(),
            // RTS
            0x60 => self.rts(),
            // SBC
            0xE9 => self.sbc(bus, AddressingMode::Immediate),
            0xED => self.sbc(bus, AddressingMode::Absolute),
            0xE5 => self.sbc(bus, AddressingMode::ZeroPage),
            0xE1 => self.sbc(bus, AddressingMode::IndexedIndirectX),
            0xF1 => self.sbc(bus, AddressingMode::IndexedIndirectY),
            0xF5 => self.sbc(bus, AddressingMode::IndexedZeroPageX),
            0xFD => self.sbc(bus, AddressingMode::IndexedAbsoluteX),
            0xF9 => self.sbc(bus, AddressingMode::IndexedAbsoluteY),
            // SEC
            0x38 => self.sec(),
            // SED
            0xF8 => self.sed(),
            // SEI
            0x78 => self.sei(),
            // STA
            0x8D => self.sta(bus, AddressingMode::Absolute),
            0x85 => self.sta(bus, AddressingMode::ZeroPage),
            0x81 => self.sta(bus, AddressingMode::IndexedIndirectX),
            0x91 => self.sta(bus, AddressingMode::IndexedIndirectY),
            0x95 => self.sta(bus, AddressingMode::IndexedZeroPageX),
            0x9D => self.sta(bus, AddressingMode::IndexedAbsoluteX),
            0x99 => self.sta(bus, AddressingMode::IndexedAbsoluteY),
            // STX
            0x8E => self.stx(bus, AddressingMode::Absolute),
            0x86 => self.stx(bus, AddressingMode::ZeroPage),
            0x96 => self.stx(bus, AddressingMode::IndexedZeroPageY),
            // STY
            0x8C => self.sty(bus, AddressingMode::Absolute),
            0x84 => self.sty(bus, AddressingMode::ZeroPage),
            0x94 => self.sty(bus, AddressingMode::IndexedZeroPageX),
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

    fn adc(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'ADC'");
    }

    fn and(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'AND'");
    }

    fn asl(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
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

    fn bit(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
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

    fn cmp(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'CMP'");
    }

    fn cpx(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'CPX'");
    }

    fn cpy(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'CPY'");
    }

    fn dec(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'DEC'");
    }

    fn dex(&mut self) {
        panic!("Unimplemented opcode 'DEX'");
    }

    fn dey(&mut self) {
        panic!("Unimplemented opcode 'DEY'");
    }

    fn eor(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'EOR'");
    }

    fn inc(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'INC'");
    }

    fn inx(&mut self) {
        panic!("Unimplemented opcode 'INX'");
    }

    fn iny(&mut self) {
        panic!("Unimplemented opcode 'INY'");
    }

    fn jmp(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'JMP'");
    }

    fn jsr(&mut self) {
        panic!("Unimplemented opcode 'JSR'");
    }

    fn lda(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'LDA'");
    }

    fn ldx(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'LDX'");
    }

    fn ldy(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'LDY'");
    }

    fn lsr(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'LSR'");
    }

    fn ora(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
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

    fn rol(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'ROL'");
    }

    fn ror(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'ROR'");
    }

    fn rti(&mut self) {
        panic!("Unimplemented opcode 'RTI'");
    }

    fn rts(&mut self) {
        panic!("Unimplemented opcode 'RTS'");
    }

    fn sbc(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
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

    fn sta(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'STA'");
    }

    fn stx(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
        panic!("Unimplemented opcode 'STX'");
    }

    fn sty(&mut self, bus: &mut dyn Bus16, addressing_mode: AddressingMode) {
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
