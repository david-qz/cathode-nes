/// A 16-bit bus.
pub trait Bus16 {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, value: u8);

    fn read_word(&self, address: u16) -> u16 {
        let low_byte = self.read_byte(address + 0);
        let high_byte = self.read_byte(address + 1);
        (high_byte as u16) << 8 | low_byte as u16
    }

    fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address + 0, ((value & 0x00FF) >> 0) as u8);
        self.write_byte(address + 1, ((value & 0xFF00) >> 8) as u8);
    }
}
