use core::fmt::{Debug, Formatter};
use serde::{Deserialize, Serialize};
use crate::internal::memory::oam::ObjectAttributes;
use crate::memory::Licensee::{NewLicensee, OldLicensee};

#[derive(Copy, Clone)]
pub struct OAMObject {
  pub lcd_y: u8,
  pub lcd_x: u8,
  pub tile_index: u8,
  pub attributes: ObjectAttributes,
}

#[derive(Copy, Clone)]
pub enum ROMSize {
  KB32,
  KB64,
  KB128,
  KB256,
  KB512,
  MB1,
  MB2,
  MB4,
  MB8,
}

pub enum Licensee {
  OldLicensee(u8),
  NewLicensee(char, char),
}

impl Licensee {
  pub fn from_bytes(rom_bytes: &[u8]) -> Self {
    let licensee_code = rom_bytes[0x14B];
    if licensee_code == 0x33 {
      NewLicensee(
        rom_bytes[0x0144] as char,
        rom_bytes[0x0145] as char,
      )
    } else {
      OldLicensee(licensee_code)
    }
  }

  pub fn is_licensed_by_nintendo(&self) -> bool {
    match self {
      OldLicensee(code) => *code == 0x01,
      NewLicensee(upper, lower) => *upper == '0' && *lower == '1'
    }
  }

  pub fn get_name(&self) -> String {
    let name = match self {
      OldLicensee(code) => match *code {
        0x00 => "None",
        0x01 => "Nintendo",
        0x08 => "Capcom",
        0x09 => "Hot-B",
        0x0A => "Jaleco",
        0x0B => "Coconuts Japan",
        0x0C => "Elite Systems",
        0x13 => "EA (Electronic Arts)",
        0x18 => "Hudsonsoft",
        0x19 => "ITC Entertainment",
        0x1A => "Yanoman",
        0x1D => "Japan Clary",
        0x1F => "Virgin Interactive",
        0x24 => "PCM Complete",
        0x25 => "San-X",
        0x28 => "Kotobuki Systems",
        0x29 => "Seta",
        0x30 => "Infogrames",
        0x31 => "Nintendo",
        0x32 => "Bandai",
        0x34 => "Konami",
        0x35 => "HectorSoft",
        0x38 => "Capcom",
        0x39 => "Banpresto",
        0x3C => ".Entertainment i",
        0x3E => "Gremlin",
        0x41 => "Ubisoft",
        0x42 => "Atlus",
        0x44 => "Malibu",
        0x46 => "Angel",
        0x47 => "Spectrum Holoby",
        0x49 => "Irem",
        0x4A => "Virgin Interactive",
        0x4D => "Malibu",
        0x4F => "U.S. Gold",
        0x50 => "Absolute",
        0x51 => "Acclaim",
        0x52 => "Activision",
        0x53 => "American Sammy",
        0x54 => "GameTek",
        0x55 => "Park Place",
        0x56 => "LJN",
        0x57 => "Matchbox",
        0x59 => "Milton Bradley",
        0x5A => "Mindscape",
        0x5B => "Romstar",
        0x5C => "Naxat Soft",
        0x5D => "Tradewest",
        0x60 => "Titus",
        0x61 => "Virgin Interactive",
        0x67 => "Ocean Interactive",
        0x69 => "EA (Electronic Arts)",
        0x6E => "Elite Systems",
        0x6F => "Electro Brain",
        0x70 => "Infogrames",
        0x71 => "Interplay",
        0x72 => "Broderbund",
        0x73 => "Sculptered Soft",
        0x75 => "The Sales Curve",
        0x78 => "t.hq",
        0x79 => "Accolade",
        0x7A => "Triffix Entertainment",
        0x7C => "Microprose",
        0x7F => "Kemco",
        0x80 => "Misawa Entertainment",
        0x83 => "Lozc",
        0x86 => "Tokuma Shoten Intermedia",
        0x8B => "Bullet-Proof Software",
        0x8C => "Vic Tokai",
        0x8E => "Ape",
        0x8F => "I’Max",
        0x91 => "Chunsoft Co.",
        0x92 => "Video System",
        0x93 => "Tsubaraya Productions Co.",
        0x95 => "Varie Corporation",
        0x96 => "Yonezawa/S’Pal",
        0x97 => "Kaneko",
        0x99 => "Arc",
        0x9A => "Nihon Bussan",
        0x9B => "Tecmo",
        0x9C => "Imagineer",
        0x9D => "Banpresto",
        0x9F => "Nova",
        0xA1 => "Hori Electric",
        0xA2 => "Bandai",
        0xA4 => "Konami",
        0xA6 => "Kawada",
        0xA7 => "Takara",
        0xA9 => "Technos Japan",
        0xAA => "Broderbund",
        0xAC => "Toei Animation",
        0xAD => "Toho",
        0xAF => "Namco",
        0xB0 => "acclaim",
        0xB1 => "ASCII or Nexsoft",
        0xB2 => "Bandai",
        0xB4 => "Square Enix",
        0xB6 => "HAL Laboratory",
        0xB7 => "SNK",
        0xB9 => "Pony Canyon",
        0xBA => "Culture Brain",
        0xBB => "Sunsoft",
        0xBD => "Sony Imagesoft",
        0xBF => "Sammy",
        0xC0 => "Taito",
        0xC2 => "Kemco",
        0xC3 => "Squaresoft",
        0xC4 => "Tokuma Shoten Intermedia",
        0xC5 => "Data East",
        0xC6 => "Tonkinhouse",
        0xC8 => "Koei",
        0xC9 => "UFL",
        0xCA => "Ultra",
        0xCB => "Vap",
        0xCC => "Use Corporation",
        0xCD => "Meldac",
        0xCE => ".Pony Canyon or",
        0xCF => "Angel",
        0xD0 => "Taito",
        0xD1 => "Sofel",
        0xD2 => "Quest",
        0xD3 => "Sigma Enterprises",
        0xD4 => "ASK Kodansha Co.",
        0xD6 => "Naxat Soft",
        0xD7 => "Copya System",
        0xD9 => "Banpresto",
        0xDA => "Tomy",
        0xDB => "LJN",
        0xDD => "NCS",
        0xDE => "Human",
        0xDF => "Altron",
        0xE0 => "Jaleco",
        0xE1 => "Towa Chiki",
        0xE2 => "Yutaka",
        0xE3 => "Varie",
        0xE5 => "Epcoh",
        0xE7 => "Athena",
        0xE8 => "Asmik ACE Entertainment",
        0xE9 => "Natsume",
        0xEA => "King Records",
        0xEB => "Atlus",
        0xEC => "Epic/Sony Records",
        0xEE => "IGS",
        0xF0 => "A Wave",
        0xF3 => "Extreme Entertainment",
        0xFF => "LJN",
        _ => "Unknown"
      },
      NewLicensee(upper, lower) => match (*upper, *lower) {
        ('0', '0') => "None",
        ('0', '1') => "Nintendo R&D1",
        ('0', '8') => "Capcom",
        ('1', '3') => "Electronic Arts",
        ('1', '8') => "Hudson Soft",
        ('1', '9') => "b-ai",
        ('2', '0') => "kss",
        ('2', '2') => "pow",
        ('2', '4') => "PCM Complete",
        ('2', '5') => "san-x",
        ('2', '8') => "Kemco Japan",
        ('2', '9') => "seta",
        ('3', '0') => "Viacom",
        ('3', '1') => "Nintendo",
        ('3', '2') => "Bandai",
        ('3', '3') => "Ocean/Acclaim",
        ('3', '4') => "Konami",
        ('3', '5') => "Hector",
        ('3', '7') => "Taito",
        ('3', '8') => "Hudson",
        ('3', '9') => "Banpresto",
        ('4', '1') => "Ubi Soft",
        ('4', '2') => "Atlus",
        ('4', '4') => "Malibu",
        ('4', '6') => "angel",
        ('4', '7') => "Bullet-Proof",
        ('4', '9') => "irem",
        ('5', '0') => "Absolute",
        ('5', '1') => "Acclaim",
        ('5', '2') => "Activision",
        ('5', '3') => "American sammy",
        ('5', '4') => "Konami",
        ('5', '5') => "Hi tech entertainment",
        ('5', '6') => "LJN",
        ('5', '7') => "Matchbox",
        ('5', '8') => "Mattel",
        ('5', '9') => "Milton Bradley",
        ('6', '0') => "Titus",
        ('6', '1') => "Virgin",
        ('6', '4') => "LucasArts",
        ('6', '7') => "Ocean",
        ('6', '9') => "Electronic Arts",
        ('7', '0') => "Infogrames",
        ('7', '1') => "Interplay",
        ('7', '2') => "Broderbund",
        ('7', '3') => "sculptured",
        ('7', '5') => "sci",
        ('7', '8') => "THQ",
        ('7', '9') => "Accolade",
        ('8', '0') => "misawa",
        ('8', '3') => "lozc",
        ('8', '6') => "Tokuma Shoten Intermedia",
        ('8', '7') => "Tsukuda Original",
        ('9', '1') => "Chunsoft",
        ('9', '2') => "Video system",
        ('9', '3') => "Ocean/Acclaim",
        ('9', '5') => "Varie",
        ('9', '6') => "Yonezawa/s’pal",
        ('9', '7') => "Kaneko",
        ('9', '9') => "Pack in soft",
        ('9', 'H') => "Bottom Up",
        ('A', '4') => "Konami (Yu-Gi-Oh!)",
        _ => "Unknown"
      }
    };
    String::from(name)
  }
}

#[derive(Copy, Clone)]
pub enum CartridgeType {
  MBC,
  MBC1,
  MBC2,
  MMM01,
  MBC3,
  MBC5,
  MBC6,
  MBC7,
  PocketCamera,
  Bandai,
  HuC3,
  HuC1,
}

impl CartridgeType {
  pub fn from_byte(byte: u8) -> Self {
    match byte {
      0x00 => CartridgeType::MBC,
      0x01..=0x03 => CartridgeType::MBC1,
      0x05..=0x06 => CartridgeType::MBC2,
      0x0B..=0x0D => CartridgeType::MMM01,
      0x0F..=0x13 => CartridgeType::MBC3,
      0x19..=0x1E => CartridgeType::MBC5,
      0x20 => CartridgeType::MBC6,
      0x22 => CartridgeType::MBC7,
      0xFC => CartridgeType::PocketCamera,
      0xFD => CartridgeType::Bandai,
      0xFE => CartridgeType::HuC3,
      0xFF => CartridgeType::HuC1,
      _ => panic!("Unknown cartridge for byte {:#x}", byte)
    }
  }
}

impl Debug for CartridgeType {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    match self {
      CartridgeType::MBC => write!(f, "MBC"),
      CartridgeType::MBC1 => write!(f, "MBC1"),
      CartridgeType::MBC2 => write!(f, "MBC2"),
      CartridgeType::MMM01 => write!(f, "MMM01"),
      CartridgeType::MBC3 => write!(f, "MBC3"),
      CartridgeType::MBC5 => write!(f, "MBC5"),
      CartridgeType::MBC6 => write!(f, "MBC6"),
      CartridgeType::MBC7 => write!(f, "MBC7"),
      CartridgeType::PocketCamera => write!(f, "Pocket Camera"),
      CartridgeType::Bandai => write!(f, "Bandai"),
      CartridgeType::HuC3 => write!(f, "HuC3"),
      CartridgeType::HuC1 => write!(f, "HuC1")
    }
  }
}

impl Debug for ROMSize {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    match self {
      ROMSize::KB32 => write!(f, "32 kB"),
      ROMSize::KB64 => write!(f, "64 kB"),
      ROMSize::KB128 => write!(f, "128 kB"),
      ROMSize::KB256 => write!(f, "256 kB"),
      ROMSize::KB512 => write!(f, "512 kB"),
      ROMSize::MB1 => write!(f, "1 MB"),
      ROMSize::MB2 => write!(f, "2 MB"),
      ROMSize::MB4 => write!(f, "4 MB"),
      ROMSize::MB8 => write!(f, "8 MB")
    }
  }
}

impl ROMSize {
  pub fn from_byte(byte: u8) -> ROMSize {
    match byte {
      0x00 => ROMSize::KB32,
      0x01 => ROMSize::KB64,
      0x02 => ROMSize::KB128,
      0x03 => ROMSize::KB256,
      0x04 => ROMSize::KB512,
      0x05 => ROMSize::MB1,
      0x06 => ROMSize::MB2,
      0x07 => ROMSize::MB4,
      0x08 => ROMSize::MB8,
      _ => panic!("Byte {} does not correspond to any known ROM size", byte)
    }
  }

  pub fn bytes(&self) -> usize {
    match self {
      ROMSize::KB32 => 0x8000,
      ROMSize::KB64 => 0x10000,
      ROMSize::KB128 => 0x20000,
      ROMSize::KB256 => 0x40000,
      ROMSize::KB512 => 0x80000,
      ROMSize::MB1 => 0x100000,
      ROMSize::MB2 => 0x200000,
      ROMSize::MB4 => 0x400000,
      ROMSize::MB8 => 0x800000
    }
  }
}

#[derive(Copy, Clone)]
pub enum RAMSize {
  Unavailable,
  KB8,
  KB32,
  KB64,
  KB128,
}

impl Debug for RAMSize {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    match self {
      RAMSize::Unavailable => write!(f, "Unavailable"),
      RAMSize::KB8 => write!(f, "8 kB"),
      RAMSize::KB32 => write!(f, "32 kB"),
      RAMSize::KB64 => write!(f, "64 kB"),
      RAMSize::KB128 => write!(f, "128 kB"),
    }
  }
}

impl RAMSize {
  pub fn from_byte(byte: u8) -> RAMSize {
    match byte {
      0x00 => RAMSize::Unavailable,
      0x01 => RAMSize::Unavailable,
      0x02 => RAMSize::KB8,
      0x03 => RAMSize::KB32,
      0x04 => RAMSize::KB128,
      0x05 => RAMSize::KB64,
      _ => panic!("Byte {} does not correspond to any known RAM size", byte)
    }
  }

  pub fn bytes(&self) -> usize {
    match self {
      RAMSize::Unavailable => 0,
      RAMSize::KB8 => 0x8000,
      RAMSize::KB32 => 0x8000,
      RAMSize::KB64 => 0x10000,
      RAMSize::KB128 => 0x20000,
    }
  }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum CGBMode {
  Monochrome,
  Color,
  PGB,
}

impl CGBMode {
  pub fn from_byte(byte: u8) -> CGBMode {
    match byte & 0xBF {
      0x80 => CGBMode::Color,
      0x82 => CGBMode::PGB,
      0x84 => CGBMode::PGB,
      _ => CGBMode::Monochrome
    }
  }
}

impl Debug for CGBMode {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    match self {
      CGBMode::Monochrome => write!(f, "Monochrome"),
      CGBMode::Color => write!(f, "Color"),
      CGBMode::PGB => write!(f, "PGB")
    }
  }
}
