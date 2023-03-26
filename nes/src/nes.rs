use crate::{
    cartridge::{Cartridge, EmptyCartridgeSlot},
    ppu::PPU,
};
use mos_6502::{
    cpu::CPU,
    debugging::{Debugger, ExecutionState},
    memory::Bus16,
};
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

const NES_CPU_RAM_SIZE: usize = 0x0800;
const NES_PPU_RAM_SIZE: usize = 0x0800;

pub struct NES {
    cpu: RefCell<CPU>,
    cpu_ram: Box<RefCell<[u8; NES_CPU_RAM_SIZE]>>,
    ppu: RefCell<PPU>,
    ppu_ram: Box<RefCell<[u8; NES_PPU_RAM_SIZE]>>,
    cartridge: RefCell<Box<dyn Cartridge>>,
    debugger: Option<Rc<RefCell<Debugger>>>,
}

impl NES {
    pub fn new() -> Self {
        Self {
            cpu: RefCell::new(CPU::new()),
            cpu_ram: Box::new(RefCell::new([0; NES_CPU_RAM_SIZE])),
            ppu: RefCell::new(PPU::new()),
            ppu_ram: Box::new(RefCell::new([0; NES_PPU_RAM_SIZE])),
            cartridge: RefCell::new(Box::new(EmptyCartridgeSlot)),
            debugger: None,
        }
    }

    pub fn insert_cartridge(&mut self, cartridge: Box<dyn Cartridge>) {
        self.cartridge = RefCell::new(cartridge);
        self.cpu.borrow_mut().reset(&mut self.cpu_bus())
    }

    pub fn get_pc(&self) -> u16 {
        self.cpu.borrow().pc
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.cpu.borrow_mut().pc = pc;
    }

    pub fn current_state(&self) -> ExecutionState {
        self.cpu.borrow().current_state(&mut self.cpu_bus())
    }

    pub fn jammed(&self) -> bool {
        self.cpu.borrow().jammed
    }

    pub fn enable_debugger(&mut self) {
        let debugger = Rc::new(RefCell::new(Debugger::new(None)));
        self.cpu.borrow_mut().attach_debugger(Rc::clone(&debugger));
        self.debugger = Some(debugger);
    }

    pub fn dump_backtrace(&self) {
        if let Some(debugger) = &self.debugger {
            debugger.borrow().dump_backtrace();
        }
    }

    pub fn tick(&mut self) {
        let mut cpu = self.cpu.borrow_mut();
        let cycles = cpu.execute_instruction(&mut self.cpu_bus());
    }

    fn cpu_bus(&self) -> CpuBus {
        CpuBus {
            ram: (*self.cpu_ram).borrow_mut(),
            cartridge: self.cartridge.borrow_mut(),
            ppu: self.ppu.borrow_mut(),
        }
    }
}

struct CpuBus<'a> {
    ram: RefMut<'a, [u8; NES_CPU_RAM_SIZE]>,
    ppu: RefMut<'a, PPU>,
    cartridge: RefMut<'a, Box<dyn Cartridge>>,
}

impl<'a> Bus16 for CpuBus<'a> {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0..=0x1FFF => self.ram[address as usize % NES_CPU_RAM_SIZE],
            0x2000..=0x3FFF => self.ppu.get_register(address),
            0x4000..=0x401F => {
                // TODO: APU and I/O memory mappings
                0
            }
            0x4020.. => self.cartridge.read_cpu_byte(address),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0..=0x1FFF => self.ram[address as usize % NES_CPU_RAM_SIZE] = value,
            0x2000..=0x3FFF => *self.ppu.get_register_mut(address) = value,
            0x4000..=0x401F => {
                // TODO: APU and I/O memory mappings
            }
            0x4020.. => self.cartridge.write_cpu_byte(address, value),
        }
    }
}

struct PpuBus<'a> {
    ram: RefMut<'a, Box<[u8; NES_PPU_RAM_SIZE]>>,
    ppu: RefMut<'a, PPU>,
    cartridge: RefMut<'a, dyn Cartridge>,
}

impl<'a> Bus16 for PpuBus<'a> {
    fn read_byte(&self, address: u16) -> u8 {
        // TODO
        0
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        // TODO
    }
}
