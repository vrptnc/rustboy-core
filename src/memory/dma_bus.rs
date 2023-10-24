use std::cell::RefCell;
use std::rc::Rc;

use crate::memory::mbc::MBC;
use crate::memory::memory::Memory;

pub struct DMAMemoryBus<'a> {
  pub rom: Rc<RefCell<dyn MBC>>,
  pub vram: &'a mut dyn Memory,
  pub wram: &'a mut dyn Memory,
  pub oam: &'a mut dyn Memory,
}

impl<'a> Memory for DMAMemoryBus<'a> {
  fn read(&self, address: u16) -> u8 {
    match address {
      0x0000..=0x7FFF => self.rom.borrow().read(address),
      0x8000..=0x9FFF => self.vram.read(address),
      0xA000..=0xBFFF => self.rom.borrow().read(address),
      0xC000..=0xDFFF => self.wram.read(address),
      _ => panic!("DMABus can't read from address {}", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      0x8000..=0x9FFF => self.vram.write(address, value),
      0xFE00..=0xFE9F => self.oam.write(address, value),
      _ => panic!("DMABus can't write to address {}", address)

    }
  }
}