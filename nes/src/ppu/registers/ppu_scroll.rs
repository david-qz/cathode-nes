pub struct PpuScroll {
    offset_x: u8,
    offset_y: u8,
    first_write: bool,
}

impl PpuScroll {
    pub fn new() -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            first_write: true,
        }
    }

    pub fn write(&mut self, value: u8) {
        if self.first_write {
            self.offset_x = value;
        } else {
            self.offset_y = value;
        }
        self.first_write = !self.first_write;
    }

    pub fn offset_x(&self) -> u8 {
        self.offset_x
    }

    pub fn offset_y(&self) -> u8 {
        self.offset_y
    }

    pub fn reset_latch(&mut self) {
        self.offset_x = 0;
        self.offset_y = 0;
        self.first_write = true;
    }
}
