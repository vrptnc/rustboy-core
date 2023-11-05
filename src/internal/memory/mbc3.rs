use std::cell::{RefCell, RefMut};
use log::info;

use crate::internal::memory::mbc::{Loadable, MBC};
use crate::internal::memory::memory::Memory;
use crate::internal::util::bit_util::{BitUtil, WordUtil};
use crate::memory::{RAMSize, ROMSize};

#[derive(Copy, Clone)]
struct FormattedRTC {
    nanoseconds: u32,
    seconds: u8,
    minutes: u8,
    hours: u8,
    days_low: u8,
    days_high: u8,
}

impl FormattedRTC {
    const DAYS_NANOSECONDS: u64 = 24 * 3600 * 1_000_000_000;
    const HOURS_NANOSECONDS: u64 = 3600 * 1_000_000_000;
    const MINUTES_NANOSECONDS: u64 = 60 * 1_000_000_000;
    const SECONDS_NANOSECONDS: u64 = 1_000_000_000;

    pub fn from_rtc(rtc: &RTC) -> FormattedRTC {
        let mut remaining_nanoseconds = rtc.nanoseconds;
        let days = (remaining_nanoseconds / FormattedRTC::DAYS_NANOSECONDS) as u16;
        remaining_nanoseconds = remaining_nanoseconds % FormattedRTC::DAYS_NANOSECONDS;
        let hours = (remaining_nanoseconds / FormattedRTC::HOURS_NANOSECONDS) as u8;
        remaining_nanoseconds = remaining_nanoseconds % FormattedRTC::HOURS_NANOSECONDS;
        let minutes = (remaining_nanoseconds / FormattedRTC::MINUTES_NANOSECONDS) as u8;
        remaining_nanoseconds = remaining_nanoseconds % FormattedRTC::MINUTES_NANOSECONDS;
        let seconds = (remaining_nanoseconds / FormattedRTC::SECONDS_NANOSECONDS) as u8;
        remaining_nanoseconds = remaining_nanoseconds % FormattedRTC::SECONDS_NANOSECONDS;
        FormattedRTC {
            nanoseconds: remaining_nanoseconds as u32,
            seconds,
            minutes,
            hours,
            days_low: days.get_low_byte(),
            days_high: (days.get_high_byte() & 0x01) | (if rtc.halted { 0x40 } else { 0x00 }) | (if rtc.days_carry { 0x80 } else { 0x00 }),
        }
    }
}

struct RTC {
    nanoseconds: u64,
    days_carry: bool,
    halted: bool,
    formatted_rtc: RefCell<Option<FormattedRTC>>,
}

impl Clone for RTC {
    fn clone(&self) -> Self {
        RTC {
            nanoseconds: self.nanoseconds,
            days_carry: self.days_carry,
            halted: self.halted,
            formatted_rtc: self.formatted_rtc.clone(),
        }
    }
}

impl RTC {
    const MAX_DAYS_IN_NANOSECONDS: u64 = 512 * 24 * 3600 * 1_000_000_000;

    pub fn new() -> RTC {
        RTC {
            nanoseconds: 0,
            days_carry: false,
            halted: false,
            formatted_rtc: RefCell::new(None),
        }
    }

    pub fn update_from_formatted_rtc(&mut self, formatted_rtc: FormattedRTC) {
        self.nanoseconds = formatted_rtc.nanoseconds as u64 +
            formatted_rtc.seconds as u64 * FormattedRTC::SECONDS_NANOSECONDS +
            formatted_rtc.minutes as u64 * FormattedRTC::MINUTES_NANOSECONDS +
            formatted_rtc.hours as u64 * FormattedRTC::HOURS_NANOSECONDS +
            (formatted_rtc.days_low as u64 + if formatted_rtc.days_high.get_bit(0) { 0x100u64 } else { 0x000u64 }) * FormattedRTC::DAYS_NANOSECONDS;
        self.days_carry = formatted_rtc.days_high.get_bit(7);
        self.halted = formatted_rtc.days_high.get_bit(6);
        self.formatted_rtc.replace(Some(formatted_rtc));
    }

    pub fn set_seconds(&mut self, seconds: u8) {
        let mut formatted_rtc = *self.get_formatted_rtc();
        formatted_rtc.seconds = seconds;
        self.update_from_formatted_rtc(formatted_rtc);
    }

    pub fn set_minutes(&mut self, minutes: u8) {
        let mut formatted_rtc = *self.get_formatted_rtc();
        formatted_rtc.minutes = minutes;
        self.update_from_formatted_rtc(formatted_rtc);
    }

    pub fn set_hours(&mut self, hours: u8) {
        let mut formatted_rtc = *self.get_formatted_rtc();
        formatted_rtc.hours = hours;
        self.update_from_formatted_rtc(formatted_rtc);
    }

    pub fn set_days_low(&mut self, days_low: u8) {
        let mut formatted_rtc = *self.get_formatted_rtc();
        formatted_rtc.days_low = days_low;
        self.update_from_formatted_rtc(formatted_rtc);
    }

    pub fn set_days_high(&mut self, days_high: u8) {
        let mut formatted_rtc = *self.get_formatted_rtc();
        formatted_rtc.days_high = days_high;
        self.update_from_formatted_rtc(formatted_rtc);
    }

    pub fn tick(&mut self, nanoseconds: u64) {
        if self.halted {
            return;
        }
        let new_nanoseconds = self.nanoseconds + nanoseconds;
        if new_nanoseconds >= RTC::MAX_DAYS_IN_NANOSECONDS {
            self.nanoseconds = new_nanoseconds % RTC::MAX_DAYS_IN_NANOSECONDS;
            self.days_carry = true;
        } else {
            self.nanoseconds = new_nanoseconds;
        }
        self.formatted_rtc.replace(None);
    }

    pub fn get_formatted_rtc(&self) -> RefMut<FormattedRTC> {
        RefMut::map(self.formatted_rtc.borrow_mut(), |formatted_rtc| formatted_rtc.get_or_insert(FormattedRTC::from_rtc(self)))
    }
}

pub struct MBC3 {
    rtc: RTC,
    rtc_registers: RTC,
    clock_counter_data_latch: bool,
    ram_enabled: bool,
    rom_bank_address: usize,
    ram_bank_address: usize,
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl MBC for MBC3 {
    fn tick(&mut self, double_speed: bool) {
        let passed_nanoseconds = if double_speed { 500 } else { 1000 };
        self.rtc.tick(passed_nanoseconds);
    }
}

impl MBC3 {
    pub fn new(rom_size: ROMSize, ram_size: RAMSize) -> MBC3 {
        info!("Loading new MBC3 cartridge with ROM size {:?} and RAM size {:?}", rom_size, ram_size);
        MBC3 {
            rtc: RTC::new(),
            rtc_registers: RTC::new(),
            clock_counter_data_latch: false,
            ram_enabled: false,
            rom_bank_address: 0x01,
            ram_bank_address: 0x00,
            ram: vec![0; ram_size.bytes()],
            rom: vec![0; rom_size.bytes()],
        }
    }

    fn latch_counter_data(&mut self) {
        self.rtc_registers = self.rtc.clone();
    }
}

impl Memory for MBC3 {
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
                match self.ram_bank_address {
                    0x0..=0x7 => {
                        let address_in_ram = ((address as usize) & 0x1FFF) | (self.ram_bank_address << 13);
                        self.ram[address_in_ram]
                    }
                    0x8 => self.rtc_registers.get_formatted_rtc().seconds,
                    0x9 => self.rtc_registers.get_formatted_rtc().minutes,
                    0xA => self.rtc_registers.get_formatted_rtc().hours,
                    0xB => self.rtc_registers.get_formatted_rtc().days_low,
                    0xC => self.rtc_registers.get_formatted_rtc().days_high,
                    _ => panic!("{:#06x} is not a valid RAM bank address", self.ram_bank_address)
                }
            }
            _ => panic!("Can't read from address {:#06x} on MBC3", address)
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.rom_bank_address = value as usize;
                if self.rom_bank_address == 0 {
                    self.rom_bank_address = 1;
                }
            }
            0x4000..=0x5FFF if value <= 0x0C => {
                self.ram_bank_address = (value & 0x0F) as usize;
            }
            0x6000..=0x7FFF => {
                let new_value = (value & 1u8) == 1;
                if new_value & !self.clock_counter_data_latch {
                    self.latch_counter_data();
                }
                self.clock_counter_data_latch = new_value
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    match self.ram_bank_address {
                        0x0..=0x7 => {
                            let address_in_ram = ((address as usize) & 0x1FFF) | (self.ram_bank_address << 13);
                            self.ram[address_in_ram] = value;
                        }
                        0x8 => {
                            self.rtc_registers.set_seconds(value);
                            self.rtc.set_seconds(value);
                        }
                        0x9 => {
                            self.rtc_registers.set_minutes(value);
                            self.rtc.set_minutes(value);
                        }
                        0xA => {
                            self.rtc_registers.set_hours(value);
                            self.rtc.set_hours(value);
                        }
                        0xB => {
                            self.rtc_registers.set_days_low(value);
                            self.rtc.set_days_low(value);
                        }
                        0xC => {
                            self.rtc_registers.set_days_high(value);
                            self.rtc.set_days_high(value);
                        }
                        _ => panic!("{:#06x} is not a valid RAM bank address", self.ram_bank_address)
                    };
                }
            }
            _ => panic!("Can't write to address {:#06x} on MBC3", address)
        };
    }
}

impl Loadable for MBC3 {
    fn load_byte(&mut self, address: usize, value: u8) {
        self.rom[address] = value;
    }

    fn load_bytes(&mut self, address: usize, values: &[u8]) {
        self.rom.as_mut_slice()[address..((address + values.len()))].copy_from_slice(values);
    }
}

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;

    use super::*;

    #[test]
    fn read_write_ram() {
        let mut memory = MBC3::new(ROMSize::KB256, RAMSize::KB32);
        memory.write(0x0000, 0xA); // Enable RAM
        memory.write(0xA000, 0xAB);
        memory.write(0xA080, 0xCD);
        memory.write(0xA1FF, 0xEF);
        assert_eq_hex!(memory.read(0xA000), 0xAB);
        assert_eq_hex!(memory.read(0xA080), 0xCD);
        assert_eq_hex!(memory.read(0xA1FF), 0xEF);
    }

    #[test]
    fn ram_enabled_register_blocks_writes() {
        let mut memory = MBC3::new(ROMSize::KB256, RAMSize::KB32);
        memory.write(0x0000, 0xA); // Enable RAM
        memory.write(0xA080, 0xAB);
        memory.write(0x0000, 0xB); // Disable RAM
        memory.write(0xA080, 0xCD);
        assert_eq_hex!(memory.read(0xA080), 0xAB);
    }

    #[test]
    fn read_lower_rom() {
        let mut memory = MBC3::new(ROMSize::KB256, RAMSize::KB32);
        memory.load_byte(0x0000, 0x12);
        memory.load_byte(0x2ABC, 0x34);
        memory.load_byte(0x3FFF, 0x56);
        assert_eq_hex!(memory.read(0x0000), 0x12);
        assert_eq_hex!(memory.read(0x2ABC), 0x34);
        assert_eq_hex!(memory.read(0x3FFF), 0x56);
    }

    #[test]
    fn read_upper_rom() {
        let mut memory = MBC3::new(ROMSize::KB256, RAMSize::KB32);
        memory.load_byte(0x4000, 0x12);
        memory.load_byte(0x5ABC, 0x34);
        memory.load_byte(0x7FFF, 0x56);
        memory.load_byte(0x14000, 0x78); // Load bytes into bank 5
        memory.load_byte(0x15ABC, 0x9A);
        memory.load_byte(0x17FFF, 0xBC);
        assert_eq_hex!(memory.read(0x4000), 0x12);
        assert_eq_hex!(memory.read(0x5ABC), 0x34);
        assert_eq_hex!(memory.read(0x7FFF), 0x56);
        memory.write(0x3000, 0x05);
        // Switch to bank 5
        assert_eq_hex!(memory.read(0x4000), 0x78);
        assert_eq_hex!(memory.read(0x5ABC), 0x9A);
        assert_eq_hex!(memory.read(0x7FFF), 0xBC);
    }

    #[test]
    fn rom_bank_address_is_never_zero() {
        let mut memory = MBC3::new(ROMSize::KB256, RAMSize::KB32);
        memory.write(0x3000, 0x00);
        memory.load_byte(0x4000, 0x12);
        memory.load_byte(0x5ABC, 0x34);
        memory.load_byte(0x7FFF, 0x56);
        assert_eq_hex!(memory.read(0x4000), 0x12);
        assert_eq_hex!(memory.read(0x5ABC), 0x34);
        assert_eq_hex!(memory.read(0x7FFF), 0x56);
    }

    #[test]
    fn read_write_rtc() {
        let mut memory = MBC3::new(ROMSize::KB256, RAMSize::KB32);
        memory.write(0x0000, 0xA); // Enable RAM
        memory.write(0x4000, 0x08); // Set RAM bank to RTC seconds
        memory.write(0xA000, 56); // Write 56 seconds
        memory.write(0x4000, 0x09); // Set RAM bank to RTC minutes
        memory.write(0xA000, 34); // Write 34 minutes
        memory.write(0x4000, 0x0A); // Set RAM bank to RTC hours
        memory.write(0xA000, 12); // Write 12 hours
        memory.write(0x4000, 0x0B); // Set RAM bank to RTC days low
        memory.write(0xA000, 105); // Write 105 days low
        memory.write(0x4000, 0x0C); // Set RAM bank to RTC days high
        memory.write(0xA000, 0x81); // Write 768 days high (non-halted)
        memory.write(0x0000, 0xB); // Disable RAM
        memory.write(0x4000, 0x08); // Set RAM bank to RTC seconds
        assert_eq!(memory.read(0xA000), 56); // Read seconds
        memory.write(0x4000, 0x09); // Set RAM bank to RTC minutes
        assert_eq!(memory.read(0xA000), 34); // Read minutes
        memory.write(0x4000, 0x0A); // Set RAM bank to RTC hours
        assert_eq!(memory.read(0xA000), 12); // Read hours
        memory.write(0x4000, 0x0B); // Set RAM bank to RTC days low
        assert_eq!(memory.read(0xA000), 105); // Read days low
        memory.write(0x4000, 0x0C);
        // Set RAM bank to RTC days high
        assert_eq_hex!(memory.read(0xA000), 0x81); // Read days high (non-halted)
    }

    #[test]
    fn tick_rtc() {
        let mut memory = MBC3::new(ROMSize::KB256, RAMSize::KB32);
        memory.write(0x0000, 0xA); // Enable RAM
        memory.write(0x4000, 0x08); // Set RAM bank to RTC seconds
        memory.write(0xA000, 59); // Write 59 seconds
        memory.write(0x4000, 0x09); // Set RAM bank to RTC minutes
        memory.write(0xA000, 59); // Write 59 minutes
        memory.write(0x4000, 0x0A); // Set RAM bank to RTC hours
        memory.write(0xA000, 23); // Write 23 hours

        memory.write(0x4000, 0x0B); // Set RAM bank to RTC days low
        memory.write(0xA000, 0xFF); // Write 512 days
        memory.write(0x4000, 0x0C); // Set RAM bank to RTC days high
        memory.write(0xA000, 0x01); // Write 512 days (non-halted, no carry)
        memory.write(0x0000, 0xB); // Disable RAM
        // Tick a full second (1 tick = 1 microsecond)
        for _ in 0..1_000_000usize {
            memory.tick(false);
        }
        memory.tick(false);
        memory.write(0x4000, 0x08); // Set RAM bank to RTC seconds
        assert_eq!(memory.read(0xA000), 59); // Read seconds
        memory.write(0x4000, 0x09); // Set RAM bank to RTC minutes
        assert_eq!(memory.read(0xA000), 59); // Read minutes
        memory.write(0x4000, 0x0A); // Set RAM bank to RTC hours
        assert_eq!(memory.read(0xA000), 23); // Read hours
        memory.write(0x4000, 0x0B); // Set RAM bank to RTC days low
        assert_eq!(memory.read(0xA000), 0xFF); // Read days low
        memory.write(0x4000, 0x0C);
        // Set RAM bank to RTC days high
        assert_eq_hex!(memory.read(0xA000), 0x01); // Read days high (non-halted, no carry)

        memory.write(0x6000, 0x00);
        memory.write(0x6000, 0x01);
        memory.write(0x4000, 0x08); // Set RAM bank to RTC seconds
        assert_eq!(memory.read(0xA000), 0); // Read seconds
        memory.write(0x4000, 0x09); // Set RAM bank to RTC minutes
        assert_eq!(memory.read(0xA000), 0); // Read minutes
        memory.write(0x4000, 0x0A); // Set RAM bank to RTC hours
        assert_eq!(memory.read(0xA000), 0); // Read hours
        memory.write(0x4000, 0x0B); // Set RAM bank to RTC days low
        assert_eq!(memory.read(0xA000), 0); // Read days low
        memory.write(0x4000, 0x0C);
        // Set RAM bank to RTC days high
        assert_eq_hex!(memory.read(0xA000), 0x80); // Read days high (non-halted, carry enabled)
    }
}