pub struct Frame {
    data: Vec<u8>,
}

impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;
    pub const BYTES_PER_PIXEL: usize = 3;

    pub fn new() -> Self {
        Self {
            data: vec![0; Frame::WIDTH * Frame::HEIGHT * Frame::BYTES_PER_PIXEL],
        }
    }

    pub fn write(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base_idx = (y * Frame::WIDTH + x) * Frame::BYTES_PER_PIXEL;
        self.data[base_idx + 0] = rgb.0;
        self.data[base_idx + 1] = rgb.1;
        self.data[base_idx + 2] = rgb.2;
    }

    pub fn data_rgb8(&self) -> &[u8] {
        &self.data
    }

    pub fn clear_with(&mut self, rgb: (u8, u8, u8)) {
        for base_idx in (0..self.data.len()).step_by(3) {
            self.data[base_idx + 0] = rgb.0;
            self.data[base_idx + 1] = rgb.1;
            self.data[base_idx + 2] = rgb.2;
        }
    }
}
