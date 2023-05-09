use mos_6502::memory::Bus16;

use crate::{
    cartridge::Cartridge,
    input::ControllerPort,
    memory::Ram,
    ppu::{PpuRegister, PPU},
};

enum MappedAddress {
    Ram(u16),
    Ppu(PpuRegister),
    OamDma,
    ControllerPortA,
    ControllerPortB,
    Cartridge(u16),
    Unimplemented,
}

fn map_address(address: u16) -> MappedAddress {
    match address {
        0..=0x1FFF => MappedAddress::Ram(address),
        0x2000..=0x3FFF => match address % 8 {
            0 => MappedAddress::Ppu(PpuRegister::PpuCtrl),
            1 => MappedAddress::Ppu(PpuRegister::PpuMask),
            2 => MappedAddress::Ppu(PpuRegister::PpuStatus),
            3 => MappedAddress::Ppu(PpuRegister::OamAddr),
            4 => MappedAddress::Ppu(PpuRegister::OamData),
            5 => MappedAddress::Ppu(PpuRegister::PpuScroll),
            6 => MappedAddress::Ppu(PpuRegister::PpuAddr),
            7 => MappedAddress::Ppu(PpuRegister::PpuData),
            _ => unreachable!(),
        },
        0x4014 => MappedAddress::OamDma,
        0x4016 => MappedAddress::ControllerPortA,
        0x4017 => MappedAddress::ControllerPortB,
        0x4020.. => MappedAddress::Cartridge(address),
        _ => MappedAddress::Unimplemented, // TODO: More APU and I/O
    }
}

fn read_page<B>(bus: &mut B, page: u8) -> [u8; 256]
where
    B: Bus16,
{
    let base_address = page as u16 * 256;
    let mut page_data = [0; 256];
    for i in 0..256 {
        page_data[i] = bus.read_byte(base_address + i as u16);
    }
    page_data
}

pub(crate) struct CpuBus<'a> {
    pub ram: &'a mut Ram<2048>,
    pub ppu: &'a mut PPU,
    pub port_a: &'a mut ControllerPort,
    pub port_b: &'a mut ControllerPort,
    pub cartridge: &'a mut dyn Cartridge,
}

impl<'a> Bus16 for CpuBus<'a> {
    fn peek_byte(&self, address: u16) -> u8 {
        match map_address(address) {
            MappedAddress::Ram(address) => self.ram[address],
            MappedAddress::Ppu(register) => self.ppu.peek_register(register),
            MappedAddress::OamDma => 0, // Open bus
            MappedAddress::ControllerPortA => self.port_a.peek(),
            MappedAddress::ControllerPortB => self.port_b.peek(),
            MappedAddress::Cartridge(address) => self.cartridge.cpu_peek(address),
            MappedAddress::Unimplemented => 0,
        }
    }

    fn read_byte(&mut self, address: u16) -> u8 {
        match map_address(address) {
            MappedAddress::Ram(address) => self.ram[address],
            MappedAddress::Ppu(register) => self.ppu.read_register(self.cartridge, register),
            MappedAddress::OamDma => 0, // Open bus
            MappedAddress::ControllerPortA => self.port_a.read(),
            MappedAddress::ControllerPortB => self.port_b.read(),
            MappedAddress::Cartridge(address) => self.cartridge.cpu_read(address),
            MappedAddress::Unimplemented => 0,
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match map_address(address) {
            MappedAddress::Ram(address) => self.ram[address] = value,
            MappedAddress::Ppu(register) => {
                self.ppu.write_register(self.cartridge, register, value)
            }
            MappedAddress::OamDma => {
                let page_data = read_page(self, value);
                self.ppu.oam_dma(&page_data);
            }
            MappedAddress::ControllerPortA => {
                if value & 0x01 != 0 {
                    self.port_a.poll();
                    self.port_b.poll();
                }
            }
            MappedAddress::ControllerPortB => (),
            MappedAddress::Cartridge(address) => self.cartridge.cpu_write(address, value),
            MappedAddress::Unimplemented => (),
        }
    }
}

pub(crate) struct FrozenCpuBus<'a> {
    pub ram: &'a Ram<2048>,
    pub ppu: &'a PPU,
    pub port_a: &'a ControllerPort,
    pub port_b: &'a ControllerPort,
    pub cartridge: &'a dyn Cartridge,
}

impl<'a> Bus16 for FrozenCpuBus<'a> {
    fn peek_byte(&self, address: u16) -> u8 {
        match map_address(address) {
            MappedAddress::Ram(address) => self.ram[address],
            MappedAddress::Ppu(register) => self.ppu.peek_register(register),
            MappedAddress::OamDma => 0, // Open bus
            MappedAddress::ControllerPortA => self.port_a.peek(),
            MappedAddress::ControllerPortB => self.port_b.peek(),
            MappedAddress::Cartridge(address) => self.cartridge.cpu_peek(address),
            MappedAddress::Unimplemented => 0,
        }
    }

    fn read_byte(&mut self, _address: u16) -> u8 {
        panic!("Attempt to read from frozen bus.");
    }

    fn write_byte(&mut self, _address: u16, _value: u8) {
        panic!("Attempt to write to frozen bus.");
    }
}
