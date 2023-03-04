use crate::bus::Bus16;

const MEMORY_SIZE: usize = 2usize.pow(16);

/// A Bus16 implementation that provides a full 16-bit address space without mirroring or mapping.
pub struct FlatMemory {
    bytes: Box<[u8; MEMORY_SIZE]>,
}

impl FlatMemory {
    pub fn new() -> Self {
        Self {
            bytes: Box::new([0; MEMORY_SIZE]),
        }
    }
}

impl Bus16 for FlatMemory {
    fn read_byte(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.bytes[address as usize] = value;
    }
}
