pub struct Frame {
    data: Vec<(u8, u8, u8)>,
}

impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;

    pub fn new() -> Self {
        Self {
            data: vec![(0, 0, 0); Frame::WIDTH * Frame::HEIGHT],
        }
    }

    pub fn write(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        self.data[y * Frame::WIDTH + x] = rgb;
    }

    pub fn data_rgb8(&self) -> &[u8] {
        unsafe {
            let ptr: *const u8 = std::mem::transmute(self.data.as_ptr());
            let len: usize = self.data.len() * 3;
            std::slice::from_raw_parts(ptr, len)
        }
    }

    pub fn clear_with(&mut self, rgb: (u8, u8, u8)) {
        self.data.fill(rgb);
    }
}
