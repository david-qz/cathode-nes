const INES_HEADER_LENGTH: usize = 16;
const INES_TRAINER_LENGTH: usize = 512;
const INES_PRG_ROM_UNITS: usize = 16384;
const INES_CHR_ROM_UNITS: usize = 8192;

pub struct INesHeader {
    bytes: [u8; INES_HEADER_LENGTH],
}

impl INesHeader {
    fn new(bytes: &[u8; 16]) -> Self {
        Self {
            bytes: bytes.clone(),
        }
    }

    fn prg_rom_size(&self) -> usize {
        self.bytes[4] as usize * INES_PRG_ROM_UNITS
    }

    fn chr_rom_size(&self) -> usize {
        self.bytes[5] as usize * INES_CHR_ROM_UNITS
    }

    pub fn mirroring(&self) -> Mirroring {
        if self.bytes[6] & (1 << 3) != 0 {
            Mirroring::FourScreen
        } else if self.bytes[6] & (1 << 0) != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        }
    }

    pub fn has_persistent_memory(&self) -> bool {
        self.bytes[6] & (1 << 1) != 0
    }

    fn has_trainer(&self) -> bool {
        self.bytes[6] & (1 << 2) != 0
    }

    pub fn mapper_number(&self) -> u16 {
        (self.bytes[7] as u16) & 0xF0 | ((self.bytes[6] as u16) & 0xF0) >> 4
    }

    pub fn console_type(&self) -> ConsoleType {
        match self.bytes[7] & 0x03 {
            0 => ConsoleType::Nes,
            1 => ConsoleType::VsSystem,
            2 => ConsoleType::Playchoice10,
            3 => ConsoleType::ExtendedConsoleType,
            _ => unreachable!(),
        }
    }

    fn is_ines_2_header(&self) -> bool {
        self.bytes[7] & (0x0C) == 8
    }
}

pub struct RomFile {
    pub header: INesHeader,
    pub trainer: Option<Box<[u8]>>,
    pub prg_rom: Box<[u8]>,
    pub chr_rom: Box<[u8]>,
}

#[derive(Debug)]
pub enum RomLoadError {
    UnsupportedFormat,
    UnsupportedConsole,
    UnsupportedMapper,
    MalformedRomFile,
}

impl RomFile {
    pub fn load(bytes: Vec<u8>) -> Result<RomFile, RomLoadError> {
        if bytes.len() < 16 {
            return Err(RomLoadError::MalformedRomFile);
        }
        if bytes[0..4] != [0x4E, 0x45, 0x53, 0x1A] {
            return Err(RomLoadError::UnsupportedFormat);
        }

        let header = INesHeader::new(bytes[0..16].try_into().unwrap());

        if header.is_ines_2_header() {
            return Err(RomLoadError::UnsupportedFormat);
        }
        if header.mapper_number() != 0 {
            return Err(RomLoadError::UnsupportedMapper);
        }
        if header.console_type() != ConsoleType::Nes {
            return Err(RomLoadError::UnsupportedConsole);
        }

        let mut cursor = INES_HEADER_LENGTH;
        let mut consume_bytes = |n: usize| -> Result<Box<[u8]>, RomLoadError> {
            match bytes.get(cursor..cursor + n) {
                Some(slice) => {
                    let bytes: Box<[u8]> = Box::from(slice);
                    cursor += n;
                    Ok(bytes)
                }
                None => Err(RomLoadError::MalformedRomFile),
            }
        };

        let trainer = if header.has_trainer() {
            Some(consume_bytes(INES_TRAINER_LENGTH)?)
        } else {
            None
        };
        let prg_rom = consume_bytes(header.prg_rom_size())?;
        let chr_rom = consume_bytes(header.chr_rom_size())?;

        Ok(RomFile {
            header,
            trainer,
            prg_rom,
            chr_rom,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

#[derive(Debug, PartialEq)]
pub enum ConsoleType {
    Nes,
    VsSystem,
    Playchoice10,
    ExtendedConsoleType,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_nes_test() {
        let binary = std::fs::read("test-roms/nestest/nestest.nes").unwrap();
        let rom_file = RomFile::load(binary).unwrap();

        assert_eq!(rom_file.header.mapper_number(), 0);
        assert_eq!(rom_file.header.mirroring(), Mirroring::Horizontal);
        assert_eq!(rom_file.header.has_persistent_memory(), false);
        assert_eq!(rom_file.prg_rom.len(), 16384);
        assert_eq!(rom_file.chr_rom.len(), 8192);
        assert_eq!(rom_file.trainer, None);
    }
}
