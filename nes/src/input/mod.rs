pub(crate) struct ControllerPort {
    read_buffer: Vec<u8>,
    index: u8,
    overrun_default: u8,
    incoming_state: Option<(Vec<u8>, u8)>,
}

impl ControllerPort {
    pub fn poll(&mut self) {
        if let Some((read_buffer, overrun_default)) = self.incoming_state.take() {
            self.read_buffer = read_buffer;
            self.overrun_default = overrun_default;
        }
        self.index = 0;
    }

    fn current_byte(&self) -> u8 {
        *self
            .read_buffer
            .get(self.index as usize)
            .unwrap_or(&self.overrun_default)
    }

    pub fn peek(&self) -> u8 {
        self.current_byte()
    }

    pub fn read(&mut self) -> u8 {
        let byte = self.current_byte();
        self.index += 1;
        byte
    }

    pub fn update<S: ControllerState>(&mut self, state: &S) {
        self.incoming_state = Some((state.read_buffer(), state.overrun_default()));
    }
}

impl Default for ControllerPort {
    fn default() -> Self {
        Self {
            read_buffer: vec![0, 0, 0, 0, 0, 0, 0, 0],
            index: 0,
            overrun_default: 1,
            incoming_state: None,
        }
    }
}

pub trait ControllerState {
    fn read_buffer(&self) -> Vec<u8>;
    fn overrun_default(&self) -> u8;
}

mod standard_controller;
pub use standard_controller::StandardController;
