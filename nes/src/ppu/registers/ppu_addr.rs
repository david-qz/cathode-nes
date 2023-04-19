pub struct PpuAddr {
    address: u16,
}

impl PpuAddr {
    pub fn new() -> Self {
        Self { address: 0 }
    }

    pub fn write(&mut self, value: u8) {
        self.address <<= 8;
        self.address |= value as u16;
        self.address &= 0x3FFF;
    }

    pub fn get(&self) -> u16 {
        self.address
    }

    pub fn reset_latch(&mut self) {
        self.address = 0;
    }

    pub fn increment_address(&mut self, increment: u16) {
        self.address = self.address.wrapping_add(increment);
        self.address &= 0x3FFF;
    }
}
