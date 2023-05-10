use crate::{
    cartridge::Cartridge,
    cpu_bus::{CpuBus, FrozenCpuBus},
    frame::Frame,
    input::{ControllerPort, ControllerState},
    memory::Ram,
    ppu::PPU,
};
use macros::{cpu_bus, frozen_cpu_bus};
use mos_6502::{
    cpu::CPU,
    debugging::{Debugger, ExecutionState},
};
use std::{cell::RefCell, rc::Rc};

pub struct NES {
    cpu: CPU,
    ram: Ram<2048>,
    ppu: PPU,
    port_a: ControllerPort,
    port_b: ControllerPort,
    cartridge: Box<dyn Cartridge>,
    frame: Frame,
    debugger: Option<Rc<RefCell<Debugger>>>,
}

impl NES {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            ram: Ram::<2048>::new(),
            ppu: PPU::new(),
            cartridge: Default::default(),
            port_a: Default::default(),
            port_b: Default::default(),
            frame: Frame::new(),
            debugger: None,
        }
    }

    pub fn insert_cartridge(&mut self, cartridge: Box<dyn Cartridge>) {
        self.cartridge = cartridge;
        let mut bus = cpu_bus!(self);
        self.cpu.reset(&mut bus)
    }

    pub fn get_pc(&self) -> u16 {
        self.cpu.pc
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.cpu.pc = pc;
    }

    pub fn current_state(&self) -> ExecutionState {
        let bus = frozen_cpu_bus!(self);
        self.cpu.current_state(&bus)
    }

    pub fn jammed(&self) -> bool {
        self.cpu.jammed
    }

    pub fn enable_debugger(&mut self) {
        let debugger = Rc::new(RefCell::new(Debugger::new()));
        self.cpu.attach_debugger(Rc::clone(&debugger));
        self.debugger = Some(debugger);
    }

    pub fn dump_backtrace(&self) {
        if let Some(debugger) = &self.debugger {
            debugger.borrow().dump_backtrace();
        }
    }

    pub fn in_vblank(&self) -> bool {
        self.ppu.in_vblank()
    }

    pub fn borrow_frame(&self) -> &Frame {
        &self.frame
    }

    pub fn tick(&mut self) {
        let cpu_cycles = {
            let mut bus = cpu_bus!(self);
            self.cpu.execute_instruction(&mut bus)
        };

        let ppu_cycles = cpu_cycles * 3;
        self.ppu
            .tick(self.cartridge.as_mut(), &mut self.frame, ppu_cycles);

        self.cpu.nmi = self.ppu.interrupt;
    }

    pub fn advance_to_next_frame(&mut self) {
        let mut last_in_vblank = self.in_vblank();
        while !self.jammed() {
            self.tick();
            let in_vblank = self.in_vblank();
            if !last_in_vblank && in_vblank {
                return;
            }
            last_in_vblank = in_vblank;
        }
    }

    pub fn update_controller_port_a<S: ControllerState>(&mut self, state: &S) {
        self.port_a.update(state);
    }

    pub fn update_controller_port_b<S: ControllerState>(&mut self, state: &S) {
        self.port_b.update(state);
    }
}

mod macros {
    macro_rules! cpu_bus {
        ($nes:expr) => {
            CpuBus {
                ram: &mut $nes.ram,
                ppu: &mut $nes.ppu,
                port_a: &mut $nes.port_a,
                port_b: &mut $nes.port_b,
                cartridge: $nes.cartridge.as_mut(),
            }
        };
    }

    macro_rules! frozen_cpu_bus {
        ($nes:expr) => {
            FrozenCpuBus {
                ram: &$nes.ram,
                ppu: &$nes.ppu,
                port_a: &$nes.port_a,
                port_b: &$nes.port_b,
                cartridge: $nes.cartridge.as_ref(),
            }
        };
    }

    pub(super) use cpu_bus;
    pub(super) use frozen_cpu_bus;
}
