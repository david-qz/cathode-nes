#[derive(Clone, Copy)]
pub enum SpriteSize {
    EightByEight,
    EightBySixteen,
}

pub struct Sprite<'a> {
    bytes: &'a [u8],
}

impl<'a> Sprite<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        assert_eq!(bytes.len(), 4);
        Self { bytes }
    }

    pub fn x_pos(&self) -> u8 {
        self.bytes[3]
    }

    pub fn y_pos(&self) -> u8 {
        self.bytes[0]
    }

    pub fn tile_index(&self, size: SpriteSize) -> u8 {
        match size {
            SpriteSize::EightByEight => self.bytes[1],
            SpriteSize::EightBySixteen => self.bytes[1] & 0xFE,
        }
    }

    pub fn bank_for_eight_by_sixteen_sprite(&self) -> u16 {
        match self.bytes[1] & 0x01 != 0 {
            false => 0x0000,
            true => 0x1000,
        }
    }

    pub fn palette_section(&self) -> u8 {
        self.bytes[2] & 0x03 + 4
    }

    pub fn above_background(&self) -> bool {
        self.bytes[2] & 0x20 != 0
    }

    pub fn flipped_horizontally(&self) -> bool {
        self.bytes[2] & 0x40 != 0
    }

    pub fn flipped_vertically(&self) -> bool {
        self.bytes[2] & 0x80 != 0
    }

    pub fn contains_point(&self, x: usize, y: usize, size: SpriteSize) -> bool {
        let width = 8;
        let height = match size {
            SpriteSize::EightByEight => 8,
            SpriteSize::EightBySixteen => 16,
        };
        let x_pos = self.x_pos() as usize;
        let y_pos = self.y_pos() as usize;

        if x >= x_pos && x < x_pos + width && y >= y_pos && y < y_pos + height {
            true
        } else {
            false
        }
    }
}

pub struct TileSlice {
    lower_bit_plane: u8,
    upper_bit_plane: u8,
}

impl TileSlice {
    pub fn new(lower_bit_plane: u8, upper_bit_plane: u8) -> Self {
        Self {
            lower_bit_plane,
            upper_bit_plane,
        }
    }

    pub fn pattern_color(&self, pixel: u16) -> u16 {
        let lower_bit_set = self.lower_bit_plane & (1 << (7 - pixel)) != 0;
        let upper_bit_set = self.upper_bit_plane & (1 << (7 - pixel)) != 0;

        (upper_bit_set as u16) << 1 | (lower_bit_set as u16)
    }
}

pub struct BackgroundSlice {
    tile_slice: TileSlice,
    palette_section: u8,
}

impl BackgroundSlice {
    pub fn new(lower_bit_plane: u8, upper_bit_plane: u8, palette_section: u8) -> Self {
        let tile_slice = TileSlice::new(lower_bit_plane, upper_bit_plane);
        Self {
            tile_slice,
            palette_section,
        }
    }

    pub fn color(&self, pixel: u16) -> u16 {
        let pattern_color = self.tile_slice.pattern_color(pixel);

        if pattern_color != 0 {
            (self.palette_section as u16) << 2 | pattern_color
        } else {
            0
        }
    }
}
