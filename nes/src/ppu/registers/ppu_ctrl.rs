use crate::ppu::rendering::SpriteSize;

pub struct PpuCtrl {
    value: u8,
}

impl PpuCtrl {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn write(&mut self, value: u8) {
        self.value = value;
    }

    pub fn nametable_base_address(&self) -> u16 {
        match self.value & 0x03 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        }
    }

    pub fn vram_address_increment(&self) -> u16 {
        match self.value & 0x04 != 0 {
            false => 1,
            true => 32,
        }
    }

    pub fn sprite_pattern_table_address_for_8x8(&self) -> u16 {
        match self.value & 0x08 != 0 {
            false => 0x0000,
            true => 0x1000,
        }
    }

    pub fn background_pattern_table_address(&self) -> u16 {
        match self.value & 0x10 != 0 {
            false => 0x0000,
            true => 0x1000,
        }
    }

    pub fn sprite_size(&self) -> SpriteSize {
        match self.value & 0x20 != 0 {
            false => SpriteSize::EightByEight,
            true => SpriteSize::EightBySixteen,
        }
    }

    pub fn nmi_enabled(&self) -> bool {
        (self.value & 0x80) != 0
    }
}
