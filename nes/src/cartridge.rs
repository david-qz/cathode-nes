use crate::{
    memory::{Ram, Rom},
    rom::{RomFile, RomLoadError},
};

pub trait Cartridge {
    fn read_cpu_byte(&mut self, address: u16) -> u8;
    fn write_cpu_byte(&mut self, address: u16, value: u8);

    // fn read_ppu_byte(&self, address: u16) -> u8;
    // fn write_ppu_byte(&mut self, address: u16, value: u8);
}

impl dyn Cartridge {
    pub fn load(bytes: Vec<u8>) -> Result<Box<dyn Cartridge>, RomLoadError> {
        let rom_file = RomFile::load(bytes)?;
        Ok(Self::new(rom_file))
    }

    pub fn new(rom_file: RomFile) -> Box<dyn Cartridge> {
        match rom_file.header.mapper_number() {
            0 => match rom_file.prg_rom.len() {
                16384 => Box::new(NROM::<16384>::new(rom_file)),
                32768 => Box::new(NROM::<32768>::new(rom_file)),
                _ => panic!("NROM rom file has unsupported prg_rom size!"),
            },
            _ => panic!("Unsupported mapper number!"),
        }
    }
}

pub struct EmptyCartridgeSlot;

#[allow(unused_variables)]
impl Cartridge for EmptyCartridgeSlot {
    fn read_cpu_byte(&mut self, address: u16) -> u8 {
        0
    }

    fn write_cpu_byte(&mut self, address: u16, value: u8) {}
}

struct NROM<const PRG_ROM_SIZE: usize> {
    prg_rom: Rom<PRG_ROM_SIZE>,
    chr_rom: Rom<8192>,
    prg_ram: Option<Ram<2048>>,
}

impl<const PRG_ROM_SIZE: usize> NROM<PRG_ROM_SIZE> {
    fn new(rom_file: RomFile) -> Self {
        assert_eq!(rom_file.header.mapper_number(), 0);

        Self {
            prg_rom: Rom::from_slice(&rom_file.prg_rom),
            chr_rom: Rom::from_slice(&rom_file.chr_rom),
            prg_ram: if rom_file.header.has_persistent_memory() {
                Some(Ram::new())
            } else {
                None
            },
        }
    }
}

impl<const PRG_ROM_SIZE: usize> Cartridge for NROM<PRG_ROM_SIZE> {
    fn read_cpu_byte(&mut self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF => {
                if let Some(prg_ram) = &self.prg_ram {
                    prg_ram[address - 0x6000]
                } else {
                    0
                }
            }
            0x8000.. => self.prg_rom[(address - 0x8000)],
            _ => panic!("NROM: cartridge addressed outside valid range!"),
        }
    }

    fn write_cpu_byte(&mut self, address: u16, value: u8) {
        match address {
            0x6000..=0x7FFF => {
                if let Some(prg_ram) = &mut self.prg_ram {
                    prg_ram[(address - 0x6000)] = value
                }
            }
            0x8000.. => (),
            _ => panic!("NROM: cartridge addressed outside valid range!"),
        }
    }
}
