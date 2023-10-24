use serde::{Deserialize, Serialize};

use crate::memory::memory::Memory;

#[derive(Serialize, Deserialize)]
pub struct ControlRegisters {
  key0: u8,
  bank: u8,
}

impl ControlRegisters {
  pub fn new() -> ControlRegisters {
    ControlRegisters {
      key0: 0,
      bank: 0,
    }
  }
}

impl Memory for ControlRegisters {
  fn read(&self, address: u16) -> u8 {
    match address {
      0xFF4C => self.key0,
      0xFF50 => self.bank,
      _ => panic!("Can't read control register from address {}", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      0xFF4C => self.key0 = value,
      0xFF50 => self.bank = value,
      _ => panic!("Can't write to control register at address {}", address)
    }
  }
}