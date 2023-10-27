use crate::internal::memory::mbc::{Loadable, MBC};
use crate::internal::memory::memory::{Memory, ROMSize};
use crate::internal::util::bit_util::BitUtil;

pub struct MBC2 {
  ram_enabled: bool,
  bank_address: usize,
  rom: Vec<u8>,
  ram: Vec<u8>,
}

impl MBC for MBC2 {}

impl MBC2 {
  pub fn new(rom_size: ROMSize) -> MBC2 {
    MBC2 {
      ram_enabled: false,
      bank_address: 0x01,
      ram: vec![0; 0x200],
      rom: vec![0; rom_size.bytes()],
    }
  }
}

impl Memory for MBC2 {
  fn read(&self, address: u16) -> u8 {
    match address {
      0x0000..=0x3FFF => {
        self.rom[address as usize]
      },
      0x4000..=0x7FFF => {
        let address_in_rom = ((address as usize) & 0x3FFF) | (self.bank_address << 14);
        self.rom[address_in_rom]
      },
      0xA000..=0xBFFF => {
        let address_in_ram = (address as usize) & 0x1FF;
        self.ram[address_in_ram]
      },
      _ => panic!("Can't read from address {:#06x} on MBC2", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      0x0000..=0x3FFF => {
        if address.get_bit(8) {
          self.bank_address = (value & 0x1F) as usize;
          if self.bank_address == 0 {
            self.bank_address = 1;
          }
        } else {
          self.ram_enabled = (value & 0x0F) == 0x0A;
        }
      },
      0xA000..=0xBFFF => {
        let address_in_ram = (address as usize) & 0x1FF;
        self.ram[address_in_ram] = value;
      },
      _ => panic!("Can't write to address {:#06x} on MBC2", address)
    };
  }
}

impl Loadable for MBC2 {
  fn load_byte(&mut self, address: usize, value: u8) {
    self.rom[address] = value;
  }

  fn load_bytes(&mut self, address: usize, values: &[u8]) {
    self.rom.as_mut_slice()[address..(address + values.len())].copy_from_slice(values);
  }
}

#[cfg(test)]
mod tests {
  use assert_hex::assert_eq_hex;

  use super::*;

  #[test]
  fn read_write_ram() {
    let mut memory = MBC2::new(ROMSize::KB256);
    memory.write(0x0000, 0xA); // Enable RAM
    memory.write(0xA000, 0xAB);
    memory.write(0xA080, 0xCD);
    memory.write(0xA1FF, 0xEF);
    assert_eq_hex!(memory.read(0xA000), 0xAB);
    assert_eq_hex!(memory.read(0xA080), 0xCD);
    assert_eq_hex!(memory.read(0xA1FF), 0xEF);
  }

  #[test]
  fn read_write_ram_wraps() {
    let mut memory = MBC2::new(ROMSize::KB256);
    memory.write(0x0000, 0xA); // Enable RAM
    memory.write(0xA000, 0xAB);
    memory.write(0xAC80, 0xCD);
    memory.write(0xB3FF, 0xEF);
    assert_eq_hex!(memory.read(0xA000), 0xAB);
    assert_eq_hex!(memory.read(0xA080), 0xCD);
    assert_eq_hex!(memory.read(0xA1FF), 0xEF);
  }

  #[test]
  fn read_lower_rom() {
    let mut memory = MBC2::new(ROMSize::KB256);
    memory.load_byte(0x0000, 0x12);
    memory.load_byte(0x2ABC, 0x34);
    memory.load_byte(0x3FFF, 0x56);
    assert_eq_hex!(memory.read(0x0000), 0x12);
    assert_eq_hex!(memory.read(0x2ABC), 0x34);
    assert_eq_hex!(memory.read(0x3FFF), 0x56);
  }

  #[test]
  fn read_upper_rom() {
    let mut memory = MBC2::new(ROMSize::KB256);
    memory.load_byte(0x4000, 0x12);
    memory.load_byte(0x5ABC, 0x34);
    memory.load_byte(0x7FFF, 0x56);
    memory.load_byte(0x14000, 0x78); // Load bytes into bank 5
    memory.load_byte(0x15ABC, 0x9A);
    memory.load_byte(0x17FFF, 0xBC);
    assert_eq_hex!(memory.read(0x4000), 0x12);
    assert_eq_hex!(memory.read(0x5ABC), 0x34);
    assert_eq_hex!(memory.read(0x7FFF), 0x56);
    memory.write(0x0100, 0x05); // Switch to bank 5
    assert_eq_hex!(memory.read(0x4000), 0x78);
    assert_eq_hex!(memory.read(0x5ABC), 0x9A);
    assert_eq_hex!(memory.read(0x7FFF), 0xBC);
  }

  #[test]
  fn bank_address_is_never_zero() {
    let mut memory = MBC2::new(ROMSize::KB256);
    memory.write(0x0100, 0x00);
    memory.load_byte(0x4000, 0x12);
    memory.load_byte(0x5ABC, 0x34);
    memory.load_byte(0x7FFF, 0x56);
    assert_eq_hex!(memory.read(0x4000), 0x12);
    assert_eq_hex!(memory.read(0x5ABC), 0x34);
    assert_eq_hex!(memory.read(0x7FFF), 0x56);

  }
}