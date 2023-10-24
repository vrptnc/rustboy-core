use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::memory::Memory;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct LinearMemory<const SIZE: usize, const START_ADDRESS: u16> {
  #[serde_as(as = "[_;SIZE]")]
  bytes: [u8; SIZE],
}

impl<const SIZE: usize, const START_ADDRESS: u16> Memory for LinearMemory<SIZE, START_ADDRESS> {
  fn read(&self, address: u16) -> u8 {
    self.bytes[address as usize - START_ADDRESS as usize]
  }

  fn write(&mut self, address: u16, value: u8) {
    self.bytes[address as usize - START_ADDRESS as usize] = value
  }
}

impl<const SIZE: usize, const START_ADDRESS: u16> LinearMemory<SIZE, START_ADDRESS> {
  pub fn new() -> LinearMemory<SIZE, START_ADDRESS> {
    LinearMemory {
      bytes: [0; SIZE],
    }
  }
}