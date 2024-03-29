use crate::{
    debugging::{Debugger, ExecutionState},
    memory::Bus16,
};
use std::{cell::RefCell, rc::Rc};

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

    pub nmi: bool,
    last_nmi: bool,
    pub irq: bool,

    pub total_cycles: u64,
    pub jammed: bool,
    debugger: Option<Rc<RefCell<Debugger>>>,
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
            irq_disable: true,
            decimal_mode: false,
            overflow: false,
            negative: false,
            nmi: false,
            last_nmi: false,
            irq: false,
            total_cycles: 0,
            jammed: false,
            debugger: None,
        }
    }

    pub fn attach_debugger(&mut self, debugger: Rc<RefCell<Debugger>>) {
        self.debugger = Some(debugger);
    }

    pub fn detach_debugger(&mut self) {
        self.debugger = None;
    }

    pub fn current_state(&self, bus: &dyn Bus16) -> ExecutionState {
        ExecutionState::new(self, bus)
    }

    pub fn status_register(&self) -> u8 {
        self.encode_p(false)
    }

    pub fn reset(&mut self, bus: &mut dyn Bus16) {
        self.pc = bus.read_word(Self::RESET_VECTOR);
        self.s = 0xFD;
        self.irq_disable = true;
        self.total_cycles += 7;
    }

    pub fn execute_instruction(&mut self, bus: &mut dyn Bus16) -> u64 {
        if self.jammed {
            return 1;
        }

        if let Some(debugger) = &self.debugger {
            debugger.borrow_mut().record_state(self.current_state(bus));
        }

        let cycles_at_start = self.total_cycles;

        if self.nmi && !self.last_nmi {
            self.nmi(bus);
        }
        self.last_nmi = self.nmi;

        if self.irq && !self.irq_disable {
            self.irq(bus);
        }

        let opcode = bus.read_byte(self.pc);
        match opcode {
            0x00 => self.brk(bus, 7),
            0x01 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.ora(bus, effective_address, 2, 6);
            }
            0x02 => self.jam(),
            0x03 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.slo(bus, effective_address, 2, 8);
            }
            0x04 => self.nop(2, 3),
            0x05 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.ora(bus, effective_address, 2, 3);
            }
            0x06 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.asl(bus, Some(effective_address), 2, 5);
            }
            0x07 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.slo(bus, effective_address, 2, 5);
            }
            0x08 => self.php(bus, 1, 3),
            0x09 => {
                let effective_address = self.resolve_address_immediate();
                self.ora(bus, effective_address, 2, 2);
            }
            0x0A => {
                // Accumulator addressing mode.
                self.asl(bus, None, 1, 2);
            }
            0x0B => {
                let effective_address = self.resolve_address_immediate();
                self.anc(bus, effective_address, 2, 2);
            }
            0x0C => self.nop(3, 4),
            0x0D => {
                let effective_address = self.resolve_address_absolute(bus);
                self.ora(bus, effective_address, 3, 4);
            }
            0x0E => {
                let effective_address = self.resolve_address_absolute(bus);
                self.asl(bus, Some(effective_address), 3, 6);
            }
            0x0F => {
                let effective_address = self.resolve_address_absolute(bus);
                self.slo(bus, effective_address, 3, 6);
            }
            0x10 => self.bpl(bus, 2, 2),
            0x11 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, true);
                self.ora(bus, effective_address, 2, 5);
            }
            0x12 => self.jam(),
            0x13 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, false);
                self.slo(bus, effective_address, 2, 8);
            }
            0x14 => self.nop(2, 4),
            0x15 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.ora(bus, effective_address, 2, 4);
            }
            0x16 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.asl(bus, Some(effective_address), 2, 6);
            }
            0x17 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.slo(bus, effective_address, 2, 6);
            }
            0x18 => self.clc(1, 2),
            0x19 => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, true);
                self.ora(bus, effective_address, 3, 4);
            }
            0x1A => self.nop(1, 2),
            0x1B => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, false);
                self.slo(bus, effective_address, 3, 7);
            }
            0x1C => {
                let _ = self.resolve_address_indexed_absolute_x(bus, true);
                self.nop(3, 4);
            }
            0x1D => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, true);
                self.ora(bus, effective_address, 3, 4);
            }
            0x1E => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.asl(bus, Some(effective_address), 3, 7);
            }
            0x1F => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.slo(bus, effective_address, 3, 7);
            }
            0x20 => {
                let effective_address = self.resolve_address_absolute(bus);
                self.jsr(bus, effective_address, 3, 6);
            }
            0x21 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.and(bus, effective_address, 2, 6);
            }
            0x22 => self.jam(),
            0x23 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.rla(bus, effective_address, 2, 8);
            }
            0x24 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.bit(bus, effective_address, 2, 3);
            }
            0x25 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.and(bus, effective_address, 2, 3);
            }
            0x26 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.rol(bus, Some(effective_address), 2, 5);
            }
            0x27 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.rla(bus, effective_address, 2, 5);
            }
            0x28 => self.plp(bus, 1, 4),
            0x29 => {
                let effective_address = self.resolve_address_immediate();
                self.and(bus, effective_address, 2, 2);
            }
            0x2A => {
                // Accumulator addressing mode.
                self.rol(bus, None, 1, 2);
            }
            0x2B => {
                let effective_address = self.resolve_address_immediate();
                self.anc(bus, effective_address, 2, 2);
            }
            0x2C => {
                let effective_address = self.resolve_address_absolute(bus);
                self.bit(bus, effective_address, 3, 4);
            }
            0x2D => {
                let effective_address = self.resolve_address_absolute(bus);
                self.and(bus, effective_address, 3, 4);
            }
            0x2E => {
                let effective_address = self.resolve_address_absolute(bus);
                self.rol(bus, Some(effective_address), 3, 6);
            }
            0x2F => {
                let effective_address = self.resolve_address_absolute(bus);
                self.rla(bus, effective_address, 3, 6);
            }
            0x30 => self.bmi(bus, 2, 2),
            0x31 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, true);
                self.and(bus, effective_address, 2, 5);
            }
            0x32 => self.jam(),
            0x33 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, false);
                self.rla(bus, effective_address, 2, 8);
            }
            0x34 => self.nop(2, 4),
            0x35 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.and(bus, effective_address, 2, 4);
            }
            0x36 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.rol(bus, Some(effective_address), 2, 6);
            }
            0x37 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.rla(bus, effective_address, 2, 6);
            }
            0x38 => self.sec(1, 2),
            0x39 => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, true);
                self.and(bus, effective_address, 3, 4);
            }
            0x3A => self.nop(1, 2),
            0x3B => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, false);
                self.rla(bus, effective_address, 3, 7);
            }
            0x3C => {
                let _ = self.resolve_address_indexed_absolute_x(bus, true);
                self.nop(3, 4);
            }
            0x3D => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, true);
                self.and(bus, effective_address, 3, 4);
            }
            0x3E => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.rol(bus, Some(effective_address), 3, 7);
            }
            0x3F => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.rla(bus, effective_address, 3, 7);
            }
            0x40 => self.rti(bus, 6),
            0x41 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.eor(bus, effective_address, 2, 6);
            }
            0x42 => self.jam(),
            0x43 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.sre(bus, effective_address, 2, 8);
            }
            0x44 => self.nop(2, 3),
            0x45 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.eor(bus, effective_address, 2, 3);
            }
            0x46 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.lsr(bus, Some(effective_address), 2, 5);
            }
            0x47 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.sre(bus, effective_address, 2, 5);
            }
            0x48 => self.pha(bus, 1, 3),
            0x49 => {
                let effective_address = self.resolve_address_immediate();
                self.eor(bus, effective_address, 2, 2);
            }
            0x4A => {
                // Accumulator addressing mode.
                self.lsr(bus, None, 1, 2);
            }
            0x4B => {
                let effective_address = self.resolve_address_immediate();
                self.alr(bus, effective_address, 2, 2);
            }
            0x4C => {
                let effective_address = self.resolve_address_absolute(bus);
                self.jmp(effective_address, 3)
            }
            0x4D => {
                let effective_address = self.resolve_address_absolute(bus);
                self.eor(bus, effective_address, 3, 4);
            }
            0x4E => {
                let effective_address = self.resolve_address_absolute(bus);
                self.lsr(bus, Some(effective_address), 3, 6);
            }
            0x4F => {
                let effective_address = self.resolve_address_absolute(bus);
                self.sre(bus, effective_address, 3, 6);
            }
            0x50 => self.bvc(bus, 2, 2),
            0x51 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, true);
                self.eor(bus, effective_address, 2, 5);
            }
            0x52 => self.jam(),
            0x53 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, false);
                self.sre(bus, effective_address, 2, 8);
            }
            0x54 => self.nop(2, 4),
            0x55 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.eor(bus, effective_address, 2, 4);
            }
            0x56 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.lsr(bus, Some(effective_address), 2, 6);
            }
            0x57 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.sre(bus, effective_address, 2, 6);
            }
            0x58 => self.cli(1, 2),
            0x59 => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, true);
                self.eor(bus, effective_address, 3, 4);
            }
            0x5A => self.nop(1, 2),
            0x5B => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, false);
                self.sre(bus, effective_address, 3, 7);
            }
            0x5C => {
                let _ = self.resolve_address_indexed_absolute_x(bus, true);
                self.nop(3, 4);
            }
            0x5D => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, true);
                self.eor(bus, effective_address, 3, 4);
            }
            0x5E => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.lsr(bus, Some(effective_address), 3, 7);
            }
            0x5F => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.sre(bus, effective_address, 3, 7);
            }
            0x60 => self.rts(bus, 6),
            0x61 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.adc(bus, effective_address, 2, 6)
            }
            0x62 => self.jam(),
            0x63 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.rra(bus, effective_address, 2, 8);
            }
            0x64 => self.nop(2, 3),
            0x65 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.adc(bus, effective_address, 2, 3)
            }
            0x66 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.ror(bus, Some(effective_address), 2, 5);
            }
            0x67 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.rra(bus, effective_address, 2, 5);
            }
            0x68 => self.pla(bus, 1, 4),
            0x69 => {
                let effective_address = self.resolve_address_immediate();
                self.adc(bus, effective_address, 2, 2)
            }
            0x6A => {
                // Accumulator addressing mode.
                self.ror(bus, None, 1, 2);
            }
            0x6B => {
                let effective_address = self.resolve_address_immediate();
                self.arr(bus, effective_address, 2, 2);
            }
            0x6C => {
                let effective_address = self.resolve_address_absolute_indirect(bus);
                self.jmp(effective_address, 5)
            }
            0x6D => {
                let effective_address = self.resolve_address_absolute(bus);
                self.adc(bus, effective_address, 3, 4)
            }
            0x6E => {
                let effective_address = self.resolve_address_absolute(bus);
                self.ror(bus, Some(effective_address), 3, 6);
            }
            0x6F => {
                let effective_address = self.resolve_address_absolute(bus);
                self.rra(bus, effective_address, 3, 6);
            }
            0x70 => self.bvs(bus, 2, 2),
            0x71 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, true);
                self.adc(bus, effective_address, 2, 5);
            }
            0x72 => self.jam(),
            0x73 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, false);
                self.rra(bus, effective_address, 2, 8);
            }
            0x74 => self.nop(2, 4),
            0x75 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.adc(bus, effective_address, 2, 4)
            }
            0x76 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.ror(bus, Some(effective_address), 2, 6);
            }
            0x77 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.rra(bus, effective_address, 2, 6);
            }
            0x78 => self.sei(1, 2),
            0x79 => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, true);
                self.adc(bus, effective_address, 3, 4);
            }
            0x7A => self.nop(1, 2),
            0x7B => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, false);
                self.rra(bus, effective_address, 3, 7);
            }
            0x7C => {
                let _ = self.resolve_address_indexed_absolute_x(bus, true);
                self.nop(3, 4);
            }
            0x7D => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, true);
                self.adc(bus, effective_address, 3, 4);
            }
            0x7E => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.ror(bus, Some(effective_address), 3, 7);
            }
            0x7F => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.rra(bus, effective_address, 3, 7);
            }
            0x80 => self.nop(2, 2),
            0x81 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.sta(bus, effective_address, 2, 6);
            }
            0x82 => self.nop(2, 2),
            0x83 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.sax(bus, effective_address, 2, 6);
            }
            0x84 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.sty(bus, effective_address, 2, 3);
            }
            0x85 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.sta(bus, effective_address, 2, 3);
            }
            0x86 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.stx(bus, effective_address, 2, 3);
            }
            0x87 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.sax(bus, effective_address, 2, 3);
            }
            0x88 => self.dey(1, 2),
            0x89 => self.nop(2, 2),
            0x8A => self.txa(1, 2),
            0x8B => {
                let effective_address = self.resolve_address_immediate();
                self.xaa(bus, effective_address, 2, 2);
            }
            0x8C => {
                let effective_address = self.resolve_address_absolute(bus);
                self.sty(bus, effective_address, 3, 4);
            }
            0x8D => {
                let effective_address = self.resolve_address_absolute(bus);
                self.sta(bus, effective_address, 3, 4);
            }
            0x8E => {
                let effective_address = self.resolve_address_absolute(bus);
                self.stx(bus, effective_address, 3, 4);
            }
            0x8F => {
                let effective_address = self.resolve_address_absolute(bus);
                self.sax(bus, effective_address, 3, 4);
            }
            0x90 => self.bcc(bus, 2, 2),
            0x91 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, false);
                self.sta(bus, effective_address, 2, 6);
            }
            0x92 => self.jam(),
            0x93 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, false);
                self.sha(bus, effective_address, 2, 6);
            }
            0x94 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.sty(bus, effective_address, 2, 4);
            }
            0x95 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.sta(bus, effective_address, 2, 4);
            }
            0x96 => {
                let effective_address = self.resolve_address_indexed_zero_page_y(bus);
                self.stx(bus, effective_address, 2, 4);
            }
            0x97 => {
                let effective_address = self.resolve_address_indexed_zero_page_y(bus);
                self.sax(bus, effective_address, 2, 4);
            }
            0x98 => self.tya(1, 2),
            0x99 => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, false);
                self.sta(bus, effective_address, 3, 5);
            }
            0x9A => self.txs(1, 2),
            0x9B => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, false);
                self.tas(bus, effective_address, 3, 5);
            }
            0x9C => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.shy(bus, effective_address, 3, 5);
            }
            0x9D => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.sta(bus, effective_address, 3, 5);
            }
            0x9E => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, false);
                self.shx(bus, effective_address, 3, 5);
            }
            0x9F => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, false);
                self.sha(bus, effective_address, 3, 5);
            }
            0xA0 => {
                let effective_address = self.resolve_address_immediate();
                self.ldy(bus, effective_address, 2, 2);
            }
            0xA1 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.lda(bus, effective_address, 2, 6);
            }
            0xA2 => {
                let effective_address = self.resolve_address_immediate();
                self.ldx(bus, effective_address, 2, 2);
            }
            0xA3 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.lax(bus, effective_address, 2, 6);
            }
            0xA4 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.ldy(bus, effective_address, 2, 3);
            }
            0xA5 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.lda(bus, effective_address, 2, 3);
            }
            0xA6 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.ldx(bus, effective_address, 2, 3);
            }
            0xA7 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.lax(bus, effective_address, 2, 3);
            }
            0xA8 => self.tay(1, 2),
            0xA9 => {
                let effective_address = self.resolve_address_immediate();
                self.lda(bus, effective_address, 2, 2);
            }
            0xAA => self.tax(1, 2),
            0xAB => {
                let effective_address = self.resolve_address_immediate();
                self.lxa(bus, effective_address, 2, 2);
            }
            0xAC => {
                let effective_address = self.resolve_address_absolute(bus);
                self.ldy(bus, effective_address, 3, 4);
            }
            0xAD => {
                let effective_address = self.resolve_address_absolute(bus);
                self.lda(bus, effective_address, 3, 4);
            }
            0xAE => {
                let effective_address = self.resolve_address_absolute(bus);
                self.ldx(bus, effective_address, 3, 4);
            }
            0xAF => {
                let effective_address = self.resolve_address_absolute(bus);
                self.lax(bus, effective_address, 3, 4);
            }
            0xB0 => self.bcs(bus, 2, 2),
            0xB1 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, true);
                self.lda(bus, effective_address, 2, 5);
            }
            0xB2 => self.jam(),
            0xB3 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, true);
                self.lax(bus, effective_address, 2, 5);
            }
            0xB4 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.ldy(bus, effective_address, 2, 4);
            }
            0xB5 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.lda(bus, effective_address, 2, 4);
            }
            0xB6 => {
                let effective_address = self.resolve_address_indexed_zero_page_y(bus);
                self.ldx(bus, effective_address, 2, 4);
            }
            0xB7 => {
                let effective_address = self.resolve_address_indexed_zero_page_y(bus);
                self.lax(bus, effective_address, 2, 4);
            }
            0xB8 => self.clv(1, 2),
            0xB9 => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, true);
                self.lda(bus, effective_address, 3, 4);
            }
            0xBA => self.tsx(1, 2),
            0xBB => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, true);
                self.las(bus, effective_address, 3, 4);
            }
            0xBC => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, true);
                self.ldy(bus, effective_address, 3, 4);
            }
            0xBD => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, true);
                self.lda(bus, effective_address, 3, 4);
            }
            0xBE => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, true);
                self.ldx(bus, effective_address, 3, 4);
            }
            0xBF => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, true);
                self.lax(bus, effective_address, 3, 4);
            }
            0xC0 => {
                let effective_address = self.resolve_address_immediate();
                self.cpy(bus, effective_address, 2, 2);
            }
            0xC1 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.cmp(bus, effective_address, 2, 6);
            }
            0xC2 => self.nop(2, 2),
            0xC3 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.dcp(bus, effective_address, 2, 8);
            }
            0xC4 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.cpy(bus, effective_address, 2, 3);
            }
            0xC5 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.cmp(bus, effective_address, 2, 3);
            }
            0xC6 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.dec(bus, effective_address, 2, 5);
            }
            0xC7 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.dcp(bus, effective_address, 2, 5);
            }
            0xC8 => self.iny(1, 2),
            0xC9 => {
                let effective_address = self.resolve_address_immediate();
                self.cmp(bus, effective_address, 2, 2);
            }
            0xCA => self.dex(1, 2),
            0xCB => {
                let effective_address = self.resolve_address_immediate();
                self.sbx(bus, effective_address, 2, 2);
            }
            0xCC => {
                let effective_address = self.resolve_address_absolute(bus);
                self.cpy(bus, effective_address, 3, 4);
            }
            0xCD => {
                let effective_address = self.resolve_address_absolute(bus);
                self.cmp(bus, effective_address, 3, 4);
            }
            0xCE => {
                let effective_address = self.resolve_address_absolute(bus);
                self.dec(bus, effective_address, 3, 6);
            }
            0xCF => {
                let effective_address = self.resolve_address_absolute(bus);
                self.dcp(bus, effective_address, 3, 6);
            }
            0xD0 => self.bne(bus, 2, 2),
            0xD1 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, true);
                self.cmp(bus, effective_address, 2, 5);
            }
            0xD2 => self.jam(),
            0xD3 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, false);
                self.dcp(bus, effective_address, 2, 8);
            }
            0xD4 => self.nop(2, 4),
            0xD5 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.cmp(bus, effective_address, 2, 4);
            }
            0xD6 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.dec(bus, effective_address, 2, 6);
            }
            0xD7 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.dcp(bus, effective_address, 2, 6);
            }
            0xD8 => self.cld(1, 2),
            0xD9 => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, true);
                self.cmp(bus, effective_address, 3, 4);
            }
            0xDA => self.nop(1, 2),
            0xDB => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, false);
                self.dcp(bus, effective_address, 3, 7);
            }
            0xDC => {
                let _ = self.resolve_address_indexed_absolute_x(bus, true);
                self.nop(3, 4);
            }
            0xDD => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, true);
                self.cmp(bus, effective_address, 3, 4);
            }
            0xDE => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.dec(bus, effective_address, 3, 7);
            }
            0xDF => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.dcp(bus, effective_address, 3, 7);
            }
            0xE0 => {
                let effective_address = self.resolve_address_immediate();
                self.cpx(bus, effective_address, 2, 2);
            }
            0xE1 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.sbc(bus, effective_address, 2, 6);
            }
            0xE2 => self.nop(2, 2),
            0xE3 => {
                let effective_address = self.resolve_address_indexed_indirect_x(bus);
                self.isc(bus, effective_address, 2, 8);
            }
            0xE4 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.cpx(bus, effective_address, 2, 3);
            }
            0xE5 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.sbc(bus, effective_address, 2, 3);
            }
            0xE6 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.inc(bus, effective_address, 2, 5);
            }
            0xE7 => {
                let effective_address = self.resolve_address_zero_page(bus);
                self.isc(bus, effective_address, 2, 5);
            }
            0xE8 => self.inx(1, 2),
            0xE9 => {
                let effective_address = self.resolve_address_immediate();
                self.sbc(bus, effective_address, 2, 2);
            }
            0xEA => self.nop(1, 2),
            0xEB => {
                let effective_address = self.resolve_address_immediate();
                self.sbc(bus, effective_address, 2, 2);
            }
            0xEC => {
                let effective_address = self.resolve_address_absolute(bus);
                self.cpx(bus, effective_address, 3, 4);
            }
            0xED => {
                let effective_address = self.resolve_address_absolute(bus);
                self.sbc(bus, effective_address, 3, 4);
            }
            0xEE => {
                let effective_address = self.resolve_address_absolute(bus);
                self.inc(bus, effective_address, 3, 6);
            }
            0xEF => {
                let effective_address = self.resolve_address_absolute(bus);
                self.isc(bus, effective_address, 3, 6);
            }
            0xF0 => self.beq(bus, 2, 2),
            0xF1 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, true);
                self.sbc(bus, effective_address, 2, 5);
            }
            0xF2 => self.jam(),
            0xF3 => {
                let effective_address = self.resolve_address_indirect_indexed_y(bus, false);
                self.isc(bus, effective_address, 2, 8);
            }
            0xF4 => self.nop(2, 4),
            0xF5 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.sbc(bus, effective_address, 2, 4);
            }
            0xF6 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.inc(bus, effective_address, 2, 6);
            }
            0xF7 => {
                let effective_address = self.resolve_address_indexed_zero_page_x(bus);
                self.isc(bus, effective_address, 2, 6);
            }
            0xF8 => self.sed(1, 2),
            0xF9 => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, true);
                self.sbc(bus, effective_address, 3, 4);
            }
            0xFA => self.nop(1, 2),
            0xFB => {
                let effective_address = self.resolve_address_indexed_absolute_y(bus, false);
                self.isc(bus, effective_address, 3, 7);
            }
            0xFC => {
                let _ = self.resolve_address_indexed_absolute_x(bus, true);
                self.nop(3, 4);
            }
            0xFD => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, true);
                self.sbc(bus, effective_address, 3, 4);
            }
            0xFE => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.inc(bus, effective_address, 3, 7);
            }
            0xFF => {
                let effective_address = self.resolve_address_indexed_absolute_x(bus, false);
                self.isc(bus, effective_address, 3, 7);
            }
        };

        self.total_cycles - cycles_at_start
    }

    fn nmi(&mut self, bus: &mut dyn Bus16) {
        self.push_word(bus, self.pc);
        self.push_byte(bus, self.encode_p(false));
        self.irq_disable = true;
        self.pc = bus.read_word(Self::NMI_VECTOR);
        self.total_cycles += 7;
    }

    fn irq(&mut self, bus: &mut dyn Bus16) {
        self.push_word(bus, self.pc);
        self.push_byte(bus, self.encode_p(false));
        self.irq_disable = true;
        self.pc = bus.read_word(Self::IRQ_VECTOR);
        self.total_cycles += 7;
    }

    #[inline(always)]
    fn crosses_page_boundary(a: u16, b: u16) -> bool {
        a & 0xFF00 != b & 0xFF00
    }

    fn resolve_address_immediate(&self) -> u16 {
        self.pc + 1
    }

    fn resolve_address_absolute(&self, bus: &mut dyn Bus16) -> u16 {
        bus.read_word(self.pc + 1)
    }

    fn resolve_address_zero_page(&self, bus: &mut dyn Bus16) -> u16 {
        bus.read_byte(self.pc + 1) as u16
    }

    fn resolve_address_indexed_zero_page_x(&self, bus: &mut dyn Bus16) -> u16 {
        let base_address = bus.read_byte(self.pc + 1);
        base_address.wrapping_add(self.x) as u16
    }

    fn resolve_address_indexed_zero_page_y(&self, bus: &mut dyn Bus16) -> u16 {
        let base_address = bus.read_byte(self.pc + 1);
        base_address.wrapping_add(self.y) as u16
    }

    fn resolve_address_indexed_absolute_x(
        &mut self,
        bus: &mut dyn Bus16,
        extra_cycles: bool,
    ) -> u16 {
        let base_address = bus.read_word(self.pc + 1);
        let effective_address = base_address.wrapping_add(self.x as u16);
        if extra_cycles && CPU::crosses_page_boundary(base_address, effective_address) {
            self.total_cycles += 1;
        }
        effective_address
    }

    fn resolve_address_indexed_absolute_y(
        &mut self,
        bus: &mut dyn Bus16,
        extra_cycles: bool,
    ) -> u16 {
        let base_address = bus.read_word(self.pc + 1);
        let effective_address = base_address.wrapping_add(self.y as u16);
        if extra_cycles && CPU::crosses_page_boundary(base_address, effective_address) {
            self.total_cycles += 1;
        }
        effective_address
    }

    fn read_word_with_page_wrapping(bus: &mut dyn Bus16, address: u16) -> u16 {
        let low_byte = bus.read_byte(address);
        let high_byte = bus.read_byte(address & 0xFF00 | address.wrapping_add(1) & 0x00FF);
        (high_byte as u16) << 8 | low_byte as u16
    }

    fn resolve_address_indexed_indirect_x(&self, bus: &mut dyn Bus16) -> u16 {
        let base_address = bus.read_byte(self.pc + 1);
        let indirect_address = base_address.wrapping_add(self.x) as u16;
        CPU::read_word_with_page_wrapping(bus, indirect_address)
    }

    fn resolve_address_indirect_indexed_y(
        &mut self,
        bus: &mut dyn Bus16,
        extra_cycles: bool,
    ) -> u16 {
        let indirect_address = bus.read_byte(self.pc + 1) as u16;
        let base_address = CPU::read_word_with_page_wrapping(bus, indirect_address);
        let effective_address = base_address.wrapping_add(self.y as u16);
        if extra_cycles && CPU::crosses_page_boundary(base_address, effective_address) {
            self.total_cycles += 1;
        }
        effective_address
    }

    fn resolve_address_absolute_indirect(&self, bus: &mut dyn Bus16) -> u16 {
        let indirect_address = bus.read_word(self.pc + 1);
        CPU::read_word_with_page_wrapping(bus, indirect_address)
    }

    fn resolve_address_relative(&self, bus: &mut dyn Bus16) -> u16 {
        // NOTE: This is the only addressing mode helper that is supposed to be called after the PC has been incremented
        //       by the instruction length. It makes the offset math a little easier this way as the 6502 would have
        //       incremented its PC twice before calculating the offset addition also.
        let offset = (bus.read_byte(self.pc - 1) as i8) as i16;
        self.pc.wrapping_add_signed(offset)
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
    fn jsr(&mut self, bus: &mut dyn Bus16, jmp_address: u16, length: u16, cycles: u64) {
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

    fn adder(rhs: u8, lhs: u8, carry: bool) -> (u8, bool, bool) {
        let (sum, carry1) = rhs.overflowing_add(lhs);
        let (sum, carry2) = sum.overflowing_add(carry as u8);
        (
            sum,
            carry1 || carry2,
            ((sum ^ rhs) & (sum ^ lhs) & (1 << 7)) != 0,
        )
    }

    // Operation ADC: Add memory to accumulator with carry.
    fn adc(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        let (sum, carry, overflow) = CPU::adder(self.a, value, self.carry);
        self.a = sum;
        self.carry = carry;
        self.overflow = overflow;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation SBC: Subtract memory from accumulator with borrow.
    fn sbc(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        let (sum, carry, overflow) = CPU::adder(self.a, !value, self.carry);
        self.a = sum;
        self.carry = carry;
        self.overflow = overflow;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation AND: "AND" memory with accumulator.
    fn and(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        self.a = self.a & value;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation ORA: "OR" memory with accumulator.
    fn ora(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        self.a = self.a | value;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation EOR: "XOR" memory with accumulator.
    fn eor(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        self.a = self.a ^ value;
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation BIT: Test bits in memory with accumulator.
    fn bit(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        self.zero = self.a & value == 0;
        self.negative = value & 0b10000000 != 0;
        self.overflow = value & 0b01000000 != 0;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation ASL: Shift left one bit (memory or accumulator).
    fn asl(&mut self, bus: &mut dyn Bus16, address: Option<u16>, length: u16, cycles: u64) {
        let value = match address {
            Some(address) => bus.read_byte(address),
            None => self.a,
        };

        let result = value << 1;
        self.set_nz_flags(result);
        self.carry = value & (1 << 7) != 0;

        match address {
            Some(address) => bus.write_byte(address, result),
            None => self.a = result,
        }

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation LSR: Shift right one bit (memory or accumulator).
    fn lsr(&mut self, bus: &mut dyn Bus16, address: Option<u16>, length: u16, cycles: u64) {
        let value = match address {
            Some(address) => bus.read_byte(address),
            None => self.a,
        };

        let result = value >> 1;
        self.set_nz_flags(result);
        self.carry = value & (1 << 0) != 0;

        match address {
            Some(address) => bus.write_byte(address, result),
            None => self.a = result,
        }

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation ROL: Rotate left one bit (memory or accumulator).
    fn rol(&mut self, bus: &mut dyn Bus16, address: Option<u16>, length: u16, cycles: u64) {
        let value = match address {
            Some(address) => bus.read_byte(address),
            None => self.a,
        };

        let result = value.rotate_left(1) & 0b11111110 | (self.carry as u8) << 0;
        self.carry = value & (1 << 7) != 0;
        self.set_nz_flags(result);

        match address {
            Some(address) => bus.write_byte(address, result),
            None => self.a = result,
        }

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation ROR: Rotate right one bit (memory or accumulator).
    fn ror(&mut self, bus: &mut dyn Bus16, address: Option<u16>, length: u16, cycles: u64) {
        let value = match address {
            Some(address) => bus.read_byte(address),
            None => self.a,
        };

        let result = value.rotate_right(1) & 0b01111111 | (self.carry as u8) << 7;
        self.carry = value & (1 << 0) != 0;
        self.set_nz_flags(result);

        match address {
            Some(address) => bus.write_byte(address, result),
            None => self.a = result,
        }

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation JMP: Jump to new location.
    fn jmp(&mut self, jmp_address: u16, cycles: u64) {
        self.pc = jmp_address;
        self.total_cycles += cycles;
    }

    fn relative_conditional_branch(&mut self, bus: &mut dyn Bus16, should_branch: bool) {
        if should_branch {
            let target_address = self.resolve_address_relative(bus);

            if CPU::crosses_page_boundary(self.pc, target_address) {
                self.total_cycles += 2;
            } else {
                self.total_cycles += 1;
            }

            self.pc = target_address;
        }
    }

    // Operation BEQ: Branch on result zero.
    fn beq(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.pc += length;
        self.relative_conditional_branch(bus, self.zero);
        self.total_cycles += cycles;
    }

    // Operation BNE: Branch on result not zero.
    fn bne(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.pc += length;
        self.relative_conditional_branch(bus, !self.zero);
        self.total_cycles += cycles;
    }

    // Operation BCC: Branch on carry clear.
    fn bcc(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.pc += length;
        self.relative_conditional_branch(bus, !self.carry);
        self.total_cycles += cycles;
    }

    // Operation BCS: Branch on carry set.
    fn bcs(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.pc += length;
        self.relative_conditional_branch(bus, self.carry);
        self.total_cycles += cycles;
    }

    // Operation BVC: Branch on overflow clear.
    fn bvc(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.pc += length;
        self.relative_conditional_branch(bus, !self.overflow);
        self.total_cycles += cycles;
    }

    // Operation BVS: Branch on overflow set.
    fn bvs(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.pc += length;
        self.relative_conditional_branch(bus, self.overflow);
        self.total_cycles += cycles;
    }

    // Operation BMI: Branch on result minus.
    fn bmi(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.pc += length;
        self.relative_conditional_branch(bus, self.negative);
        self.total_cycles += cycles;
    }

    // Operation BPL: Branch on result plus.
    fn bpl(&mut self, bus: &mut dyn Bus16, length: u16, cycles: u64) {
        self.pc += length;
        self.relative_conditional_branch(bus, !self.negative);
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
    fn cmp(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        self.compare_value(self.a, value);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation CPX: Compare memory and index X.
    fn cpx(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        self.compare_value(self.x, value);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation CPY: Compare memory and index Y.
    fn cpy(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        self.compare_value(self.y, value);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation INC: Increment memory by one.
    fn inc(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        let result = value.wrapping_add(1);
        self.set_nz_flags(result);
        bus.write_byte(address, result);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation DEC: Decrement memory by one.
    fn dec(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
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
    fn lda(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.a = bus.read_byte(address);
        self.set_nz_flags(self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation LDX: Load index X with memory.
    fn ldx(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.x = bus.read_byte(address);
        self.set_nz_flags(self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation LDY: Load index Y with memory.
    fn ldy(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.y = bus.read_byte(address);
        self.set_nz_flags(self.y);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation STA: Store accumulator in memory.
    fn sta(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        bus.write_byte(address, self.a);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation STX: Store index X in memory.
    fn stx(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        bus.write_byte(address, self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // Operation STY: Store index Y in memory.
    fn sty(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
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

    // "Illegal" operation LAX: LDA + LDX
    fn lax(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        self.a = value;
        self.x = value;
        self.set_nz_flags(value);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation SAX: A & X -> M
    fn sax(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = self.a & self.x;
        bus.write_byte(address, value);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation DCP: DEC + CMP
    fn dcp(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.dec(bus, address, 0, 0);
        self.cmp(bus, address, 0, 0);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation ISC: INC + SBC
    fn isc(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.inc(bus, address, 0, 0);
        self.sbc(bus, address, 0, 0);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation SLO: ASL + ORA
    fn slo(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.asl(bus, Some(address), 0, 0);
        self.ora(bus, address, 0, 0);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation RLA: ROL + AND
    fn rla(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.rol(bus, Some(address), 0, 0);
        self.and(bus, address, 0, 0);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation SRE: LSR + EOR
    fn sre(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.lsr(bus, Some(address), 0, 0);
        self.eor(bus, address, 0, 0);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation RRA: ROR + ADC
    fn rra(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.ror(bus, Some(address), 0, 0);
        self.adc(bus, address, 0, 0);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation ANC: AND + set carry flag from bit 7
    fn anc(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.and(bus, address, 0, 0);
        self.carry = self.negative; // AND will set negative flag from bit 7

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation ALR: AND + LSR
    fn alr(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.and(bus, address, 0, 0);
        self.lsr(bus, None, 0, 0);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation ARR: AND + ROR
    fn arr(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.and(bus, address, 0, 0);
        self.ror(bus, None, 0, 0);

        self.carry = self.a & (1 << 6) != 0;
        self.overflow = self.a & (1 << 6) ^ self.a & (1 << 5) != 0;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation XAA: AND X + AND oper
    fn xaa(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.a = self.a & self.x;
        self.and(bus, address, 0, 0);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation LXA: Store * AND oper in A and X
    fn lxa(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.and(bus, address, 0, 0);
        self.x = self.a;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation SHA: Store A AND X AND (high-byte of address + 1) at address
    fn sha(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let result = self.a & self.x & ((address >> 8) as u8).wrapping_add(1);
        bus.write_byte(address, result);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation SHX: Store X AND (high-byte of address + 1) at address
    fn shx(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let result = self.x & ((address >> 8) as u8).wrapping_add(1);
        bus.write_byte(address, result);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation SHY: Store Y AND (high-byte of address + 1) at address
    fn shy(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let result = self.y & ((address >> 8) as u8).wrapping_add(1);
        bus.write_byte(address, result);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation TAS: Puts A AND X in SP and stores A AND X AND (high-byte of addr. + 1) at addr.
    fn tas(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        self.s = self.a & self.x;
        self.sha(bus, address, 0, 0);

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation LAS: M AND SP -> A, X, SP
    fn las(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        let result = value & self.s;
        self.a = result;
        self.x = result;
        self.s = result;

        self.pc += length;
        self.total_cycles += cycles;
    }

    // "Illegal" operation SBX: (A AND X) - oper -> X
    fn sbx(&mut self, bus: &mut dyn Bus16, address: u16, length: u16, cycles: u64) {
        let value = bus.read_byte(address);
        let intermediate = self.a & self.x;

        self.x = intermediate.wrapping_sub(value);
        self.compare_value(intermediate, self.x);

        self.pc += length;
        self.total_cycles += cycles;
    }

    fn jam(&mut self) {
        self.jammed = true;
    }
}
