use serde::{Deserialize, Serialize};

use crate::internal::memory::memory::Memory;

#[derive(Serialize, Deserialize)]
pub struct UnmappedMemory();

impl UnmappedMemory {
  pub fn new() -> Self {
    UnmappedMemory()
  }
}

impl Memory for UnmappedMemory {
  fn read(&self, address: u16) -> u8 {
    match address {
      0xFF03 => 0xFF,
      0xFF08..=0xFF0E => 0xFF,
      0xFF15 => 0xFF,
      0xFF1F => 0xFF,
      0xFF27..=0xFF2F => 0xFF,
      0xFF4E => 0xFF,
      0xFF57..=0xFF67 => 0xFF,
      0xFF6D..=0xFF6F => 0xFF,
      0xFF71..=0xFF7F => 0xFF,
      _ => panic!("UnmappedMemory can't read from address {}", address)
    }
  }

  fn write(&mut self, _address: u16, _value: u8) {
  }
}