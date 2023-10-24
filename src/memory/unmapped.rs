use serde::{Deserialize, Serialize};

use crate::memory::memory::Memory;

#[derive(Serialize, Deserialize)]
pub struct UnmappedMemory {
  ff03: u8,
  ff08_ff0e: [u8; 7],
  ff27_ff2f: [u8; 9],
  ff4e: u8,
  ff57_ff67: [u8; 0x11],
  ff6d_ff6f: [u8; 0x3],
  ff71_ff7f: [u8; 0xF],
}

impl UnmappedMemory {
  pub fn new() -> Self {
    UnmappedMemory {
      ff03: 0,
      ff08_ff0e: [0; 7],
      ff27_ff2f: [0; 9],
      ff4e: 0,
      ff57_ff67: [0; 0x11],
      ff6d_ff6f: [0; 0x3],
      ff71_ff7f: [0; 0xF],
    }
  }
}

impl Memory for UnmappedMemory {
  fn read(&self, address: u16) -> u8 {
    match address {
      0xFF03 => self.ff03,
      0xFF08..=0xFF0E => self.ff08_ff0e[address as usize - 0xFF08],
      0xFF27..=0xFF2F => self.ff27_ff2f[address as usize - 0xFF27],
      0xFF4E => self.ff4e,
      0xFF57..=0xFF67 => self.ff57_ff67[address as usize - 0xFF57],
      0xFF6D..=0xFF6F => self.ff6d_ff6f[address as usize - 0xFF6D],
      0xFF71..=0xFF7F => self.ff71_ff7f[address as usize - 0xFF71],
      _ => panic!("UnmappedMemory can't read from address {}", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      0xFF03 => self.ff03 = value,
      0xFF08..=0xFF0E => self.ff08_ff0e[address as usize - 0xFF08] = value,
      0xFF27..=0xFF2F => self.ff27_ff2f[address as usize - 0xFF27] = value,
      0xFF4E => self.ff4e = value,
      0xFF57..=0xFF67 => self.ff57_ff67[address as usize - 0xFF57] = value,
      0xFF6D..=0xFF6F => self.ff6d_ff6f[address as usize - 0xFF6D] = value,
      0xFF71..=0xFF7F => self.ff71_ff7f[address as usize - 0xFF71] = value,
      _ => panic!("UnmappedMemory can't write to address {}", address)
    }
  }
}