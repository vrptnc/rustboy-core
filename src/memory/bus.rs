use std::cell::RefCell;
use std::rc::Rc;

use crate::memory::mbc::MBC;
use crate::memory::memory::Memory;

pub struct MemoryBus<'a> {
  pub rom: Rc<RefCell<dyn MBC>>,
  pub vram: &'a mut dyn Memory,
  pub wram: &'a mut dyn Memory,
  pub reserved_area_1: &'a mut dyn Memory,
  pub oam: &'a mut dyn Memory,
  pub reserved_area_2: &'a mut dyn Memory,
  pub button_controller: &'a mut dyn Memory,
  pub timer: &'a mut dyn Memory,
  pub interrupt_controller: &'a mut dyn Memory,
  pub speed_controller: &'a mut dyn Memory,
  pub audio_controller: &'a mut dyn Memory,
  pub lcd: &'a mut dyn Memory,
  pub dma: &'a mut dyn Memory,
  pub cram: &'a mut dyn Memory,
  pub control_registers: &'a mut dyn Memory,
  pub stack: &'a mut dyn Memory,
  pub unmapped_memory: &'a mut dyn Memory
}

impl<'a> Memory for MemoryBus<'a> {
  fn read(&self, address: u16) -> u8 {
    match address {
      0x0000..=0x7FFF => (*self.rom).borrow().read(address),
      0x8000..=0x9FFF => self.vram.read(address),
      0xA000..=0xBFFF => (*self.rom).borrow().read(address),
      0xC000..=0xDFFF => self.wram.read(address),
      0xE000..=0xFDFF => self.reserved_area_1.read(address),
      0xFE00..=0xFE9F => self.oam.read(address),
      0xFEA0..=0xFEA1 => self.interrupt_controller.read(address),
      0xFEA2..=0xFEFF => self.reserved_area_2.read(address),
      0xFF00 => self.button_controller.read(address),
      0xFF01..=0xFF02 => 0, // TODO: implement serial transfer
      0xFF03 => self.unmapped_memory.read(address),
      0xFF04..=0xFF07 => self.timer.read(address),
      0xFF08..=0xFF0E => self.unmapped_memory.read(address),
      0xFF0F => self.interrupt_controller.read(address),
      0xFF10..=0xFF26 => self.audio_controller.read(address),
      0xFF27..=0xFF2F => self.unmapped_memory.read(address),
      0xFF30..=0xFF3F => self.audio_controller.read(address),
      0xFF40..=0xFF45 => self.lcd.read(address),
      0xFF46 => self.dma.read(address),
      0xFF47..=0xFF49 => self.cram.read(address),
      0xFF4A..=0xFF4B => self.lcd.read(address),
      0xFF4C => self.control_registers.read(address),
      0xFF4D => self.speed_controller.read(address),
      0xFF4E => self.unmapped_memory.read(address),
      0xFF4F => self.vram.read(address),
      0xFF50 => self.control_registers.read(address),
      0xFF51..=0xFF55 => self.dma.read(address),
      0xFF56 => 0, // TODO: implement infrared transfer
      0xFF57..=0xFF67 => self.unmapped_memory.read(address),
      0xFF68..=0xFF6B => self.cram.read(address),
      0xFF6C => self.lcd.read(address),
      0xFF6D..=0xFF6F => self.unmapped_memory.read(address),
      0xFF70 => self.wram.read(address),
      0xFF71..=0xFF7F => self.unmapped_memory.read(address),
      0xFF80..=0xFFFE => self.stack.read(address),
      0xFFFF => self.interrupt_controller.read(0xFFFF),
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      0x0000..=0x7FFF => (*self.rom).borrow_mut().write(address, value),
      0x8000..=0x9FFF => self.vram.write(address, value),
      0xA000..=0xBFFF => (*self.rom).borrow_mut().write(address, value),
      0xC000..=0xDFFF => self.wram.write(address, value),
      0xE000..=0xFDFF => self.reserved_area_1.write(address, value),
      0xFE00..=0xFE9F => self.oam.write(address, value),
      0xFEA0 => self.interrupt_controller.write(address, value),
      0xFEA1..=0xFEFF => self.reserved_area_2.write(address, value),
      0xFF00 => self.button_controller.write(address, value),
      0xFF01..=0xFF02 => {}, // Serial communication not implemented (yet)
      0xFF03 => self.unmapped_memory.write(address, value),
      0xFF04..=0xFF07 => self.timer.write(address, value),
      0xFF08..=0xFF0E => self.unmapped_memory.write(address, value),
      0xFF0F => self.interrupt_controller.write(address, value),
      0xFF10..=0xFF26 => self.audio_controller.write(address, value),
      0xFF27..=0xFF2F => self.unmapped_memory.write(address, value),
      0xFF30..=0xFF3F => self.audio_controller.write(address, value),
      0xFF40..=0xFF45 => self.lcd.write(address, value),
      0xFF46 => self.dma.write(address, value),
      0xFF47..=0xFF49 => self.cram.write(address, value),
      0xFF4A..=0xFF4B => self.lcd.write(address, value),
      0xFF4C => self.control_registers.write(address, value),
      0xFF4D => self.speed_controller.write(address, value),
      0xFF4E => self.unmapped_memory.write(address, value),
      0xFF4F => self.vram.write(address, value),
      0xFF50 => self.control_registers.write(address, value),
      0xFF51..=0xFF55 => self.dma.write(address, value),
      0xFF56 => {}, // Infrared communication not implemented (yet)
      0xFF57..=0xFF67 => self.unmapped_memory.write(address, value),
      0xFF68..=0xFF6B => self.cram.write(address, value),
      0xFF6C => self.lcd.write(address, value),
      0xFF6D..=0xFF6F => self.unmapped_memory.write(address, value),
      0xFF70 => self.wram.write(address, value),
      0xFF71..=0xFF7F => self.unmapped_memory.write(address, value),
      0xFF80..=0xFFFE => self.stack.write(address, value),
      0xFFFF => self.interrupt_controller.write(address, value)
    }
  }
}