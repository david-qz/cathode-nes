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
}

/// A MOS 6502 CPU
pub struct CPU {
    pub a: u8,
    pub x: u8,
    pub y: u8,

    pub pc: u16,
    pub s: u8,

    pub carry: bool,
    pub zero: bool,
    pub irq_disable: bool,
    pub decimal_mode: bool,
    pub overflow: bool,
    pub negative: bool,

    pub total_cycles: u64,
    initialized: bool,
}

impl CPU {
    pub const RESET_VECTOR: u16 = 0xFFFC;
    pub const IRQ_VECTOR: u16 = 0xFFFE;
    pub const NMI_VECTOR: u16 = 0xFFFA;

    const STACK_BASE: u16 = 0x0100;

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
            0x65 => self.adc(bus, AddressingMode::ZeroPage, 2, 3),
            0x61 => self.adc(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0x71 => self.adc(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0x75 => self.adc(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0x7D => self.adc(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0x79 => self.adc(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // AND
            0x29 => self.and(bus, AddressingMode::Immediate, 2, 2),
            0x2D => self.and(bus, AddressingMode::Absolute, 3, 4),
            0x25 => self.and(bus, AddressingMode::ZeroPage, 2, 3),
            0x21 => self.and(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0x31 => self.and(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0x35 => self.and(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0x3D => self.and(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0x39 => self.and(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // ASL
            0x0E => self.asl(bus, AddressingMode::Absolute, 3, 4),
            0x06 => self.asl(bus, AddressingMode::ZeroPage, 2, 3),
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
            0x24 => self.bit(bus, AddressingMode::ZeroPage, 2, 3),
            // BMI
            0x30 => self.bmi(bus, 2, 2),
            // BNE
            0xD0 => self.bne(bus, 2, 2),
            // BPL
            0x10 => self.bpl(bus, 2, 2),
            // BRK
            0x00 => self.brk(bus, 7),
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
            0xC5 => self.cmp(bus, AddressingMode::ZeroPage, 2, 3),
            0xC1 => self.cmp(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0xD1 => self.cmp(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0xD5 => self.cmp(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0xDD => self.cmp(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0xD9 => self.cmp(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // CPX
            0xE0 => self.cpx(bus, AddressingMode::Immediate, 2, 2),
            0xEC => self.cpx(bus, AddressingMode::Absolute, 3, 4),
            0xE4 => self.cpx(bus, AddressingMode::ZeroPage, 2, 3),
            // CPY
            0xC0 => self.cpy(bus, AddressingMode::Immediate, 2, 2),
            0xCC => self.cpy(bus, AddressingMode::Absolute, 3, 4),
            0xC4 => self.cpy(bus, AddressingMode::ZeroPage, 2, 3),
            // DEC
            0xCE => self.dec(bus, AddressingMode::Absolute, 3, 6),
            0xC6 => self.dec(bus, AddressingMode::ZeroPage, 2, 5),
            0xD6 => self.dec(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0xDE => self.dec(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // DEX
            0xCA => self.dex(1, 2),
            // DEY
            0x88 => self.dey(1, 2),
            // EOR
            0x49 => self.eor(bus, AddressingMode::Immediate, 2, 2),
            0x4D => self.eor(bus, AddressingMode::Absolute, 3, 4),
            0x45 => self.eor(bus, AddressingMode::ZeroPage, 2, 3),
            0x41 => self.eor(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0x51 => self.eor(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0x55 => self.eor(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0x5D => self.eor(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0x59 => self.eor(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // INC
            0xEE => self.inc(bus, AddressingMode::Absolute, 3, 6),
            0xE6 => self.inc(bus, AddressingMode::ZeroPage, 2, 5),
            0xF6 => self.inc(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0xFE => self.inc(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // INX
            0xE8 => self.inx(1, 2),
            // INY
            0xC8 => self.iny(1, 2),
            // JMP
            0x4C => self.jmp(bus, AddressingMode::Absolute, 3),
            0x6C => self.jmp(bus, AddressingMode::AbsoluteIndirect, 5),
            // JSR
            0x20 => self.jsr(bus, AddressingMode::Absolute, 3, 6),
            // LDA
            0xA9 => self.lda(bus, AddressingMode::Immediate, 2, 2),
            0xAD => self.lda(bus, AddressingMode::Absolute, 3, 4),
            0xA5 => self.lda(bus, AddressingMode::ZeroPage, 2, 3),
            0xA1 => self.lda(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0xB1 => self.lda(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0xB5 => self.lda(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0xBD => self.lda(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0xB9 => self.lda(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // LDX
            0xA2 => self.ldx(bus, AddressingMode::Immediate, 2, 2),
            0xAE => self.ldx(bus, AddressingMode::Absolute, 3, 4),
            0xA6 => self.ldx(bus, AddressingMode::ZeroPage, 2, 3),
            0xBE => self.ldx(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            0xB6 => self.ldx(bus, AddressingMode::IndexedZeroPageY, 2, 4),
            // LDY
            0xA0 => self.ldy(bus, AddressingMode::Immediate, 2, 2),
            0xAC => self.ldy(bus, AddressingMode::Absolute, 3, 4),
            0xA4 => self.ldy(bus, AddressingMode::ZeroPage, 2, 3),
            0xB4 => self.ldy(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0xBC => self.ldy(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            // LSR
            0x4E => self.lsr(bus, AddressingMode::Absolute, 3, 6),
            0x46 => self.lsr(bus, AddressingMode::ZeroPage, 2, 5),
            0x4A => self.lsr(bus, AddressingMode::Accumulator, 1, 2),
            0x56 => self.lsr(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0x5E => self.lsr(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // NOP
            0xEA => self.nop(1, 2),
            // ORA
            0x09 => self.ora(bus, AddressingMode::Immediate, 2, 2),
            0x0D => self.ora(bus, AddressingMode::Absolute, 3, 4),
            0x05 => self.ora(bus, AddressingMode::ZeroPage, 2, 3),
            0x01 => self.ora(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0x11 => self.ora(bus, AddressingMode::IndexedIndirectY, 2, 5),
            0x15 => self.ora(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0x1D => self.ora(bus, AddressingMode::IndexedAbsoluteX, 3, 4),
            0x19 => self.ora(bus, AddressingMode::IndexedAbsoluteY, 3, 4),
            // PHA
            0x48 => self.pha(bus, 1, 3),
            // PHP
            0x08 => self.php(bus, 1, 3),
            // PLA
            0x68 => self.pla(bus, 1, 4),
            // PLP
            0x28 => self.plp(bus, 1, 4),
            // ROL
            0x2E => self.rol(bus, AddressingMode::Absolute, 3, 6),
            0x26 => self.rol(bus, AddressingMode::ZeroPage, 2, 5),
            0x2A => self.rol(bus, AddressingMode::Accumulator, 1, 2),
            0x36 => self.rol(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0x3E => self.rol(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // ROR
            0x6E => self.ror(bus, AddressingMode::Absolute, 3, 6),
            0x66 => self.ror(bus, AddressingMode::ZeroPage, 2, 5),
            0x6A => self.ror(bus, AddressingMode::Accumulator, 1, 2),
            0x76 => self.ror(bus, AddressingMode::IndexedZeroPageX, 2, 6),
            0x7E => self.ror(bus, AddressingMode::IndexedAbsoluteX, 3, 7),
            // RTI
            0x40 => self.rti(bus, 6),
            // RTS
            0x60 => self.rts(bus, 6),
            // SBC
            0xE9 => self.sbc(bus, AddressingMode::Immediate, 2, 2),
            0xED => self.sbc(bus, AddressingMode::Absolute, 3, 4),
            0xE5 => self.sbc(bus, AddressingMode::ZeroPage, 2, 3),
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
            0x85 => self.sta(bus, AddressingMode::ZeroPage, 2, 3),
            0x81 => self.sta(bus, AddressingMode::IndexedIndirectX, 2, 6),
            0x91 => self.sta(bus, AddressingMode::IndexedIndirectY, 2, 6),
            0x95 => self.sta(bus, AddressingMode::IndexedZeroPageX, 2, 4),
            0x9D => self.sta(bus, AddressingMode::IndexedAbsoluteX, 3, 5),
            0x99 => self.sta(bus, AddressingMode::IndexedAbsoluteY, 3, 5),
            // STX
            0x8E => self.stx(bus, AddressingMode::Absolute, 3, 4),
            0x86 => self.stx(bus, AddressingMode::ZeroPage, 2, 3),
            0x96 => self.stx(bus, AddressingMode::IndexedZeroPageY, 2, 4),
            // STY
            0x8C => self.sty(bus, AddressingMode::Absolute, 3, 4),
            0x84 => self.sty(bus, AddressingMode::ZeroPage, 2, 3),
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
            _ => panic!("Unknown opcode: 0x{:X}", opcode),
        };

        self.total_cycles - cycles_at_start
    }

    #[inline(always)]
    fn crosses_page_boundary(a: u16, b: u16) -> bool {
        a & 0xFF00 != b & 0xFF00
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
                let effective_address = base_address.wrapping_add(self.x as u16);
                if CPU::crosses_page_boundary(base_address, effective_address) {
                    self.total_cycles += 1;
                }
                effective_address
            }
            AddressingMode::IndexedAbsoluteY => {
                let base_address = bus.read_word(self.pc + 1);
                let effective_address = base_address.wrapping_add(self.y as u16);
                if CPU::crosses_page_boundary(base_address, effective_address) {
                    self.total_cycles += 1;
                }
                effective_address
            }
            AddressingMode::IndexedIndirectX => {
                let indirect_base_address_zero_page = bus.read_byte(self.pc + 1);
                let indirect_address = indirect_base_address_zero_page.wrapping_add(self.x) as u16;
                bus.read_word(indirect_address)
            }
            AddressingMode::IndexedIndirectY => {
                let indirect_address_zero_page = bus.read_byte(self.pc + 1);
                let base_address = bus.read_word(indirect_address_zero_page as u16);
                let effective_address = base_address.wrapping_add(self.y as u16);
                if CPU::crosses_page_boundary(base_address, effective_address) {
                    self.total_cycles += 1;
                }
                effective_address
            }
            AddressingMode::AbsoluteIndirect => {
                let indirect_address = bus.read_word(self.pc + 1);
                bus.read_word(indirect_address)
            }
            AddressingMode::Accumulator => {
                panic!("Attempt to resolve address of accumulator register!")
            }
        }
    }

    fn resolve_relative_offset(&self, bus: &mut dyn Bus16) -> i16 {
        (bus.read_byte(self.pc + 1) as i8) as i16
    }

    fn push_byte(&mut self, bus: &mut dyn Bus16, value: u8) {
        bus.write_byte(Self::STACK_BASE + self.s as u16, value);
        self.s = self.s.wrapping_sub(1);
    }

    fn push_word(&mut self, bus: &mut dyn Bus16, value: u16) {
        self.push_byte(bus, ((value & 0xFF00) >> 8) as u8);
        self.push_byte(bus, ((value & 0x00FF) >> 0) as u8);
    }

    fn pull_byte(&mut self, bus: &mut dyn Bus16) -> u8 {
        self.s = self.s.wrapping_add(1);
        bus.read_byte(Self::STACK_BASE + self.s as u16)
    }

    fn pull_word(&mut self, bus: &mut dyn Bus16) -> u16 {
        let l_byte = self.pull_byte(bus);
        let h_byte = self.pull_byte(bus);
        (h_byte as u16) << 8 | (l_byte as u16)
    }

    #[rustfmt::skip]
    fn encode_p(&self, brk_command: bool) -> u8 {
        0 | (self.negative as u8)     << 7
          | (self.overflow as u8)     << 6
          | 1                         << 5
          | (brk_command as u8)       << 4
          | (self.decimal_mode as u8) << 3
          | (self.irq_disable as u8)  << 2
          | (self.zero as u8)         << 1
          | (self.carry as u8)        << 0
    }

    #[rustfmt::skip]
    fn decode_p(&mut self, p: u8) {
        self.negative     = p & (1 << 7) != 0;
        self.overflow     = p & (1 << 6) != 0;
        self.decimal_mode = p & (1 << 3) != 0;
        self.irq_disable  = p & (1 << 2) != 0;
        self.zero         = p & (1 << 1) != 0;
        self.carry        = p & (1 << 0) != 0;
    }

    fn set_nz_flags(&mut self, value: u8) {
        self.zero = value == 0;
        self.negative = value & (1 << 7) != 0;
    }

    // Operation PHA: Push accumulator on stack.
    fn pha(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.push_byte(bus, self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation PHP: Push processor status on stack.
    fn php(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        let p = self.encode_p(true);
        self.push_byte(bus, p);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation PLA: Pull accumulator from stack.
    fn pla(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.a = self.pull_byte(bus);
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation PLP: Pull processor status from stack.
    fn plp(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        let p = self.pull_byte(bus);
        self.decode_p(p);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation JSR: Jump to subroutine.
    fn jsr(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let jmp_address = self.resolve_address(bus, addr_mode);
        self.push_word(bus, self.pc + length - 1);

        self.pc = jmp_address;
        self.total_cycles += cycles;
    }

    // Operation RTS: Return from subroutine.
    fn rts(&mut self, bus: &mut dyn Bus16, cycles: u64) {
        let jmp_address = self.pull_word(bus);

        self.pc = jmp_address + 1;
        self.total_cycles += cycles;
    }

    // Operation BRK: Force break.
    fn brk(&mut self, bus: &mut dyn Bus16, cycles: u64) {
        let return_address = self.pc + 2;
        self.push_word(bus, return_address);
        let p = self.encode_p(true);
        self.push_byte(bus, p);
        self.irq_disable = true;

        self.pc = bus.read_word(Self::IRQ_VECTOR);
        self.total_cycles += cycles;
    }

    // Operation RTI: Return from interrupt.
    fn rti(&mut self, bus: &mut dyn Bus16, cycles: u64) {
        let p = self.pull_byte(bus);
        self.decode_p(p);
        let return_address = self.pull_word(bus);

        self.pc = return_address;
        self.total_cycles += cycles;
    }

    fn adder(&mut self, rhs: u8, lhs: u8, carry: bool) -> (u8, bool, bool) {
        let (sum, carry1) = rhs.overflowing_add(lhs);
        let (sum, carry2) = sum.overflowing_add(carry as u8);
        (
            sum,
            carry1 || carry2,
            ((sum ^ rhs) & (sum ^ lhs) & (1 << 7)) != 0,
        )
    }

    // Operation ADC: Add memory to accumulator with carry.
    fn adc(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        if self.decimal_mode {
            panic!("ADC: decimal mode not yet implemented!");
        }

        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);

        let (sum, carry, overflow) = self.adder(self.a, value, self.carry);
        self.a = sum;
        self.carry = carry;
        self.overflow = overflow;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation SBC: Subtract memory from accumulator with borrow.
    fn sbc(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        if self.decimal_mode {
            panic!("SBC: decimal mode not yet implemented!");
        }

        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);

        let (sum, carry, overflow) = self.adder(self.a, !value, self.carry);
        self.a = sum;
        self.carry = carry;
        self.overflow = overflow;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation AND: "AND" memory with accumulator.
    fn and(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);
        self.a = self.a & value;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation ORA: "OR" memory with accumulator.
    fn ora(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);
        self.a = self.a | value;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation EOR: "XOR" memory with accumulator.
    fn eor(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);
        self.a = self.a ^ value;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation BIT: Test bits in memory with accumulator.
    fn bit(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);
        self.zero = self.a & value == 0;
        self.negative = value & 0b10000000 != 0;
        self.overflow = value & 0b01000000 != 0;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation ASL: Shift left one bit (memory or accumulator).
    fn asl(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let value = match addr_mode {
            AddressingMode::Accumulator => self.a,
            _ => {
                let address = self.resolve_address(bus, addr_mode);
                bus.read_byte(address)
            }
        };

        let result = value << 1;
        self.set_nz_flags(result);
        self.carry = value & (1 << 7) != 0;

        match addr_mode {
            AddressingMode::Accumulator => {
                self.a = result;
            }
            _ => {
                let address = self.resolve_address(bus, addr_mode);
                bus.write_byte(address, result);
            }
        }

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation LSR: Shift right one bit (memory or accumulator).
    fn lsr(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let value = match addr_mode {
            AddressingMode::Accumulator => self.a,
            _ => {
                let address = self.resolve_address(bus, addr_mode);
                bus.read_byte(address)
            }
        };

        let result = value >> 1;
        self.set_nz_flags(result);
        self.carry = value & (1 << 0) != 0;

        match addr_mode {
            AddressingMode::Accumulator => {
                self.a = result;
            }
            _ => {
                let address = self.resolve_address(bus, addr_mode);
                bus.write_byte(address, result);
            }
        }

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation ROL: Rotate left one bit (memory or accumulator).
    fn rol(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let value = match addr_mode {
            AddressingMode::Accumulator => self.a,
            _ => {
                let address = self.resolve_address(bus, addr_mode);
                bus.read_byte(address)
            }
        };

        let result = value.rotate_left(1) & 0b11111110 | (self.carry as u8) << 0;
        self.carry = value & (1 << 7) != 0;
        self.set_nz_flags(result);

        match addr_mode {
            AddressingMode::Accumulator => {
                self.a = result;
            }
            _ => {
                let address = self.resolve_address(bus, addr_mode);
                bus.write_byte(address, result);
            }
        }

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation ROR: Rotate right one bit (memory or accumulator).
    fn ror(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let value = match addr_mode {
            AddressingMode::Accumulator => self.a,
            _ => {
                let address = self.resolve_address(bus, addr_mode);
                bus.read_byte(address)
            }
        };

        let result = value.rotate_right(1) & 0b01111111 | (self.carry as u8) << 7;
        self.carry = value & (1 << 0) != 0;
        self.set_nz_flags(result);

        match addr_mode {
            AddressingMode::Accumulator => {
                self.a = result;
            }
            _ => {
                let address = self.resolve_address(bus, addr_mode);
                bus.write_byte(address, result);
            }
        }

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation JMP: Jump to new location.
    fn jmp(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, cycles: u64) {
        let jmp_address = self.resolve_address(bus, addr_mode);

        self.pc = jmp_address;
        self.total_cycles += cycles;
    }

    fn relative_conditional_branch(&mut self, bus: &mut dyn Bus16, should_branch: bool) {
        if should_branch {
            let offset = self.resolve_relative_offset(bus);
            self.pc = self.pc.wrapping_add_signed(offset);
        }
    }

    // Operation BEQ: Branch on result zero.
    fn beq(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.relative_conditional_branch(bus, self.zero);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation BNE: Branch on result not zero.
    fn bne(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.relative_conditional_branch(bus, !self.zero);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation BCC: Branch on carry clear.
    fn bcc(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.relative_conditional_branch(bus, !self.carry);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation BCS: Branch on carry set.
    fn bcs(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.relative_conditional_branch(bus, self.carry);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation BVC: Branch on overflow clear.
    fn bvc(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.relative_conditional_branch(bus, !self.overflow);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation BVS: Branch on overflow set.
    fn bvs(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.relative_conditional_branch(bus, self.overflow);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation BMI: Branch on result minus.
    fn bmi(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.relative_conditional_branch(bus, self.negative);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation BPL: Branch on result plus.
    fn bpl(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.relative_conditional_branch(bus, !self.negative);

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn compare_value(&mut self, lhs: u8, rhs: u8) {
        use std::cmp::Ordering::*;
        match lhs.cmp(&rhs) {
            Less => {
                self.zero = false;
                self.carry = false;
                self.negative = lhs.wrapping_sub(rhs) & (1 << 7) != 0;
            }
            Greater => {
                self.zero = false;
                self.carry = true;
                self.negative = lhs.wrapping_sub(rhs) & (1 << 7) != 0;
            }
            Equal => {
                self.zero = true;
                self.carry = true;
                self.negative = false;
            }
        }
    }

    // Operation CMP: Compare memory and accumulator.
    fn cmp(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);

        self.compare_value(self.a, value);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation CPX: Compare memory and index X.
    fn cpx(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);

        self.compare_value(self.x, value);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation CPY: Compare memory and index Y.
    fn cpy(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);

        self.compare_value(self.y, value);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation INC: Increment memory by one.
    fn inc(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);
        let result = value.wrapping_add(1);
        self.set_nz_flags(result);
        bus.write_byte(address, result);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation DEC: Decrement memory by one.
    fn dec(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        let value = bus.read_byte(address);
        let result = value.wrapping_sub(1);
        self.set_nz_flags(result);
        bus.write_byte(address, result);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation INX: Increment index X by one.
    fn inx(&mut self, length: u16, cycles: u64) {
        self.x = self.x.wrapping_add(1);
        self.set_nz_flags(self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation INY: Increment index Y by one.
    fn iny(&mut self, length: u16, cycles: u64) {
        self.y = self.y.wrapping_add(1);
        self.set_nz_flags(self.y);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation DEX: Decrement index X by one.
    fn dex(&mut self, length: u16, cycles: u64) {
        self.x = self.x.wrapping_sub(1);
        self.set_nz_flags(self.x);

        self.pc += length;
        self.total_cycles += cycles
    }

    // Operation DEY: Decrement index Y by one.
    fn dey(&mut self, length: u16, cycles: u64) {
        self.y = self.y.wrapping_sub(1);
        self.set_nz_flags(self.y);

        self.pc += length;
        self.total_cycles += cycles
    }

    // Operation LDA: Load accumulator with memory.
    fn lda(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        self.a = bus.read_byte(address);
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation LDX: Load index X with memory.
    fn ldx(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        self.x = bus.read_byte(address);
        self.set_nz_flags(self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation LDY: Load index Y with memory.
    fn ldy(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        self.y = bus.read_byte(address);
        self.set_nz_flags(self.y);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation STA: Store accumulator in memory.
    fn sta(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        bus.write_byte(address, self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation STX: Store index X in memory.
    fn stx(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        bus.write_byte(address, self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation STY: Store index Y in memory.
    fn sty(&mut self, bus: &mut dyn Bus16, addr_mode: AddressingMode, length: u16, cycles: u64) {
        let address = self.resolve_address(bus, addr_mode);
        bus.write_byte(address, self.y);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation TAX: Transfer accumulator to index X.
    fn tax(&mut self, length: u16, cycles: u64) {
        self.x = self.a;
        self.set_nz_flags(self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation TXA: Transfer index X to accumulator.
    fn txa(&mut self, length: u16, cycles: u64) {
        self.a = self.x;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation TAY: Transfer accumulator to index Y.
    fn tay(&mut self, length: u16, cycles: u64) {
        self.y = self.a;
        self.set_nz_flags(self.y);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation TYA: Transfer index Y to accumulator.
    fn tya(&mut self, length: u16, cycles: u64) {
        self.a = self.y;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation TSX: Transfer stack pointer to index X.
    fn tsx(&mut self, length: u16, cycles: u64) {
        self.x = self.s;
        self.set_nz_flags(self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation TXS: Transfer index X to stack pointer.
    fn txs(&mut self, length: u16, cycles: u64) {
        self.s = self.x;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation SEC: Set carry flag.
    fn sec(&mut self, length: u16, cycles: u64) {
        self.carry = true;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation SED: Set decimal mode.
    fn sed(&mut self, length: u16, cycles: u64) {
        self.decimal_mode = true;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation SEI: Set interrupt disable bit.
    fn sei(&mut self, length: u16, cycles: u64) {
        self.irq_disable = true;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation CLC: Clear carry flag.
    fn clc(&mut self, length: u16, cycles: u64) {
        self.carry = false;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation CLD: Clear decimal mode.
    fn cld(&mut self, length: u16, cycles: u64) {
        self.decimal_mode = false;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation CLI: Clear interrupt disable bit.
    fn cli(&mut self, length: u16, cycles: u64) {
        self.irq_disable = false;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation CLV: Clear overflow flag.
    fn clv(&mut self, length: u16, cycles: u64) {
        self.overflow = false;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation NOP: No operation.
    fn nop(&mut self, length: u16, cycles: u64) {
        self.pc += length;
        self.total_cycles += cycles;
    }
}
