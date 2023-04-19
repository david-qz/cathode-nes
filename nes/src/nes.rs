use crate::{cartridge::Cartridge, frame::Frame, memory::Ram, ppu::PPU};
use mos_6502::{
    cpu::CPU,
    debugging::{Debugger, ExecutionState},
    memory::Bus16,
};
use std::{cell::RefCell, rc::Rc};

pub struct NES {
    cpu: CPU,
    ram: Rc<RefCell<Ram<2048>>>,
    ppu: Rc<RefCell<PPU>>,
    cartridge: Rc<RefCell<Box<dyn Cartridge>>>,
    frame: Frame,
    debugger: Option<Rc<RefCell<Debugger>>>,
}

impl NES {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            ram: Rc::new(RefCell::new(Ram::<2048>::new())),
            ppu: Rc::new(RefCell::new(PPU::new())),
            cartridge: Rc::new(RefCell::new(Default::default())),
            frame: Frame::new(),
            debugger: None,
        }
    }

    pub fn insert_cartridge(&mut self, cartridge: Box<dyn Cartridge>) {
        self.cartridge = Rc::new(RefCell::new(cartridge));
        self.cpu.reset(&mut self.cpu_bus())
    }

    pub fn get_pc(&self) -> u16 {
        self.cpu.pc
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.cpu.pc = pc;
    }

    pub fn current_state(&self) -> ExecutionState {
        self.cpu.current_state(&mut self.cpu_bus())
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
        self.ppu.borrow().in_vblank()
    }

    pub fn borrow_frame(&self) -> &Frame {
        &self.frame
    }

    pub fn tick(&mut self) {
        let cpu_cycles = self.cpu.execute_instruction(&mut self.cpu_bus());

        let nmi_interrupt = {
            let mut cartridge = self.cartridge.borrow_mut();
            let mut ppu = self.ppu.borrow_mut();
            ppu.tick(cartridge.as_mut(), &mut self.frame, cpu_cycles * 3);
            ppu.take_interrupt()
        };

        if nmi_interrupt {
            self.cpu.nmi(&mut self.cpu_bus());
        }
    }

    fn cpu_bus(&self) -> CpuBus {
        CpuBus {
            ram: Rc::clone(&self.ram),
            cartridge: Rc::clone(&self.cartridge),
            ppu: Rc::clone(&self.ppu),
        }
    }
}

struct CpuBus {
    ram: Rc<RefCell<Ram<2048>>>,
    ppu: Rc<RefCell<PPU>>,
    cartridge: Rc<RefCell<Box<dyn Cartridge>>>,
}

impl CpuBus {
    fn read_page(&mut self, page: u8) -> [u8; 256] {
        let base_address = page as u16 * 256;
        let mut page_data = [0; 256];
        for i in 0..256 {
            page_data[i] = self.read_byte(base_address + i as u16);
        }
        page_data
    }
}

impl Bus16 for CpuBus {
    fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            0..=0x1FFF => {
                let ram = self.ram.borrow();
                ram[address]
            }
            0x2000..=0x3FFF => {
                let mut ppu = self.ppu.borrow_mut();
                let mut cartridge = self.cartridge.borrow_mut();
                match address % 8 {
                    0 => 0, // Open bus
                    1 => 0, // Open bus
                    2 => ppu.read_ppu_status(),
                    3 => 0, // Open bus
                    4 => ppu.read_oam_data(),
                    5 => 0, // Open bus
                    6 => 0, // Open bus
                    7 => ppu.read_ppu_data(cartridge.as_mut()),
                    _ => unreachable!(),
                }
            }
            0x4020.. => {
                let mut cartridge = self.cartridge.borrow_mut();
                cartridge.cpu_read(address)
            }
            _ => 0, // TODO: More APU and I/O
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0..=0x1FFF => {
                let mut ram = self.ram.borrow_mut();
                ram[address] = value
            }
            0x2000..=0x3FFF => {
                let mut ppu = self.ppu.borrow_mut();
                let mut cartridge = self.cartridge.borrow_mut();
                match address % 8 {
                    0 => ppu.write_ppu_ctrl(value),
                    1 => ppu.write_ppu_mask(value),
                    2 => (), // Open bus
                    3 => ppu.write_oam_addr(value),
                    4 => ppu.write_oam_data(value),
                    5 => ppu.write_ppu_scroll(value),
                    6 => ppu.write_ppu_addr(value),
                    7 => ppu.write_ppu_data(cartridge.as_mut(), value),
                    _ => unreachable!(),
                }
            }
            0x4014 => {
                let page_data = self.read_page(value);
                let mut ppu = self.ppu.borrow_mut();
                ppu.write_oam_dma(&page_data);
            }
            0x4020.. => {
                let mut cartridge = self.cartridge.borrow_mut();
                cartridge.cpu_write(address, value);
            }
            _ => (), // TODO: More APU and I/O
        }
    }
}
