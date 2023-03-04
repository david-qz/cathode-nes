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

    pub fn clock(&mut self, bus: &mut dyn Bus16) {
        if self.should_run_reset_procedure {
            self.pc = bus.read_word(Self::RESET_VECTOR);
            self.s = 0xFF;
            return;
        }

        let opcode = bus.read_byte(self.pc);

        match opcode {
            _ => panic!("Unimplemented opcode"),
        }
    }
}
