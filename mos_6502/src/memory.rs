use crate::cpu::CPU;

/// A 16-bit bus.
pub trait Bus16 {
    fn peek_byte(&self, address: u16) -> u8;

    fn read_byte(&mut self, address: u16) -> u8;

    fn write_byte(&mut self, address: u16, value: u8);

    fn peek_word(&self, address: u16) -> u16 {
        let lower_byte = self.peek_byte(address.wrapping_add(0));
        let upper_byte = self.peek_byte(address.wrapping_add(1));
        (upper_byte as u16) << 8 | lower_byte as u16
    }

    fn read_word(&mut self, address: u16) -> u16 {
        let lower_byte = self.read_byte(address.wrapping_add(0));
        let upper_byte = self.read_byte(address.wrapping_add(1));
        (upper_byte as u16) << 8 | lower_byte as u16
    }

    fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address.wrapping_add(0), ((value & 0x00FF) >> 0) as u8);
        self.write_byte(address.wrapping_add(1), ((value & 0xFF00) >> 8) as u8);
    }

    fn load_code(&mut self, code: &[u8], base_address: u16, reset_vector: Option<u16>) {
        for i in 0..code.len() {
            self.write_byte(base_address.wrapping_add(i as u16), code[i]);
        }

        if let Some(reset_vector) = reset_vector {
            self.write_word(CPU::RESET_VECTOR, reset_vector);
        }
    }
}

/// A Bus16 implementation that provides a full 16-bit address space without mirroring or mapping.
pub struct FlatMemory {
    bytes: Box<[u8; Self::MEMORY_SIZE]>,
}

impl FlatMemory {
    const MEMORY_SIZE: usize = 2usize.pow(16);

    pub fn new() -> Self {
        Self {
            bytes: Box::new([0; Self::MEMORY_SIZE]),
        }
    }
}

impl Bus16 for FlatMemory {
    fn peek_byte(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    fn read_byte(&mut self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.bytes[address as usize] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_and_write_to_flat_memory() {
        let mut memory = FlatMemory::new();

        memory.write_byte(0x0000, 64);
        assert_eq!(memory.read_byte(0x0000), 64);

        memory.write_word(0x0100, 0xAABB);
        assert_eq!(memory.read_word(0x0100), 0xAABB);
        assert_eq!(memory.read_byte(0x0100), 0xBB);
        assert_eq!(memory.read_byte(0x0101), 0xAA);
    }
}
