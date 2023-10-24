use crate::memory::mbc::{Loadable, MBC};
use crate::memory::memory::{Memory, RAMSize, ROMSize};

pub struct MBC1 {
  ram_enabled: bool,
  upper_bank_address_enabled: bool,
  lower_bank_address: usize,
  upper_bank_address: usize,
  rom: Vec<u8>,
  ram: Vec<u8>,
}

impl MBC for MBC1 {}

impl MBC1 {
  pub fn new(rom_size: ROMSize, ram_size: RAMSize) -> MBC1 {
    MBC1 {
      ram_enabled: false,
      upper_bank_address_enabled: false,
      lower_bank_address: 0x01,
      upper_bank_address: 0x00,
      ram: vec![0; ram_size.bytes()],
      rom: vec![0; rom_size.bytes()],
    }
  }
}

impl Loadable for MBC1 {
  fn load_byte(&mut self, address: usize, value: u8) {
    self.rom[address] = value;
  }

  fn load_bytes(&mut self, address: usize, values: &[u8]) {
    self.rom.as_mut_slice()[address..(address + values.len())].copy_from_slice(values);
  }
}

impl Memory for MBC1 {
  fn read(&self, address: u16) -> u8 {
    match address {
      0x0000..=0x3FFF => {
        let address_in_rom = ((address as usize) & 0x3FFF) | (if self.upper_bank_address_enabled { self.upper_bank_address << 19 } else { 0 });
        self.rom[address_in_rom]
      }
      0x4000..=0x7FFF => {
        let address_in_rom = ((address as usize) & 0x3FFF) | (self.lower_bank_address << 14) | (self.upper_bank_address << 19);
        self.rom[address_in_rom]
      }
      0xA000..=0xBFFF => {
        let address_in_ram = ((address as usize) & 0x1FFF) | (if self.upper_bank_address_enabled { self.upper_bank_address << 13 } else { 0 });
        self.ram[address_in_ram]
      }
      _ => panic!("Can't read from address {:#06x} on MBC1", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      0x0000..=0x1FFF => {
        self.ram_enabled = (value & 0x0F) == 0x0A;
      }
      0x2000..=0x3FFF => {
        self.lower_bank_address = (value & 0x1F) as usize;
        if self.lower_bank_address == 0 {
          self.lower_bank_address = 1;
        }
      }
      0x4000..=0x5FFF => {
        self.upper_bank_address = (value & 0x03) as usize;
      }
      0x6000..=0x7FFF => {
        self.upper_bank_address_enabled = (value & 0x01) == 0x01;
      }
      0xA000..=0xBFFF => {
        if self.ram_enabled {
          let address_in_ram = ((address as usize) & 0x1FFF) | (if self.upper_bank_address_enabled { self.upper_bank_address << 13 } else { 0 });
          self.ram[address_in_ram] = value;
        }
      }
      _ => panic!("Can't write to address {:#06x} on MBC1", address)
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn read_write_ram() {
    let mut memory = MBC1::new(ROMSize::MB8, RAMSize::KB32);
    memory.write(0x0000, 0x0A); // Enable RAM
    memory.write(0x6000, 0x01); // Enable upper bank address
    (0u8..=3u8).for_each(|bank| {
      memory.write(0x4000, bank); //Switch to bank
      memory.write(0xA123, 0x0A | (bank << 4));
      memory.write(0xB456, 0x0B | (bank << 4));
    });
    (0u8..=3u8).for_each(|bank| {
      memory.write(0x4000, bank); //Switch to bank
      assert_eq!(memory.read(0xA123), 0x0A | (bank << 4));
      assert_eq!(memory.read(0xB456), 0x0B | (bank << 4));
    });
  }

  #[test]
  fn read_write_ram_without_upper_address() {
    let mut memory = MBC1::new(ROMSize::MB8, RAMSize::KB32);
    memory.write(0x0000, 0x0A); // Enable RAM
    memory.write(0x6000, 0x00); // Disable upper bank address
    memory.write(0x4000, 0x03); //Switch to bank 3
    memory.write(0xA123, 0xAB);
    memory.write(0xB456, 0xCD);
    memory.write(0x4000, 0x00); //Switch to bank 0
    assert_eq!(memory.read(0xA123), 0xAB);
    assert_eq!(memory.read(0xB456), 0xCD);
  }

  #[test]
  fn read_lower_rom_without_upper_bank_address() {
    let mut memory = MBC1::new(ROMSize::MB8, RAMSize::KB32);
    memory.write(0x6000, 0x00); // Disable upper bank address
    memory.write(0x4000, 0x03); // Set upper bank address to 3
    memory.load_byte(0x0000, 0xAB);
    memory.load_byte(0x3FFF, 0xCD);
    assert_eq!(memory.read(0x0000), 0xAB);
    assert_eq!(memory.read(0x3FFF), 0xCD);
  }
  
  #[test]
  fn read_lower_rom_with_upper_bank_address() {
    let mut memory = MBC1::new(ROMSize::MB8, RAMSize::KB32);
    memory.load_byte(0x1234, 0x89);
    memory.load_byte(0x81234, 0xAB);
    memory.load_byte(0x101234, 0xCD);
    memory.load_byte(0x181234, 0xEF);
    memory.write(0x6000, 0x01); // Enable upper bank address
    memory.write(0x4000, 0x00); // Set upper bank address to 0
    assert_eq!(memory.read(0x1234), 0x89);
    memory.write(0x4000, 0x01); // Set upper bank address to 1
    assert_eq!(memory.read(0x1234), 0xAB);
    memory.write(0x4000, 0x02); // Set upper bank address to 2
    assert_eq!(memory.read(0x1234), 0xCD);
    memory.write(0x4000, 0x03); // Set upper bank address to 3
    assert_eq!(memory.read(0x1234), 0xEF);
  }

  #[test]
  fn read_upper_rom() {
    let mut memory = MBC1::new(ROMSize::MB8, RAMSize::KB32);
    memory.load_byte(0x1132A7, 0xAB);
    memory.write(0x2000, 0x04); // Set lower bank address to 4
    memory.write(0x4000, 0x2); // Set upper bank address to 2
    assert_eq!(memory.read(0x72A7), 0xAB);
  }

  #[test]
  fn lower_bank_address_is_never_zero() {
    let mut memory = MBC1::new(ROMSize::MB8, RAMSize::KB32);
    memory.load_byte(0x1072A7, 0xAB);
    memory.write(0x2000, 0); // Set lower bank address to 0
    memory.write(0x4000, 0x2); // Set upper bank address to 2
    assert_eq!(memory.read(0x72A7), 0xAB);
  }
}