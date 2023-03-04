use crate::bus::Bus16;

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
            0x69 => self.adc(opcode),
            0x6D => self.adc(opcode),
            0x65 => self.adc(opcode),
            0x61 => self.adc(opcode),
            0x71 => self.adc(opcode),
            0x75 => self.adc(opcode),
            0x7D => self.adc(opcode),
            0x79 => self.adc(opcode),
            // AND
            0x29 => self.and(opcode),
            0x2D => self.and(opcode),
            0x25 => self.and(opcode),
            0x21 => self.and(opcode),
            0x31 => self.and(opcode),
            0x35 => self.and(opcode),
            0x3D => self.and(opcode),
            0x39 => self.and(opcode),
            // ASL
            0x0E => self.asl(opcode),
            0x06 => self.asl(opcode),
            0x0A => self.asl(opcode),
            0x16 => self.asl(opcode),
            0x1E => self.asl(opcode),
            // BCC
            0x90 => self.bcc(),
            // BCS
            0xB0 => self.bcs(),
            // BEQ
            0xF0 => self.beq(),
            // BIT
            0x2C => self.bit(opcode),
            0x24 => self.bit(opcode),
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
            0xC9 => self.cmp(opcode),
            0xCD => self.cmp(opcode),
            0xC5 => self.cmp(opcode),
            0xC1 => self.cmp(opcode),
            0xD1 => self.cmp(opcode),
            0xD5 => self.cmp(opcode),
            0xDD => self.cmp(opcode),
            0xD9 => self.cmp(opcode),
            // CPX
            0xE0 => self.cpx(opcode),
            0xEC => self.cpx(opcode),
            0xE4 => self.cpx(opcode),
            // CPY
            0xC0 => self.cpy(opcode),
            0xCC => self.cpy(opcode),
            0xC4 => self.cpy(opcode),
            // DEC
            0xCE => self.dec(opcode),
            0xC6 => self.dec(opcode),
            0xD6 => self.dec(opcode),
            0xDE => self.dec(opcode),
            // DEX
            0xCA => self.dex(),
            // DEY
            0x88 => self.dey(),
            // EOR
            0x49 => self.eor(opcode),
            0x4D => self.eor(opcode),
            0x45 => self.eor(opcode),
            0x41 => self.eor(opcode),
            0x51 => self.eor(opcode),
            0x55 => self.eor(opcode),
            0x5D => self.eor(opcode),
            0x59 => self.eor(opcode),
            // INC
            0xEE => self.inc(opcode),
            0xE6 => self.inc(opcode),
            0xF6 => self.inc(opcode),
            0xFE => self.inc(opcode),
            // INX
            0xE8 => self.inx(),
            // INY
            0xC8 => self.iny(),
            // JMP
            0x4C => self.jmp(opcode),
            0x6C => self.jmp(opcode),
            // JSR
            0x20 => self.jsr(),
            // LDA
            0xA9 => self.lda(opcode),
            0xAD => self.lda(opcode),
            0xA5 => self.lda(opcode),
            0xA1 => self.lda(opcode),
            0xB1 => self.lda(opcode),
            0xB5 => self.lda(opcode),
            0xBD => self.lda(opcode),
            0xB9 => self.lda(opcode),
            // LDX
            0xA2 => self.ldx(opcode),
            0xAE => self.ldx(opcode),
            0xA6 => self.ldx(opcode),
            0xBE => self.ldx(opcode),
            0xB6 => self.ldx(opcode),
            // LDY
            0xA0 => self.ldy(opcode),
            0xAC => self.ldy(opcode),
            0xA4 => self.ldy(opcode),
            0xB4 => self.ldy(opcode),
            0xBC => self.ldy(opcode),
            // LSR
            0x4E => self.lsr(opcode),
            0x46 => self.lsr(opcode),
            0x4A => self.lsr(opcode),
            0x56 => self.lsr(opcode),
            0x5E => self.lsr(opcode),
            // NOP
            0xEA => (),
            // ORA
            0x09 => self.ora(opcode),
            0x0D => self.ora(opcode),
            0x05 => self.ora(opcode),
            0x01 => self.ora(opcode),
            0x11 => self.ora(opcode),
            0x15 => self.ora(opcode),
            0x1D => self.ora(opcode),
            0x19 => self.ora(opcode),
            // PHA
            0x48 => self.pha(),
            // PHP
            0x08 => self.php(),
            // PLA
            0x68 => self.pla(),
            // PLP
            0x28 => self.plp(),
            // ROL
            0x2E => self.rol(opcode),
            0x26 => self.rol(opcode),
            0x2A => self.rol(opcode),
            0x36 => self.rol(opcode),
            0x3E => self.rol(opcode),
            // ROR
            0x6E => self.ror(opcode),
            0x66 => self.ror(opcode),
            0x6A => self.ror(opcode),
            0x76 => self.ror(opcode),
            0x7E => self.ror(opcode),
            // RTI
            0x40 => self.rti(),
            // RTS
            0x60 => self.rts(),
            // SBC
            0xE9 => self.sbc(opcode),
            0xED => self.sbc(opcode),
            0xE5 => self.sbc(opcode),
            0xE1 => self.sbc(opcode),
            0xF1 => self.sbc(opcode),
            0xF5 => self.sbc(opcode),
            0xFD => self.sbc(opcode),
            0xF9 => self.sbc(opcode),
            // SEC
            0x38 => self.sec(),
            // SED
            0xF8 => self.sed(),
            // SEI
            0x78 => self.sei(),
            // STA
            0x8D => self.sta(opcode),
            0x85 => self.sta(opcode),
            0x81 => self.sta(opcode),
            0x91 => self.sta(opcode),
            0x95 => self.sta(opcode),
            0x9D => self.sta(opcode),
            0x99 => self.sta(opcode),
            // STX
            0x8E => self.stx(opcode),
            0x86 => self.stx(opcode),
            0x96 => self.stx(opcode),
            // STY
            0x8C => self.sty(opcode),
            0x84 => self.sty(opcode),
            0x94 => self.sty(opcode),
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

    fn adc(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'ADC'");
    }

    fn and(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'AND'");
    }

    fn asl(&mut self, opcode: u8) {
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

    fn bit(&mut self, opcode: u8) {
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

    fn cmp(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'CMP'");
    }

    fn cpx(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'CPX'");
    }

    fn cpy(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'CPY'");
    }

    fn dec(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'DEC'");
    }

    fn dex(&mut self) {
        panic!("Unimplemented opcode 'DEX'");
    }

    fn dey(&mut self) {
        panic!("Unimplemented opcode 'DEY'");
    }

    fn eor(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'EOR'");
    }

    fn inc(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'INC'");
    }

    fn inx(&mut self) {
        panic!("Unimplemented opcode 'INX'");
    }

    fn iny(&mut self) {
        panic!("Unimplemented opcode 'INY'");
    }

    fn jmp(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'JMP'");
    }

    fn jsr(&mut self) {
        panic!("Unimplemented opcode 'JSR'");
    }

    fn lda(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'LDA'");
    }

    fn ldx(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'LDX'");
    }

    fn ldy(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'LDY'");
    }

    fn lsr(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'LSR'");
    }

    fn ora(&mut self, opcode: u8) {
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

    fn rol(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'ROL'");
    }

    fn ror(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'ROR'");
    }

    fn rti(&mut self) {
        panic!("Unimplemented opcode 'RTI'");
    }

    fn rts(&mut self) {
        panic!("Unimplemented opcode 'RTS'");
    }

    fn sbc(&mut self, opcode: u8) {
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

    fn sta(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'STA'");
    }

    fn stx(&mut self, opcode: u8) {
        panic!("Unimplemented opcode 'STX'");
    }

    fn sty(&mut self, opcode: u8) {
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
