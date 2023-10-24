use serde::{Deserialize, Serialize};

pub trait Memory {
  fn read(&self, address: u16) -> u8;
  fn write(&mut self, address: u16, value: u8);
}

pub struct MemoryAddress {}

impl MemoryAddress {
  pub const BANK: u16 = 0xFF50; // Bank register unmaps boot ROM
  pub const P1: u16 = 0xFF00; // Port P15-10
  pub const _SB: u16 = 0xFF01; // Serial transfer register
  pub const _SC: u16 = 0xFF02; // Serial control

  // Timer control
  pub const DIV: u16 = 0xFF04; // Divider
  pub const TIMA: u16 = 0xFF05; // Timer
  pub const TMA: u16 = 0xFF06; // Timer modulo
  pub const TAC: u16 = 0xFF07; // Timer control

  // Audio control
  pub const NR10: u16 = 0xFF10; // Channel 1 sweep control
  pub const NR11: u16 = 0xFF11; // Channel 1 length timer control
  pub const NR12: u16 = 0xFF12; // Channel 1 envelope control
  pub const NR13: u16 = 0xFF13; // Channel 1 low order frequency data
  pub const NR14: u16 = 0xFF14; // Channel 1 high order frequency data + trigger/length timer control
  pub const NR21: u16 = 0xFF16; // Channel 2 length timer control
  pub const NR22: u16 = 0xFF17; // Channel 2 volume and envelope control
  pub const NR23: u16 = 0xFF18; // Channel 2 low order frequency data
  pub const NR24: u16 = 0xFF19; // Channel 2 high order frequency data + trigger/length timer control
  pub const NR30: u16 = 0xFF1A; // Channel 3 on/off control
  pub const NR31: u16 = 0xFF1B; // Channel 3 length timer control
  pub const NR32: u16 = 0xFF1C; // Channel 3 volume control
  pub const NR33: u16 = 0xFF1D; // Channel 3 low order frequency data
  pub const NR34: u16 = 0xFF1E; // Channel 3 high order frequency data + trigger/length timer control
  pub const NR41: u16 = 0xFF20; // Channel 4 length timer control
  pub const NR42: u16 = 0xFF21; // Channel 4 envelope control
  pub const NR43: u16 = 0xFF22; // Channel 4 counter control
  pub const NR44: u16 = 0xFF23; // Channel 4 trigger/length timer control
  pub const NR50: u16 = 0xFF24; // Master volume and VIN mixing control
  pub const NR51: u16 = 0xFF25; // Sound mixing control
  pub const NR52: u16 = 0xFF26; // Sound on/off control

  // LCD control
  pub const LCDC: u16 = 0xFF40; // LCDC control
  pub const STAT: u16 = 0xFF41; // LCDC status information
  pub const SCY: u16 = 0xFF42; // Scroll Y register
  pub const SCX: u16 = 0xFF43; // Scroll X register
  pub const WY: u16 = 0xFF4A; // Window Y-coordinate
  pub const WX: u16 = 0xFF4B; // Window X-coordinate
  pub const LY: u16 = 0xFF44; // LCDC Y-coordinate
  pub const LYC: u16 = 0xFF45; // LY compare register

  // Palette control
  pub const BGP: u16 = 0xFF47; // BG palette data
  pub const OBP0: u16 = 0xFF48; // OBJ palette data 0
  pub const OBP1: u16 = 0xFF49; // OBJ palette data 1
  pub const BCPS: u16 = 0xFF68; // BG write specification
  pub const BCPD: u16 = 0xFF69; // BG write data
  pub const OCPS: u16 = 0xFF6A; // OBJ write specification
  pub const OCPD: u16 = 0xFF6B; // OBJ write data
  pub const OPRI: u16 = 0xFF6C; // Object priority mode

  pub const KEY0: u16 = 0xFF4C; // CPU speed switching
  pub const KEY1: u16 = 0xFF4D; // CPU speed switching

  // VRAM control
  pub const VBK: u16 = 0xFF4F; // VRAM bank specification

  // DMA control
  pub const DMA: u16 = 0xFF46; // DMA transfer control
  pub const HDMA1: u16 = 0xFF51; // Higher-order address of HDMA transfer source
  pub const HDMA2: u16 = 0xFF52; // Lower-order address of HDMA transfer source
  pub const HDMA3: u16 = 0xFF53; // Higher-order address of HDMA transfer destination
  pub const HDMA4: u16 = 0xFF54; // Lower-order address of HDMA transfer destination
  pub const HDMA5: u16 = 0xFF55; // H-blank and general-purpose DMA control

  // Infrared communication port
  pub const _RP: u16 = 0xFF56; // Infrared communication port

  // WRAM control
  pub const SVBK: u16 = 0xFF70; // WRAM bank specification

  // Interrupt control
  pub const IF: u16 = 0xFF0F; // Interrupt request flag
  pub const IE: u16 = 0xFFFF; // Interrupt enable flag
  pub const IME: u16 = 0xFEA0; // Master interrupt enable flag
  pub const RI: u16 = 0xFEA1; // Requested interrupt
}

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

pub enum RAMSize {
  Unavailable,
  KB8,
  KB32,
  KB64,
  KB128,
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
      0x00 => CGBMode::Monochrome,
      0x80 => CGBMode::Color,
      0x82 => CGBMode::PGB,
      0x84 => CGBMode::PGB,
      _ => panic!("Invalid CGB byte:  {:#x}", byte)
    }
  }
}

#[cfg(test)]
pub mod test {
  use crate::memory::memory::Memory;

  pub struct MockMemory {
    bytes: Vec<u8>,
  }

  impl MockMemory {
    pub fn new() -> MockMemory {
      MockMemory {
        bytes: vec![0; 0x10000]
      }
    }
  }

  impl Memory for MockMemory {
    fn read(&self, address: u16) -> u8 {
      self.bytes[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
      self.bytes[address as usize] = value
    }
  }
}
