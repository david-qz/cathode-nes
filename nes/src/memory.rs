use std::ops::{Index, IndexMut};

pub struct Ram<const SIZE: usize> {
    bytes: [u8; SIZE],
}

impl<const SIZE: usize> Ram<SIZE> {
    pub fn new() -> Self {
        Self { bytes: [0; SIZE] }
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
