pub struct PpuAddr(u16);

impl PpuAddr {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn bits(&self) -> u16 {
        self.0
    }

    pub fn write(&mut self, byte: u8) {
        self.0 <<= 8;
        self.0 |= byte as u16;
        self.0 &= 0x3FFF;
    }

    pub fn increment(&mut self, increment: u16) {
        self.0 = self.0.wrapping_add(increment);
        self.0 &= 0x3FFF;
    }

    pub fn reset_latch(&mut self) {
        self.0 = 0;
    }
}
