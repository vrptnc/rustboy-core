use crate::internal::memory::mbc::{Loadable, MBC};
use crate::internal::memory::memory::{Memory, ROMSize};

pub struct MBC0 {
    rom: Vec<u8>,
}

impl MBC for MBC0 {}

impl MBC0 {
    pub fn new(rom_size: ROMSize) -> MBC0 {
        MBC0 {
            rom: vec![0; rom_size.bytes()],
        }
    }
}

impl Memory for MBC0 {
    fn read(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.rom[address as usize] = value;
    }
}

impl Loadable for MBC0 {
    fn load_byte(&mut self, address: usize, value: u8) {
        self.rom[address] = value;
    }

    fn load_bytes(&mut self, address: usize, values: &[u8]) {
        self.rom.as_mut_slice()[address..(address + values.len())].copy_from_slice(values);
    }
}