use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum WordRegister {
  AF,
  BC,
  DE,
  HL,
  PC,
  SP,
}

impl WordRegister {
  fn offset(&self) -> usize {
    match self {
      WordRegister::AF => 0,
      WordRegister::BC => 2,
      WordRegister::DE => 4,
      WordRegister::HL => 6,
      WordRegister::PC => 8,
      WordRegister::SP => 10
    }
  }

  // Also works for ss bits
  pub fn from_dd_bits(bits: u8) -> Self {
    match bits {
      0b00 => WordRegister::BC,
      0b01 => WordRegister::DE,
      0b10 => WordRegister::HL,
      0b11 => WordRegister::SP,
      _ => panic!("{} doesn't map to a register pair", bits)
    }
  }

  pub fn from_qq_bits(bits: u8) -> Self {
    match bits {
      0b00 => WordRegister::BC,
      0b01 => WordRegister::DE,
      0b10 => WordRegister::HL,
      0b11 => WordRegister::AF,
      _ => panic!("{} doesn't map to a register pair", bits)
    }
  }

  pub fn get_upper_byte_register(&self) -> ByteRegister {
    match self {
      WordRegister::AF => ByteRegister::A,
      WordRegister::BC => ByteRegister::B,
      WordRegister::DE => ByteRegister::D,
      WordRegister::HL => ByteRegister::UpperHL,
      WordRegister::PC => ByteRegister::UpperPC,
      WordRegister::SP => ByteRegister::UpperSP
    }
  }

  pub fn get_lower_byte_register(&self) -> ByteRegister {
    match self {
      WordRegister::AF => ByteRegister::F,
      WordRegister::BC => ByteRegister::C,
      WordRegister::DE => ByteRegister::E,
      WordRegister::HL => ByteRegister::LowerHL,
      WordRegister::PC => ByteRegister::LowerPC,
      WordRegister::SP => ByteRegister::LowerSP
    }
  }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ByteRegister {
  A,
  F,
  // Z | N | H | CY | x | x | x | x    Z: 1 if result was 0, N: 1 if previous op was subtraction, H: carry from bit 3, CY: carry from bit 7
  B,
  C,
  D,
  E,
  UpperHL,
  LowerHL,
  UpperPC,
  LowerPC,
  UpperSP,
  LowerSP,
}

impl ByteRegister {
  fn offset(&self) -> usize {
    match self {
      ByteRegister::A => 0,
      ByteRegister::F => 1,
      ByteRegister::B => 2,
      ByteRegister::C => 3,
      ByteRegister::D => 4,
      ByteRegister::E => 5,
      ByteRegister::UpperHL => 6,
      ByteRegister::LowerHL => 7,
      ByteRegister::UpperPC => 8,
      ByteRegister::LowerPC => 9,
      ByteRegister::UpperSP => 10,
      ByteRegister::LowerSP => 11,
    }
  }

  pub fn from_r_bits(bits: u8) -> ByteRegister {
    match bits {
      0b111 => ByteRegister::A,
      0b000 => ByteRegister::B,
      0b001 => ByteRegister::C,
      0b010 => ByteRegister::D,
      0b011 => ByteRegister::E,
      0b100 => ByteRegister::UpperHL,
      0b101 => ByteRegister::LowerHL,
      _ => panic!("{} doesn't map to a register", bits)
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct Registers([u8; 12]);

impl Registers {
  pub fn new() -> Registers {
    Registers([
      0x11, // A
      0x80, // F
      0, // B
      0, // C
      0, // D
      0, // E
      0, // H
      0, // L
      0, // P
      0, // C
      0, // S
      0  // P
    ])
  }

  pub fn read_byte(&self, register: ByteRegister) -> u8 {
    self.0[register.offset()]
  }

  pub fn write_byte(&mut self, register: ByteRegister, value: u8) {
    self.0[register.offset()] = value;
  }

  pub fn write_byte_masked(&mut self, register: ByteRegister, value: u8, mask: u8) {
    self.0[register.offset()] = (!mask & self.0[register.offset()]) | (mask & value);
  }

  pub fn read_word(&self, register: WordRegister) -> u16 {
    (&self.0[register.offset()..]).read_u16::<BigEndian>().unwrap()
  }

  pub fn write_word(&mut self, register: WordRegister, value: u16) {
    (&mut self.0[register.offset()..]).write_u16::<BigEndian>(value).unwrap();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn read_write_byte() {
    let mut registers = Registers::new();
    registers.write_byte(ByteRegister::B, 0xAB);
    assert_eq!(registers.read_byte(ByteRegister::B), 0xAB);
  }

  #[test]
  fn read_write_word() {
    let mut registers = Registers::new();
    registers.write_word(WordRegister::BC, 0xABCD);
    assert_eq!(registers.read_word(WordRegister::BC), 0xABCD);
    assert_eq!(registers.read_byte(ByteRegister::C), 0xCD);
    assert_eq!(registers.read_byte(ByteRegister::B), 0xAB);
  }
}
