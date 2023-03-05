pub mod bus;
pub mod cpu;
pub mod flat_memory;

#[cfg(test)]
mod tests {
    #[test]
    fn read_and_write_to_flat_memory() {
        use crate::bus::Bus16;
        use crate::flat_memory::FlatMemory;

        let mut memory = FlatMemory::new();

        memory.write_byte(0x0000, 64);
        assert_eq!(memory.read_byte(0x0000), 64);

        memory.write_word(0x0100, 0xAABB);
        assert_eq!(memory.read_word(0x0100), 0xAABB);
        assert_eq!(memory.read_byte(0x0100), 0xBB);
        assert_eq!(memory.read_byte(0x0101), 0xAA);
    }
}
