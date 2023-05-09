use super::ControllerState;

#[derive(Clone, Copy, Default)]
pub struct StandardController {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl ControllerState for StandardController {
    fn read_buffer(&self) -> Vec<u8> {
        vec![
            self.a as u8,
            self.b as u8,
            self.select as u8,
            self.start as u8,
            self.up as u8,
            self.down as u8,
            self.left as u8,
            self.right as u8,
        ]
    }

    fn overrun_default(&self) -> u8 {
        1
    }
}
