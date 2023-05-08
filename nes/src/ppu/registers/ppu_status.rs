pub struct PpuStatus(u8);

impl PpuStatus {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn bits(&self) -> u8 {
        self.0
    }

    pub fn read(&mut self) -> u8 {
        let value = self.0;
        self.set_vblank_started(false);
        value
    }

    pub fn set_sprite_overflow(&mut self, status: bool) {
        self.0 &= 0xDF;
        self.0 |= (status as u8) << 5;
    }

    pub fn set_sprite_zero_hit(&mut self, status: bool) {
        self.0 &= 0xBF;
        self.0 |= (status as u8) << 6;
    }

    pub fn set_vblank_started(&mut self, status: bool) {
        self.0 &= 0x7F;
        self.0 |= (status as u8) << 7;
    }
}
