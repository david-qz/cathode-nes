use std::ops::{Index, IndexMut};

pub struct Ram<const SIZE: usize> {
    bytes: [u8; SIZE],
}

impl<const SIZE: usize> Ram<SIZE> {
    pub fn new() -> Self {
        Self { bytes: [0; SIZE] }
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        self.bytes.copy_from_slice(slice)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
}

impl<const SIZE: usize> Index<u16> for Ram<SIZE> {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.bytes[(index as usize) % SIZE]
    }
}

impl<const SIZE: usize> IndexMut<u16> for Ram<SIZE> {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.bytes[(index as usize) % SIZE]
    }
}

pub struct Rom<const SIZE: usize> {
    bytes: [u8; SIZE],
}

impl<const SIZE: usize> Rom<SIZE> {
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            bytes: slice.try_into().unwrap(),
        }
    }
}

impl<const SIZE: usize> Index<u16> for Rom<SIZE> {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.bytes[(index as usize) % SIZE]
    }
}

pub struct PaletteRam {
    bytes: [u8; Self::SIZE],
}

impl PaletteRam {
    const SIZE: usize = 32;

    pub fn new() -> Self {
        Self {
            bytes: [0; Self::SIZE],
        }
    }

    fn mirror(&self, address: u16) -> u16 {
        if address >= 16 && address % 4 == 0 {
            address - 16
        } else {
            address
        }
    }
}

impl Index<u16> for PaletteRam {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.bytes[self.mirror(index) as usize % Self::SIZE]
    }
}

impl IndexMut<u16> for PaletteRam {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.bytes[self.mirror(index) as usize % Self::SIZE]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_ram_mirroring() {
        let mut palette_ram = PaletteRam::new();
        for i in 0..32 {
            palette_ram[i] = i as u8;
        }

        for i in 0..32 {
            if i == 0x00 || i == 0x04 || i == 0x08 || i == 0x0C {
                assert_eq!(palette_ram[i], (i + 16) as u8);
            } else {
                assert_eq!(palette_ram[i], i as u8);
            }
        }
    }
}
