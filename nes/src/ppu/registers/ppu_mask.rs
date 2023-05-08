pub struct PpuMask(u8);

impl PpuMask {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn write(&mut self, value: u8) {
        self.0 = value;
    }

    pub fn rendering_enabled(&self) -> bool {
        self.render_background() || self.render_sprites()
    }

    pub fn grayscale(&self) -> bool {
        self.0 & 0x01 != 0
    }

    pub fn render_background_in_left_margin(&self) -> bool {
        self.0 & 0x02 != 0
    }

    pub fn render_sprites_in_left_margin(&self) -> bool {
        self.0 & 0x04 != 0
    }

    pub fn render_background(&self) -> bool {
        self.0 & 0x08 != 0
    }

    pub fn render_sprites(&self) -> bool {
        self.0 & 0x10 != 0
    }

    pub fn emphasize_red(&self) -> bool {
        self.0 & 0x20 != 0
    }

    pub fn emphasize_green(&self) -> bool {
        self.0 & 0x40 != 0
    }

    pub fn emphasize_blue(&self) -> bool {
        self.0 & 0x80 != 0
    }
}
