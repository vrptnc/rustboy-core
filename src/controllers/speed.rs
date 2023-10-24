use mockall::automock;
use serde::{Deserialize, Serialize};

use crate::cpu::cpu::CPU;
use crate::memory::memory::{Memory, MemoryAddress};
use crate::util::bit_util::BitUtil;

#[automock]
pub trait SpeedController {
  fn double_speed(&self) -> bool;
}

#[derive(Serialize, Deserialize)]
pub struct SpeedControllerImpl(u8);

impl SpeedControllerImpl {
  pub fn new() -> Self {
    SpeedControllerImpl(0x00)
  }

  pub fn tick(&mut self, cpu: &mut dyn CPU) {
    if cpu.stopped() & self.0.get_bit(0) {
      self.0 = self.0.toggle_bit(7);
      self.0 = self.0.reset_bit(0);
      cpu.resume();
    }
  }
}

impl SpeedController for SpeedControllerImpl {
  fn double_speed(&self) -> bool {
    self.0.get_bit(7)
  }
}

impl Memory for SpeedControllerImpl {
  fn read(&self, address: u16) -> u8 {
    match address {
      MemoryAddress::KEY1 => self.0,
      _ => panic!("SpeedController can't read value at address {}", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      MemoryAddress::KEY1 => self.0 = if value.get_bit(0) { self.0.set_bit(0) } else { self.0.reset_bit(0) },
      _ => panic!("SpeedController can't write to address {}", address)
    }
  }
}