use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::internal::memory::memory::Memory;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Stack {
  #[serde_as(as = "[_;Stack::SIZE]")]
  bytes: [u8; Stack::SIZE],
}

impl Stack {
  const START_ADDRESS: u16 = 0xFF80;
  const END_ADDRESS: u16 = 0xFFFE;
  const SIZE: usize = 127;

  pub fn new() -> Stack {
    Stack {
      bytes: [0; Stack::SIZE]
    }
  }
}

impl Memory for Stack {
  fn read(&self, address: u16) -> u8 {
    match address {
      Stack::START_ADDRESS..=Stack::END_ADDRESS => self.bytes[(address - Stack::START_ADDRESS) as usize],
      _ => panic!("Can't read address {} from stack", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      Stack::START_ADDRESS..=Stack::END_ADDRESS => self.bytes[(address - Stack::START_ADDRESS) as usize] = value,
      _ => panic!("Can't write to address {} in stack", address)
    }
  }
}