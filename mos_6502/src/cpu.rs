use crate::memory::Bus16;

#[derive(Copy, Clone, Debug)]
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

    total_cycles: u64,
    initialized: bool,
}

impl CPU {
    pub const RESET_VECTOR: u16 = 0xFFFA;

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
            total_cycles: 0,
            initialized: false,
        }
    }

    pub fn reset(&mut self, bus: &mut dyn Bus16) {
        self.pc = bus.read_word(Self::RESET_VECTOR);
        self.irq_disable = true;
        self.total_cycles += 6;
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn execute_instruction(&mut self, bus: &mut dyn Bus16) -> u64 {
        if !self.initialized {
            self.reset(bus);
            self.initialized = true;
        }

        let cycles_at_start = self.total_cycles;

        let opcode = bus.read_byte(self.pc);
        match opcode {
            // ADC
            0x69 => self.adc(bus, AddressingMode::Immediate, 2, 2),
            0x6D => self.adc(bus, AddressingMode::Absolute, 3, 4),
            0x65 => self.adc(bus, AddressingMode::ZeroPage, 3, 3),
            0x61 => self.adc(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0x71 => self.adc(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0x75 => self.adc(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0x7D => self.adc(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0x79 => self.adc(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // AND
            0x29 => self.and(bus, AddressingMode::Immediate, 2, 2),
            0x2D => self.and(bus, AddressingMode::Absolute, 3, 4),
            0x25 => self.and(bus, AddressingMode::ZeroPage, 3, 3),
            0x21 => self.and(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0x31 => self.and(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0x35 => self.and(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0x3D => self.and(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0x39 => self.and(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // ASL
            0x0E => self.asl(bus, AddressingMode::Absolute, 3, 4),
            0x06 => self.asl(bus, AddressingMode::ZeroPage, 3, 3),
            0x0A => self.asl(bus, AddressingMode::Accumulator, 1, 2),
            0x16 => self.asl(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0x1E => self.asl(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // BCC
            0x90 => self.bcc(bus, 2, 2),
            // BCS
            0xB0 => self.bcs(bus, 2, 2),
            // BEQ
            0xF0 => self.beq(bus, 2, 2),
            // BIT
            0x2C => self.bit(bus, AddressingMode::Absolute, 3, 4),
            0x24 => self.bit(bus, AddressingMode::ZeroPage, 3, 3),
            // BMI
            0x30 => self.bmi(bus, 2, 2),
            // BNE
            0xD0 => self.bne(bus, 2, 2),
            // BPL
            0x10 => self.bpl(bus, 2, 2),
            // BRK
            0x00 => self.brk(bus, AddressingMode::Implied, 1, 7),
            // BVC
            0x50 => self.bvc(bus, 2, 2),
            // BVS
            0x70 => self.bvs(bus, 2, 2),
            // CLC
            0x18 => self.clc(1, 2),
            // CLD
            0xD8 => self.cld(1, 2),
            // CLI
            0x58 => self.cli(1, 2),
            // CLV
            0xB8 => self.clv(1, 2),
            //CMP
            0xC9 => self.cmp(bus, AddressingMode::Immediate, 2, 2),
            0xCD => self.cmp(bus, AddressingMode::Absolute, 3, 4),
            0xC5 => self.cmp(bus, AddressingMode::ZeroPage, 3, 3),
            0xC1 => self.cmp(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0xD1 => self.cmp(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0xD5 => self.cmp(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0xDD => self.cmp(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0xD9 => self.cmp(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // CPX
            0xE0 => self.cpx(bus, AddressingMode::Immediate, 2, 2),
            0xEC => self.cpx(bus, AddressingMode::Absolute, 3, 4),
            0xE4 => self.cpx(bus, AddressingMode::ZeroPage, 3, 3),
            // CPY
            0xC0 => self.cpy(bus, AddressingMode::Immediate, 2, 2),
            0xCC => self.cpy(bus, AddressingMode::Absolute, 3, 4),
            0xC4 => self.cpy(bus, AddressingMode::ZeroPage, 3, 3),
            // DEC
            0xCE => self.dec(bus, AddressingMode::Absolute, 3, 6),
            0xC6 => self.dec(bus, AddressingMode::ZeroPage, 3, 5),
            0xD6 => self.dec(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0xDE => self.dec(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // DEX
            0xCA => self.dex(bus, AddressingMode::Implied, 1, 2),
            // DEY
            0x88 => self.dey(bus, AddressingMode::Implied, 1, 2),
            // EOR
            0x49 => self.eor(bus, AddressingMode::Immediate, 2, 2),
            0x4D => self.eor(bus, AddressingMode::Absolute, 3, 4),
            0x45 => self.eor(bus, AddressingMode::ZeroPage, 3, 3),
            0x41 => self.eor(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0x51 => self.eor(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0x55 => self.eor(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0x5D => self.eor(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0x59 => self.eor(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // INC
            0xEE => self.inc(bus, AddressingMode::Absolute, 3, 6),
            0xE6 => self.inc(bus, AddressingMode::ZeroPage, 3, 5),
            0xF6 => self.inc(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0xFE => self.inc(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // INX
            0xE8 => self.inx(bus, AddressingMode::Implied, 1, 2),
            // INY
            0xC8 => self.iny(bus, AddressingMode::Implied, 1, 2),
            // JMP
            0x4C => self.jmp(bus, AddressingMode::Absolute, 3),
            0x6C => self.jmp(bus, AddressingMode::AbsoluteIndirect, 5),
            // JSR
            0x20 => self.jsr(bus, AddressingMode::Absolute, 3, 6),
            // LDA
            0xA9 => self.lda(bus, AddressingMode::Immediate, 2, 2),
            0xAD => self.lda(bus, AddressingMode::Absolute, 3, 4),
            0xA5 => self.lda(bus, AddressingMode::ZeroPage, 3, 3),
            0xA1 => self.lda(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0xB1 => self.lda(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0xB5 => self.lda(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0xBD => self.lda(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0xB9 => self.lda(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // LDX
            0xA2 => self.ldx(bus, AddressingMode::Immediate, 2, 2),
            0xAE => self.ldx(bus, AddressingMode::Absolute, 3, 4),
            0xA6 => self.ldx(bus, AddressingMode::ZeroPage, 3, 3),
            0xBE => self.ldx(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            0xB6 => self.ldx(bus, AddressingMode::IndexedZeroPageY, 2, 4),
            // LDY
            0xA0 => self.ldy(bus, AddressingMode::Immediate, 2, 2),
            0xAC => self.ldy(bus, AddressingMode::Absolute, 3, 4),
            0xA4 => self.ldy(bus, AddressingMode::ZeroPage, 3, 3),
            0xB4 => self.ldy(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0xBC => self.ldy(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            // LSR
            0x4E => self.lsr(bus, AddressingMode::Absolute, 3, 6),
            0x46 => self.lsr(bus, AddressingMode::ZeroPage, 3, 5),
            0x4A => self.lsr(bus, AddressingMode::Accumulator, 1, 2),
            0x56 => self.lsr(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0x5E => self.lsr(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // NOP
            0xEA => self.nop(bus, AddressingMode::Implied, 1, 2),
            // ORA
            0x09 => self.ora(bus, AddressingMode::Immediate, 2, 2),
            0x0D => self.ora(bus, AddressingMode::Absolute, 3, 4),
            0x05 => self.ora(bus, AddressingMode::ZeroPage, 3, 3),
            0x01 => self.ora(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0x11 => self.ora(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0x15 => self.ora(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0x1D => self.ora(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0x19 => self.ora(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // PHA
            0x48 => self.pha(bus, AddressingMode::Implied, 1, 3),
            // PHP
            0x08 => self.php(bus, AddressingMode::Implied, 1, 3),
            // PLA
            0x68 => self.pla(bus, AddressingMode::Implied, 1, 4),
            // PLP
            0x28 => self.plp(bus, AddressingMode::Implied, 1, 4),
            // ROL
            0x2E => self.rol(bus, AddressingMode::Absolute, 3, 6),
            0x26 => self.rol(bus, AddressingMode::ZeroPage, 3, 5),
            0x2A => self.rol(bus, AddressingMode::Accumulator, 1, 2),
            0x36 => self.rol(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0x3E => self.rol(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // ROR
            0x6E => self.ror(bus, AddressingMode::Absolute, 3, 6),
            0x66 => self.ror(bus, AddressingMode::ZeroPage, 3, 5),
            0x6A => self.ror(bus, AddressingMode::Accumulator, 1, 2),
            0x76 => self.ror(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0x7E => self.ror(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // RTI
            0x40 => self.rti(bus, AddressingMode::Implied, 1, 6),
            // RTS
            0x60 => self.rts(bus, AddressingMode::Implied, 1, 6),
            // SBC
            0xE9 => self.sbc(bus, AddressingMode::Immediate, 2, 2),
            0xED => self.sbc(bus, AddressingMode::Absolute, 3, 4),
            0xE5 => self.sbc(bus, AddressingMode::ZeroPage, 3, 3),
            0xE1 => self.sbc(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0xF1 => self.sbc(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0xF5 => self.sbc(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0xFD => self.sbc(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0xF9 => self.sbc(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // SEC
            0x38 => self.sec(1, 2),
            // SED
            0xF8 => self.sed(1, 2),
            // SEI
            0x78 => self.sei(1, 2),
            // STA
            0x8D => self.sta(bus, AddressingMode::Absolute, 3, 4),
            0x85 => self.sta(bus, AddressingMode::ZeroPage, 3, 3),
            0x81 => self.sta(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0x91 => self.sta(bus, AddressingMode::IndexedIndirectY, 2, 6),
            0x95 => self.sta(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0x9D => self.sta(bus, AddressingMode::IndexedAbsoluteX, 3, 5),
            0x99 => self.sta(bus, AddressingMode::IndexedAbsoluteY, 3, 5),
            // STX
            0x8E => self.stx(bus, AddressingMode::Absolute, 3, 4),
            0x86 => self.stx(bus, AddressingMode::ZeroPage, 3, 3),
            0x96 => self.stx(bus, AddressingMode::IndexedZeroPageY, 2, 4),
            // STY
            0x8C => self.sty(bus, AddressingMode::Absolute, 3, 4),
            0x84 => self.sty(bus, AddressingMode::ZeroPage, 3, 3),
            0x94 => self.sty(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            // TAX
            0xAA => self.tax(1, 2),
            // TAY
            0xA8 => self.tay(1, 2),
            // TSX
            0xBA => self.tsx(1, 2),
            // TXA
            0x8A => self.txa(1, 2),
            // TXS
            0x9A => self.txs(1, 2),
            // TYA
            0x98 => self.tya(1, 2),
            _ => panic!("Unknown opcode: {}", opcode),
        };

        self.total_cycles - cycles_at_start
    }

    fn resolve_address(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode) -> u16 {
        match addr_mode {
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
            AddressingMode::Accumulator => {
                panic!("Attempt to resolve address of accumulator register!")
            }
            AddressingMode::Implied => {
                panic!("Attempt to resolve address in implied addressing mode!")
            }
            AddressingMode::Relative => {
                panic!("Attempt to resolve address in relative addressing mode!")
            }
        }
    }

    fn resolve_relative_offset(&self, bus: &mut dyn Bus16) -> i16 {
        let offset: i8 = unsafe { std::mem::transmute(bus.read_byte(self.pc + 1)) };
        offset as i16
    }

    fn adc(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        if self.decimal_mode {
            panic!("ADC: decimal mode not yet implemented!");
        }

        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);

        let rhs = self.a;
        let lhs = value;
        let carry_in: u8 = unsafe { std::mem::transmute(self.carry) };

        let wide_sum = (self.a as u16) + (value as u16) + (carry_in as u16);
        let sum = (wide_sum & 0xFF) as u8;

        self.carry = wide_sum > 0xFF;
        self.overflow = ((sum ^ rhs) & (sum ^ lhs) & (1 << 7)) != 0;
        self.set_nz_flags(sum);

        self.a = sum;

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn and(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'AND'");
    }

    fn asl(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'AND'");
    }

    fn bcc(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'BCC'");
    }

    fn bcs(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'BCS'");
    }

    fn beq(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        if self.zero {
            let offset = self.resolve_relative_offset(bus);
            self.pc = self.pc.wrapping_add_signed(offset);
        }

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn bit(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'BIT'");
    }

    fn bmi(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'BMI'");
    }

    fn bne(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        if !self.zero {
            let offset = self.resolve_relative_offset(bus);
            self.pc = self.pc.wrapping_add_signed(offset);
        }

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn bpl(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'BPL'");
    }

    fn brk(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'BRK'");
    }

    fn bvc(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'BVC'");
    }

    fn bvs(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'BVS'");
    }

    fn clc(&mut self, length: u16, cycles: u64) {
        self.carry = false;

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn cld(&mut self, length: u16, cycles: u64) {
        self.decimal_mode = false;

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn cli(&mut self, length: u16, cycles: u64) {
        self.irq_disable = false;

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn clv(&mut self, length: u16, cycles: u64) {
        self.overflow = false;

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn cmp(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'CMP'");
    }

    fn cpx(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'CPX'");
    }

    fn cpy(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'CPY'");
    }

    fn dec(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'DEC'");
    }

    fn dex(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'DEX'");
    }

    fn dey(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'DEY'");
    }

    fn eor(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'EOR'");
    }

    fn inc(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'INC'");
    }

    fn inx(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'INX'");
    }

    fn iny(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'INY'");
    }

    fn jmp(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);

        self.pc = address;
        self.total_cycles += cycles;
    }

    fn jsr(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'JSR'");
    }

    fn lda(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        self.a = bus.read_byte(address);
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn ldx(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        self.x = bus.read_byte(address);
        self.set_nz_flags(self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn ldy(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'LDY'");
    }

    fn lsr(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'LSR'");
    }

    fn nop(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'NOP'");
    }

    fn ora(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'ORA'");
    }

    fn pha(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'PHA'");
    }

    fn php(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'PHP'");
    }

    fn pla(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'PLA'");
    }

    fn plp(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'PLP'");
    }

    fn rol(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'ROL'");
    }

    fn ror(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'ROR'");
    }

    fn rti(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'RTI'");
    }

    fn rts(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'RTS'");
    }

    fn sbc(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'SBC'");
    }

    fn sec(&mut self, length: u16, cycles: u64) {
        self.carry = true;

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn sed(&mut self, length: u16, cycles: u64) {
        self.decimal_mode = true;

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn sei(&mut self, length: u16, cycles: u64) {
        self.irq_disable = true;

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn sta(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        bus.write_byte(address, self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn stx(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'STX'");
    }

    fn sty(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        panic!("Unimplemented opcode 'STY'");
    }

    fn tax(&mut self, length: u16, cycles: u64) {
        self.x = self.a;
        self.set_nz_flags(self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn tay(&mut self, length: u16, cycles: u64) {
        self.y = self.a;
        self.set_nz_flags(self.y);

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn tsx(&mut self, length: u16, cycles: u64) {
        self.x = self.s;
        self.set_nz_flags(self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn txa(&mut self, length: u16, cycles: u64) {
        self.a = self.x;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn txs(&mut self, length: u16, cycles: u64) {
        self.s = self.x;

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn tya(&mut self, length: u16, cycles: u64) {
        self.a = self.y;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn set_nz_flags(&mut self, value: u8) {
        self.zero = value == 0;
        self.negative = value & (1 << 7) != 0;
    }
}
