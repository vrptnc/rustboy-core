use log::info;
use crate::internal::memory::mbc::{Loadable, MBC};
use crate::internal::memory::memory::Memory;
use crate::memory::{RAMSize, ROMSize};

pub struct MBC5 {
  ram_enabled: bool,
  ram_bank_address: usize,
  rom_bank_address: usize,
  rom: Vec<u8>,
  ram: Vec<u8>,
}

impl MBC for MBC5 {}

impl MBC5 {
  pub fn new(rom_size: ROMSize, ram_size: RAMSize) -> MBC5 {
    info!("Loading new MBC5 cartridge with ROM size {:?} and RAM size {:?}", rom_size, ram_size);
    MBC5 {
      ram_enabled: false,
      ram_bank_address: 0x00,
      rom_bank_address: 0x00,
      ram: vec![0; ram_size.bytes()],
      rom: vec![0; rom_size.bytes()],
    }
  }
}

impl Memory for MBC5 {
  fn read(&self, address: u16) -> u8 {
    match address {
      0x0000..=0x3FFF => {
        self.rom[address as usize]
      }
      0x4000..=0x7FFF => {
        let address_in_rom = ((address as usize) & 0x3FFF) | (self.rom_bank_address << 14);
        self.rom[address_in_rom]
      }
      0xA000..=0xBFFF => {
        let address_in_ram = ((address as usize) & 0x1FFF) | (self.ram_bank_address << 13);
        self.ram[address_in_ram]
      }
      _ => panic!("Can't read from address {:#06x} on MBC5", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      0x0000..=0x1FFF => {
        self.ram_enabled = (value & 0x0F) == 0x0A;
      }
      0x2000..=0x2FFF => {
        self.rom_bank_address = (self.rom_bank_address & 0x100) | (value as usize);
      }
      0x3000..=0x3FFF => {
        self.rom_bank_address = ((value as usize) << 8) | (self.rom_bank_address & 0xFF);
      }
      0x4000..=0x5FFF => {
        self.ram_bank_address = value as usize;
      }
      0xA000..=0xBFFF => {
        if self.ram_enabled {
          let address_in_ram = ((address as usize) & 0x1FFF) | (self.ram_bank_address << 13);
          self.ram[address_in_ram] = value
        }
      }
      _ => {
        // panic!("Can't write to address {:#06x} on MBC5", address)
      }
    };
  }
}

impl Loadable for MBC5 {
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
    let mut memory = MBC5::new(ROMSize::KB256, RAMSize::KB64);
    memory.write(0x0000, 0xA); // Enable RAM
    memory.write(0xA000, 0xAB);
    memory.write(0xB000, 0xCD);
    memory.write(0xBFFF, 0xEF);
    memory.write(0x4000, 0x07); // Switch RAM bank to bank 7
    memory.write(0xA000, 0x12);
    memory.write(0xB000, 0x34);
    memory.write(0xBFFF, 0x56);
    memory.write(0x4000, 0x00);
    // Switch RAM bank to bank 0
    assert_eq_hex!(memory.read(0xA000), 0xAB);
    assert_eq_hex!(memory.read(0xB000), 0xCD);
    assert_eq_hex!(memory.read(0xBFFF), 0xEF);
    memory.write(0x4000, 0x07);
    // Switch RAM bank to bank 7
    assert_eq_hex!(memory.read(0xA000), 0x12);
    assert_eq_hex!(memory.read(0xB000), 0x34);
    assert_eq_hex!(memory.read(0xBFFF), 0x56);
  }

  #[test]
  fn read_lower_rom() {
    let mut memory = MBC5::new(ROMSize::KB256, RAMSize::KB64);
    memory.load_byte(0x0000, 0x12);
    memory.load_byte(0x2ABC, 0x34);
    memory.load_byte(0x3FFF, 0x56);
    assert_eq_hex!(memory.read(0x0000), 0x12);
    assert_eq_hex!(memory.read(0x2ABC), 0x34);
    assert_eq_hex!(memory.read(0x3FFF), 0x56);
  }

  #[test]
  fn read_upper_rom() {
    let mut memory = MBC5::new(ROMSize::MB8, RAMSize::KB64);
    memory.load_byte(0x4000, 0x12);
    memory.load_byte(0x5ABC, 0x34);
    memory.load_byte(0x7FFF, 0x56);
    memory.load_byte(0x14000, 0x78); // Load bytes into bank 5
    memory.load_byte(0x15ABC, 0x9A);
    memory.load_byte(0x17FFF, 0xBC);
    memory.load_byte(0x420000, 0xAA); // Load bytes into bank 0x108
    memory.load_byte(0x421ABC, 0xBB);
    memory.load_byte(0x423FFF, 0xCC);
    memory.write(0x2000, 0x01);
    // Switch to bank 1
    assert_eq_hex!(memory.read(0x4000), 0x12);
    assert_eq_hex!(memory.read(0x5ABC), 0x34);
    assert_eq_hex!(memory.read(0x7FFF), 0x56);
    memory.write(0x2000, 0x05);
    // Switch to bank 5
    assert_eq_hex!(memory.read(0x4000), 0x78);
    assert_eq_hex!(memory.read(0x5ABC), 0x9A);
    assert_eq_hex!(memory.read(0x7FFF), 0xBC);
    memory.write(0x3000, 0x01); // Switch to bank 0x108
    memory.write(0x2000, 0x08);
    // Switch to bank 0x108
    assert_eq_hex!(memory.read(0x4000), 0xAA);
    assert_eq_hex!(memory.read(0x5ABC), 0xBB);
    assert_eq_hex!(memory.read(0x7FFF), 0xCC);
  }
}