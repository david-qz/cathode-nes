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
}

impl CPU {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0, // TODO: read reset vector and set program counter appropriately
            s: 0,  // TODO: determine where the stack pointer should be initialized
            carry: false,
            zero: false,
            irq_disable: false,
            decimal_mode: false,
            brk_command: false,
            overflow: false,
            negative: false,
        }
    }

    pub fn clock(&mut self, bus: &mut dyn Bus16) {
        let opcode = bus.read_byte(self.pc);

        match opcode {
            _ => panic!("Unimplemented opcode"),
        }
    }
}
