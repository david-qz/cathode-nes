use crate::{
    memory::{Ram, Rom},
    rom::{Mirroring, RomFile, RomLoadError},
};

pub trait Cartridge {
    fn cpu_read(&mut self, address: u16) -> u8;
    fn cpu_write(&mut self, address: u16, value: u8);

    fn ppu_read(&mut self, address: u16) -> u8;
    fn ppu_write(&mut self, address: u16, value: u8);
}

impl dyn Cartridge {
    pub fn load(bytes: Vec<u8>) -> Result<Box<dyn Cartridge>, RomLoadError> {
        let rom_file = RomFile::load(bytes)?;
        Ok(Self::new(rom_file))
    }

    pub fn new(rom_file: RomFile) -> Box<dyn Cartridge> {
        let mirroring = rom_file.header.mirroring();

        match rom_file.header.mapper_number() {
            0 => match rom_file.prg_rom.len() {
                16384 => Box::new(NROM::<16384>::new(rom_file, mirroring)),
                32768 => Box::new(NROM::<32768>::new(rom_file, mirroring)),
                _ => panic!("NROM rom file has unsupported prg_rom size!"),
            },
            _ => panic!("Unsupported mapper number!"),
        }
    }
}

impl Default for Box<dyn Cartridge> {
    fn default() -> Self {
        Box::new(EmptyCartridgeSlot)
    }
}

pub struct EmptyCartridgeSlot;

#[allow(unused_variables)]
impl Cartridge for EmptyCartridgeSlot {
    fn cpu_read(&mut self, address: u16) -> u8 {
        0
    }

    fn cpu_write(&mut self, address: u16, value: u8) {}

    fn ppu_read(&mut self, address: u16) -> u8 {
        0
    }

    fn ppu_write(&mut self, address: u16, value: u8) {}
}

struct NROM<const PRG_ROM_SIZE: usize> {
    vram: Ram<2048>,
    prg_rom: Rom<PRG_ROM_SIZE>,
    chr_rom: Rom<8192>,
    prg_ram: Option<Ram<2048>>,
    mirroring: Mirroring,
}

impl<const PRG_ROM_SIZE: usize> NROM<PRG_ROM_SIZE> {
    fn new(rom_file: RomFile, mirroring: Mirroring) -> Self {
        assert_eq!(rom_file.header.mapper_number(), 0);

        Self {
            vram: Ram::<2048>::new(),
            prg_rom: Rom::from_slice(&rom_file.prg_rom),
            chr_rom: Rom::from_slice(&rom_file.chr_rom),
            prg_ram: if rom_file.header.has_persistent_memory() {
                Some(Ram::new())
            } else {
                None
            },
            mirroring,
        }
    }
}

impl<const PRG_ROM_SIZE: usize> Cartridge for NROM<PRG_ROM_SIZE> {
    fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF => {
                if let Some(prg_ram) = &self.prg_ram {
                    prg_ram[address - 0x6000]
                } else {
                    0
                }
            }
            0x8000.. => self.prg_rom[(address - 0x8000)],
            _ => panic!("Cartridge: cpu bus addressed outside valid range!"),
        }
    }

    fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            0x6000..=0x7FFF => {
                if let Some(prg_ram) = &mut self.prg_ram {
                    prg_ram[(address - 0x6000)] = value
                }
            }
            0x8000.. => (),
            _ => panic!("Cartridge: cpu bus addressed outside valid range!"),
        }
    }

    fn ppu_read(&mut self, address: u16) -> u8 {
        match address {
            0..=0x1FFF => self.chr_rom[address],
            0x2000..=0x2FFF => {
                let mirrored_address = mirror_vram_address(address - 0x2000, self.mirroring);
                self.vram[mirrored_address]
            }
            _ => panic!("Cartridge: ppu bus addressed outside valid range!"),
        }
    }

    fn ppu_write(&mut self, address: u16, value: u8) {
        match address {
            0..=0x1FFF => (), // Can't write to chr_rom.
            0x2000..=0x2FFF => {
                let mirrored_address = mirror_vram_address(address - 0x2000, self.mirroring);
                self.vram[mirrored_address] = value;
            }
            _ => panic!("Cartridge: ppu bus addressed outside valid range!"),
        }
    }
}

fn mirror_vram_address(address: u16, mirroring: Mirroring) -> u16 {
    match mirroring {
        Mirroring::Horizontal => (address % 0x400) + 0x400 * (address / 0x800),
        Mirroring::Vertical => address % 0x800,
        _ => unimplemented!(),
    }
}
