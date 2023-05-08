pub struct OamAddr(u8);

impl OamAddr {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn bits(&self) -> u16 {
        self.0 as u16
    }

    pub fn write(&mut self, byte: u8) {
        self.0 = byte;
    }

    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    pub fn reset_latch(&mut self) {
        self.0 = 0;
    }
}
