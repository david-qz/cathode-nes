/// A 16-bit bus.
pub trait Bus16 {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, value: u8);
}
