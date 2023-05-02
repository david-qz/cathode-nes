use crate::{cartridge::Cartridge, frame::Frame, memory::Ram, ppu::PPU};
use mos_6502::{
    cpu::CPU,
    debugging::{Debugger, ExecutionState},
    memory::Bus16,
};
use std::{cell::RefCell, rc::Rc};

pub struct NES {
    cpu: CPU,
    ram: Ram<2048>,
    ppu: PPU,
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
            frame: Frame::new(),
            debugger: None,
        }
    }

    pub fn insert_cartridge(&mut self, cartridge: Box<dyn Cartridge>) {
        self.cartridge = cartridge;
        let mut bus = CpuBus::new(&mut self.ram, &mut self.ppu, self.cartridge.as_mut());
        self.cpu.reset(&mut bus)
    }

    pub fn get_pc(&self) -> u16 {
        self.cpu.pc
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.cpu.pc = pc;
    }

    // FIXME: This function should not require a mutable reference. See ExecutionState::new.
    pub fn current_state(&mut self) -> ExecutionState {
        let mut bus = CpuBus::new(&mut self.ram, &mut self.ppu, self.cartridge.as_mut());
        self.cpu.current_state(&mut bus)
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
            let mut bus = CpuBus::new(&mut self.ram, &mut self.ppu, self.cartridge.as_mut());
            self.cpu.execute_instruction(&mut bus)
        };

        let ppu_cycles = cpu_cycles * 3;
        self.ppu
            .tick(self.cartridge.as_mut(), &mut self.frame, ppu_cycles);

        if self.ppu.take_interrupt() {
            let mut bus = CpuBus::new(&mut self.ram, &mut self.ppu, self.cartridge.as_mut());
            self.cpu.nmi(&mut bus);
        }
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
}

struct CpuBus<'a> {
    ram: &'a mut Ram<2048>,
    ppu: &'a mut PPU,
    cartridge: &'a mut dyn Cartridge,
}

impl<'a> CpuBus<'a> {
    pub fn new(ram: &'a mut Ram<2048>, ppu: &'a mut PPU, cartridge: &'a mut dyn Cartridge) -> Self {
        CpuBus {
            ram,
            ppu,
            cartridge,
        }
    }

    fn read_page(&mut self, page: u8) -> [u8; 256] {
        let base_address = page as u16 * 256;
        let mut page_data = [0; 256];
        for i in 0..256 {
            page_data[i] = self.read_byte(base_address + i as u16);
        }
        page_data
    }
}

impl<'a> Bus16 for CpuBus<'a> {
    fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            0..=0x1FFF => self.ram[address],
            0x2000..=0x3FFF => {
                match address % 8 {
                    0 => 0, // Open bus
                    1 => 0, // Open bus
                    2 => self.ppu.read_ppu_status(),
                    3 => 0, // Open bus
                    4 => self.ppu.read_oam_data(),
                    5 => 0, // Open bus
                    6 => 0, // Open bus
                    7 => self.ppu.read_ppu_data(self.cartridge),
                    _ => unreachable!(),
                }
            }
            0x4020.. => self.cartridge.cpu_read(address),
            _ => 0, // TODO: More APU and I/O
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0..=0x1FFF => self.ram[address] = value,
            0x2000..=0x3FFF => {
                match address % 8 {
                    0 => self.ppu.write_ppu_ctrl(value),
                    1 => self.ppu.write_ppu_mask(value),
                    2 => (), // Open bus
                    3 => self.ppu.write_oam_addr(value),
                    4 => self.ppu.write_oam_data(value),
                    5 => self.ppu.write_ppu_scroll(value),
                    6 => self.ppu.write_ppu_addr(value),
                    7 => self.ppu.write_ppu_data(self.cartridge, value),
                    _ => unreachable!(),
                }
            }
            0x4014 => {
                let page_data = self.read_page(value);
                self.ppu.write_oam_dma(&page_data);
            }
            0x4020.. => {
                self.cartridge.cpu_write(address, value);
            }
            _ => (), // TODO: More APU and I/O
        }
    }
}
