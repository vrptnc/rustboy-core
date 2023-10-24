use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::memory::memory::{Memory, MemoryAddress};

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct WRAMImpl {
  bank_index: u8,
  #[serde_as(as = "[[_;WRAMImpl::BANK_SIZE]; 8]")]
  bytes: [[u8; WRAMImpl::BANK_SIZE]; 8],
}

impl WRAMImpl {
  const START_ADDRESS: u16 = 0xC000;
  const END_ADDRESS: u16 = 0xDFFF;
  const BANK_SIZE: usize = 0x1000;
  const BANK_0_END_ADDRESS: u16 = 0xCFFF;
  const DYNAMIC_BANK_START_ADDRESS: u16 = 0xD000;

  pub fn new() -> WRAMImpl {
    WRAMImpl {
      bank_index: 1,
      bytes: [[0; WRAMImpl::BANK_SIZE]; 8],
    }
  }
}

impl Memory for WRAMImpl {
  fn read(&self, address: u16) -> u8 {
    match address {
      WRAMImpl::START_ADDRESS..=WRAMImpl::BANK_0_END_ADDRESS => {
        self.bytes[0][(address - WRAMImpl::START_ADDRESS) as usize]
      }
      WRAMImpl::DYNAMIC_BANK_START_ADDRESS..=WRAMImpl::END_ADDRESS => {
        self.bytes[self.bank_index as usize][(address - WRAMImpl::DYNAMIC_BANK_START_ADDRESS) as usize]
      }
      MemoryAddress::SVBK => self.bank_index,
      _ => panic!("Can't read address {} from WRAM", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      WRAMImpl::START_ADDRESS..=WRAMImpl::BANK_0_END_ADDRESS => {
        self.bytes[0][(address - WRAMImpl::START_ADDRESS) as usize] = value;
      }
      WRAMImpl::DYNAMIC_BANK_START_ADDRESS..=WRAMImpl::END_ADDRESS => {
        self.bytes[self.bank_index as usize][(address - WRAMImpl::DYNAMIC_BANK_START_ADDRESS) as usize] = value;
      }
      MemoryAddress::SVBK => {
        self.bank_index = value & 0x07;
        if self.bank_index == 0 {
          self.bank_index = 1;
        }
      }
      _ => panic!("Can't write to address {} in WRAM", address)
    }
  }
}

