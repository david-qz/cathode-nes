pub struct PpuStatus {
    value: u8,
}

impl PpuStatus {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn read(&mut self) -> u8 {
        let value = self.value;
        self.set_vblank_started(false);
        value
    }

    pub fn set_sprite_overflow(&mut self, status: bool) {
        self.value &= 0xDF;
        self.value |= (status as u8) << 5;
    }

    pub fn set_sprite_zero_hit(&mut self, status: bool) {
        self.value &= 0xBF;
        self.value |= (status as u8) << 6;
    }

    pub fn set_vblank_started(&mut self, status: bool) {
        self.value &= 0x7F;
        self.value |= (status as u8) << 7;
    }
}
