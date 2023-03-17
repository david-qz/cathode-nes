use mos_6502::memory::Bus16;

pub struct PPU {
    pub ppu_ctrl: u8,
    pub ppu_mask: u8,
    pub ppu_status: u8,
    pub oam_addr: u8,
    pub oam_data: u8,
    pub ppu_scroll: u8,
    pub ppu_addr: u8,
    pub ppu_data: u8,
    pub oam_dma: u8,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            ppu_ctrl: 0,
            ppu_mask: 0,
            ppu_status: 0,
            oam_addr: 0,
            oam_data: 0,
            ppu_scroll: 0,
            ppu_addr: 0,
            ppu_data: 0,
            oam_dma: 0,
        }
    }

    pub fn tick(&mut self, bus: &mut dyn Bus16, cycles: u64) {}

    pub fn get_register(&self, address: u16) -> u8 {
        match address % 8 {
            0 => self.ppu_ctrl,
            1 => self.ppu_mask,
            2 => self.ppu_status,
            3 => self.oam_addr,
            4 => self.oam_data,
            5 => self.ppu_scroll,
            6 => self.ppu_addr,
            7 => self.ppu_data,
            _ => unreachable!(),
        }
    }

    pub fn get_register_mut(&mut self, address: u16) -> &mut u8 {
        match address % 8 {
            0 => &mut self.ppu_ctrl,
            1 => &mut self.ppu_mask,
            2 => &mut self.ppu_status,
            3 => &mut self.oam_addr,
            4 => &mut self.oam_data,
            5 => &mut self.ppu_scroll,
            6 => &mut self.ppu_addr,
            7 => &mut self.ppu_data,
            _ => unreachable!(),
        }
    }
}
