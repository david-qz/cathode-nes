pub mod bus;
mod flat_memory;

#[cfg(test)]
mod tests {
    #[test]
    fn read_and_write_to_flat_memory() {
        use crate::bus::Bus16;
        use crate::flat_memory::FlatMemory;

        let mut memory = FlatMemory::new();

        assert_eq!(memory.read_byte(0x0000), 0);
        memory.write_byte(0x0000, 64);
        assert_eq!(memory.read_byte(0x0000), 64);
    }
}
