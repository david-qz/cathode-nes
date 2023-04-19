mod palettes;
mod registers;
mod rendering;

use self::{
    palettes::NTSC_PALETTE,
    registers::{OamAddr, PpuAddr, PpuCtrl, PpuMask, PpuScroll, PpuStatus},
    rendering::{BackgroundSlice, Sprite},
};
use crate::{
    cartridge::Cartridge,
    frame::Frame,
    memory::{PaletteRam, Ram},
};

pub struct PPU {
    ppu_ctrl: PpuCtrl,
    ppu_mask: PpuMask,
    ppu_status: PpuStatus,
    oam_addr: OamAddr,
    ppu_scroll: PpuScroll,
    ppu_addr: PpuAddr,

    oam: Ram<256>,
    palette_ram: PaletteRam,

    ppu_data_read_buffer: u8,

    x: u16,
    y: u16,
    current_background_slice: BackgroundSlice,
    nmi_interrupt: bool,
}

impl PPU {
    const SCANLINE_LENGTH: u16 = 341;
    const TOTAL_SCANLINES: u16 = 262;
    const VBLANK_START_SCANLINE: u16 = 240;
    const NMI_SCANLINE: u16 = 241;

    pub fn new() -> Self {
        Self {
            ppu_ctrl: PpuCtrl::new(),
            ppu_mask: PpuMask::new(),
            ppu_status: PpuStatus::new(),
            oam_addr: OamAddr::new(),
            ppu_scroll: PpuScroll::new(),
            ppu_addr: PpuAddr::new(),

            oam: Ram::<256>::new(),
            palette_ram: PaletteRam::new(),

            ppu_data_read_buffer: 0,

            y: 0,
            x: 0,
            current_background_slice: BackgroundSlice::new(0, 0, 0),
            nmi_interrupt: false,
        }
    }

    pub fn tick(&mut self, cartridge: &mut dyn Cartridge, frame: &mut Frame, cycles: u64) {
        for _ in 0..cycles {
            self.cycle(cartridge, frame);
        }
    }

    fn cycle(&mut self, cartridge: &mut dyn Cartridge, frame: &mut Frame) {
        if self.x == 0 && self.y == 0 {
            frame.clear_with((255, 0, 255));
        }

        if self.x >= 257 && self.x <= 320 {
            self.oam_addr.reset_latch();
        }

        if self.x < 256 && self.y < 240 {
            if self.x % 8 == 0 {
                self.fetch_background_slice(cartridge);
            }

            if self.ppu_mask.render_background() {
                let color_index = self.current_background_slice.color(self.x % 8);
                let palette_index = self.palette_ram[color_index];
                let color = NTSC_PALETTE[palette_index as usize];
                frame.write(self.x as usize, self.y as usize, color);
            }

            if self.ppu_mask.render_sprites() {
                let sprite_size = self.ppu_ctrl.sprite_size();
                for (i, sprite) in self
                    .oam
                    .as_slice()
                    .chunks_exact(4)
                    .map(Sprite::new)
                    .enumerate()
                {
                    if !sprite.contains_point(self.x as usize, self.y as usize, sprite_size) {
                        continue;
                    }
                    frame.write(self.x as usize, self.y as usize, (0, 0, 255));
                    break;
                }
            }
        }

        self.x += 1;
        if self.x >= PPU::SCANLINE_LENGTH {
            self.x = 0;
            self.y += 1;

            if self.y == PPU::NMI_SCANLINE {
                if self.ppu_ctrl.nmi_enabled() {
                    self.nmi_interrupt = true;
                }
                self.ppu_status.set_vblank_started(true);
                self.ppu_status.set_sprite_zero_hit(false);
            }

            if self.y >= PPU::TOTAL_SCANLINES {
                self.y = 0;
                self.nmi_interrupt = false;
                self.ppu_status.set_vblank_started(false);
                self.ppu_status.set_sprite_zero_hit(false);
            }
        }
    }

    fn fetch_background_slice(&mut self, cartridge: &mut dyn Cartridge) {
        let tile_x = (self.x / 8) as u16;
        let tile_y = (self.y / 8) as u16;
        let fine_y = (self.y % 8) as u16;

        let nametable_address = self.ppu_ctrl.nametable_base_address();
        let nametable_offset = tile_y * 32 + tile_x;
        let nametable_entry = cartridge.ppu_read(nametable_address + nametable_offset);

        let pattern_table_address = self.ppu_ctrl.background_pattern_table_address();
        let pattern_slice_offset = (nametable_entry as u16) << 4 | fine_y;
        let lower_bit_plane = cartridge.ppu_read(pattern_table_address + pattern_slice_offset);
        let upper_bit_plane = cartridge.ppu_read(pattern_table_address + pattern_slice_offset + 8);

        let attribute_table_address = nametable_address + 0x3C0;
        let attribute_table_offset = (tile_y / 4) * 8 + (tile_x / 4);
        let attribute_byte = cartridge.ppu_read(attribute_table_address + attribute_table_offset);

        let tile_quadrant = ((tile_y / 2) % 2) << 1 | (tile_x / 2) % 2;
        let palette_section = (attribute_byte >> (tile_quadrant * 2)) & 0x03;

        self.current_background_slice =
            BackgroundSlice::new(lower_bit_plane, upper_bit_plane, palette_section);
    }

    pub fn take_interrupt(&mut self) -> bool {
        let interrupt = self.nmi_interrupt;
        self.nmi_interrupt = false;
        interrupt
    }

    pub fn write_ppu_ctrl(&mut self, value: u8) {
        self.ppu_ctrl.write(value);
    }

    pub fn write_ppu_mask(&mut self, value: u8) {
        self.ppu_mask.write(value);
    }

    pub fn read_ppu_status(&mut self) -> u8 {
        let value = self.ppu_status.read();
        self.ppu_addr.reset_latch();
        self.ppu_scroll.reset_latch();
        value
    }

    pub fn write_ppu_scroll(&mut self, value: u8) {
        self.ppu_scroll.write(value);
    }

    pub fn write_ppu_addr(&mut self, value: u8) {
        self.ppu_addr.write(value);
    }

    pub fn write_ppu_data(&mut self, cartridge: &mut dyn Cartridge, value: u8) {
        let address: u16 = self.ppu_addr.get();
        let increment = self.ppu_ctrl.vram_address_increment();
        self.ppu_addr.increment_address(increment);

        match address {
            0..=0x3EFF => cartridge.ppu_write(address, value),
            0x3F00..=0x3FFF => self.palette_ram[address - 0x3F00] = value,
            _ => unreachable!(),
        }
    }

    pub fn read_ppu_data(&mut self, cartridge: &mut dyn Cartridge) -> u8 {
        let address: u16 = self.ppu_addr.get();
        let increment = self.ppu_ctrl.vram_address_increment();
        self.ppu_addr.increment_address(increment);

        match address {
            0..=0x3EFF => {
                let buffered_read = cartridge.ppu_read(address);
                std::mem::replace(&mut self.ppu_data_read_buffer, buffered_read)
            }
            0x3F00..=0x3FFF => self.palette_ram[address - 0x3F00],
            _ => unreachable!(),
        }
    }

    pub fn write_oam_addr(&mut self, value: u8) {
        self.oam_addr.write(value);
    }

    pub fn write_oam_data(&mut self, value: u8) {
        self.oam[self.oam_addr.get() as u16] = value;
        self.oam_addr.increment();
    }

    pub fn read_oam_data(&mut self) -> u8 {
        let oam_data = self.oam[self.oam_addr.get() as u16];
        if !self.in_vblank() {
            self.oam_addr.increment();
        }
        oam_data
    }

    pub fn write_oam_dma(&mut self, oam_data: &[u8; 256]) {
        self.oam.copy_from_slice(oam_data)
    }

    pub fn in_vblank(&self) -> bool {
        self.y >= PPU::VBLANK_START_SCANLINE
    }
}
