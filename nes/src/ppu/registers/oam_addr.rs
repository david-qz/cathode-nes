pub struct OamAddr {
    value: u8,
}

impl OamAddr {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn get(&self) -> u8 {
        self.value
    }

    pub fn write(&mut self, value: u8) {
        self.value = value;
    }

    pub fn increment(&mut self) {
        self.value = self.value.wrapping_add(1);
    }

    pub fn reset_latch(&mut self) {
        self.value = 0;
    }
}
