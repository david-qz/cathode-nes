use crate::rom::{RomFile, RomLoadError};

pub trait Cartridge {
    fn read_cpu_byte(&self, address: u16) -> u8;
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
            0 => Box::new(NROM::new(rom_file)),
            _ => panic!("Unsupported mapper number!"),
        }
    }
}

pub struct EmptyCartridgeSlot;

#[allow(unused_variables)]
impl Cartridge for EmptyCartridgeSlot {
    fn read_cpu_byte(&self, address: u16) -> u8 {
        0
    }

    fn write_cpu_byte(&mut self, address: u16, value: u8) {}
}

struct NROM {
    prg_rom: Box<[u8]>,
    chr_rom: Box<[u8]>,
    prg_ram: Option<Box<[u8; 0x2000]>>,
}

impl NROM {
    fn new(rom_file: RomFile) -> Self {
        assert_eq!(rom_file.header.mapper_number(), 0);

        Self {
            prg_rom: rom_file.prg_rom,
            chr_rom: rom_file.chr_rom,
            prg_ram: if rom_file.header.has_persistent_memory() {
                Some(Box::new([0; 0x2000]))
            } else {
                None
            },
        }
    }
}

impl Cartridge for NROM {
    fn read_cpu_byte(&self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF => {
                if let Some(prg_ram) = &self.prg_ram {
                    prg_ram[(address as usize - 0x6000) % 0x2000]
                } else {
                    0
                }
            }
            0x8000.. => self.prg_rom[(address as usize - 0x8000) % self.prg_rom.len()],
            _ => panic!("NROM: cartridge addressed outside valid range!"),
        }
    }

    fn write_cpu_byte(&mut self, address: u16, value: u8) {
        match address {
            0x6000..=0x7FFF => {
                if let Some(prg_ram) = &mut self.prg_ram {
                    prg_ram[(address as usize - 0x6000) % 0x2000] = value
                }
            }
            0x8000.. => self.prg_rom[(address as usize - 0x8000) % self.prg_rom.len()] = value,
            _ => panic!("NROM: cartridge addressed outside valid range!"),
        }
    }
}
