use crate::{
    cartridge::{Cartridge, EmptyCartridgeSlot},
    memory::Ram,
    ppu::PPU,
};
use mos_6502::{
    cpu::CPU,
    debugging::{Debugger, ExecutionState},
    memory::Bus16,
};
use std::{cell::RefCell, rc::Rc};

const NES_CPU_RAM_SIZE: usize = 2048;
const NES_PPU_RAM_SIZE: usize = 2048;

pub struct NES {
    cpu: CPU,
    cpu_ram: Rc<RefCell<Ram<NES_CPU_RAM_SIZE>>>,
    ppu: Rc<RefCell<PPU>>,
    ppu_ram: Rc<RefCell<Ram<NES_PPU_RAM_SIZE>>>,
    cartridge: Rc<RefCell<Box<dyn Cartridge>>>,
    debugger: Option<Rc<RefCell<Debugger>>>,
}

impl NES {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
            cpu_ram: Rc::new(RefCell::new(Ram::<NES_CPU_RAM_SIZE>::new())),
            ppu: Rc::new(RefCell::new(PPU::new())),
            ppu_ram: Rc::new(RefCell::new(Ram::<NES_PPU_RAM_SIZE>::new())),
            cartridge: Rc::new(RefCell::new(Box::new(EmptyCartridgeSlot))),
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

    pub fn tick(&mut self) {
        let cpu_cycles = self.cpu.execute_instruction(&mut self.cpu_bus());

        let mut ppu = self.ppu.borrow_mut();
        ppu.tick(&mut self.ppu_bus(), cpu_cycles * 3);
    }

    fn cpu_bus(&self) -> CpuBus {
        CpuBus {
            ram: Rc::clone(&self.cpu_ram),
            cartridge: Rc::clone(&self.cartridge),
            ppu: Rc::clone(&self.ppu),
        }
    }

    fn ppu_bus(&self) -> PpuBus {
        PpuBus {
            ram: Rc::clone(&self.ppu_ram),
            cartridge: Rc::clone(&self.cartridge),
        }
    }
}

struct CpuBus {
    ram: Rc<RefCell<Ram<NES_CPU_RAM_SIZE>>>,
    ppu: Rc<RefCell<PPU>>,
    cartridge: Rc<RefCell<Box<dyn Cartridge>>>,
}

impl Bus16 for CpuBus {
    fn read_byte(&mut self, address: u16) -> u8 {
        let ram = self.ram.borrow();
        let ppu = self.ppu.borrow_mut();
        let mut cartridge = self.cartridge.borrow_mut();

        match address {
            0..=0x1FFF => ram[address],
            0x2000..=0x3FFF => ppu.get_register(address),
            0x4000..=0x401F => {
                // TODO: APU and I/O memory mappings
                0
            }
            0x4020.. => cartridge.read_cpu_byte(address),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        let mut ram = self.ram.borrow_mut();
        let mut ppu = self.ppu.borrow_mut();
        let mut cartridge = self.cartridge.borrow_mut();

        match address {
            0..=0x1FFF => ram[address] = value,
            0x2000..=0x3FFF => *ppu.get_register_mut(address) = value,
            0x4000..=0x401F => {
                // TODO: APU and I/O memory mappings
            }
            0x4020.. => cartridge.write_cpu_byte(address, value),
        }
    }
}

struct PpuBus {
    ram: Rc<RefCell<Ram<NES_PPU_RAM_SIZE>>>,
    cartridge: Rc<RefCell<Box<dyn Cartridge>>>,
}

impl Bus16 for PpuBus {
    fn read_byte(&mut self, address: u16) -> u8 {
        // TODO
        0
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        // TODO
    }
}
