use num::cast::AsPrimitive;

const BYTE_REVERSAL_TABLE: [u8; 0x100] = [
  0x00, 0x80, 0x40, 0xC0, 0x20, 0xA0, 0x60, 0xE0,
  0x10, 0x90, 0x50, 0xD0, 0x30, 0xB0, 0x70, 0xF0,
  0x08, 0x88, 0x48, 0xC8, 0x28, 0xA8, 0x68, 0xE8,
  0x18, 0x98, 0x58, 0xD8, 0x38, 0xB8, 0x78, 0xF8,
  0x04, 0x84, 0x44, 0xC4, 0x24, 0xA4, 0x64, 0xE4,
  0x14, 0x94, 0x54, 0xD4, 0x34, 0xB4, 0x74, 0xF4,
  0x0C, 0x8C, 0x4C, 0xCC, 0x2C, 0xAC, 0x6C, 0xEC,
  0x1C, 0x9C, 0x5C, 0xDC, 0x3C, 0xBC, 0x7C, 0xFC,
  0x02, 0x82, 0x42, 0xC2, 0x22, 0xA2, 0x62, 0xE2,
  0x12, 0x92, 0x52, 0xD2, 0x32, 0xB2, 0x72, 0xF2,
  0x0A, 0x8A, 0x4A, 0xCA, 0x2A, 0xAA, 0x6A, 0xEA,
  0x1A, 0x9A, 0x5A, 0xDA, 0x3A, 0xBA, 0x7A, 0xFA,
  0x06, 0x86, 0x46, 0xC6, 0x26, 0xA6, 0x66, 0xE6,
  0x16, 0x96, 0x56, 0xD6, 0x36, 0xB6, 0x76, 0xF6,
  0x0E, 0x8E, 0x4E, 0xCE, 0x2E, 0xAE, 0x6E, 0xEE,
  0x1E, 0x9E, 0x5E, 0xDE, 0x3E, 0xBE, 0x7E, 0xFE,
  0x01, 0x81, 0x41, 0xC1, 0x21, 0xA1, 0x61, 0xE1,
  0x11, 0x91, 0x51, 0xD1, 0x31, 0xB1, 0x71, 0xF1,
  0x09, 0x89, 0x49, 0xC9, 0x29, 0xA9, 0x69, 0xE9,
  0x19, 0x99, 0x59, 0xD9, 0x39, 0xB9, 0x79, 0xF9,
  0x05, 0x85, 0x45, 0xC5, 0x25, 0xA5, 0x65, 0xE5,
  0x15, 0x95, 0x55, 0xD5, 0x35, 0xB5, 0x75, 0xF5,
  0x0D, 0x8D, 0x4D, 0xCD, 0x2D, 0xAD, 0x6D, 0xED,
  0x1D, 0x9D, 0x5D, 0xDD, 0x3D, 0xBD, 0x7D, 0xFD,
  0x03, 0x83, 0x43, 0xC3, 0x23, 0xA3, 0x63, 0xE3,
  0x13, 0x93, 0x53, 0xD3, 0x33, 0xB3, 0x73, 0xF3,
  0x0B, 0x8B, 0x4B, 0xCB, 0x2B, 0xAB, 0x6B, 0xEB,
  0x1B, 0x9B, 0x5B, 0xDB, 0x3B, 0xBB, 0x7B, 0xFB,
  0x07, 0x87, 0x47, 0xC7, 0x27, 0xA7, 0x67, 0xE7,
  0x17, 0x97, 0x57, 0xD7, 0x37, 0xB7, 0x77, 0xF7,
  0x0F, 0x8F, 0x4F, 0xCF, 0x2F, 0xAF, 0x6F, 0xEF,
  0x1F, 0x9F, 0x5F, 0xDF, 0x3F, 0xBF, 0x7F, 0xFF
];

pub struct UnsignedCrumbIterator<T, const BITS: u32> where
  T: AsPrimitive<u8> + Copy + Clone + std::ops::Shr<u32, Output=T> {
  value: T,
  current_bit: u32,
  current_last_bit: u32,
}

impl<T, const BITS: u32> UnsignedCrumbIterator<T, BITS> where
  T: AsPrimitive<u8> + Copy + Clone + std::ops::Shr<u32, Output=T> {
  pub fn new(value: T) -> UnsignedCrumbIterator<T, BITS> {
    UnsignedCrumbIterator {
      value,
      current_bit: 0,
      current_last_bit: BITS,
    }
  }
}

impl<T, const BITS: u32> Iterator for UnsignedCrumbIterator<T, BITS> where
  T: AsPrimitive<u8> + Copy + Clone + std::ops::Shr<u32, Output=T> {
  type Item = u8;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current_bit >= BITS {
      None
    } else {
      let result = ((self.value >> self.current_bit).as_()) & 0x3;
      self.current_bit += 2;
      Some(result)
    }
  }
}

impl<T, const BITS: u32> DoubleEndedIterator for UnsignedCrumbIterator<T, BITS> where
  T: AsPrimitive<u8> + Copy + Clone + std::ops::Shr<u32, Output=T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.current_last_bit == 0 {
      None
    } else {
      self.current_last_bit -= 2;
      let result = ((self.value >> self.current_last_bit).as_()) & 0x3;
      Some(result as u8)
    }
  }
}

pub trait BitUtil {
  type CrumbIterator: DoubleEndedIterator<Item=u8>;

  fn compose(bits: &[(bool, u8)]) -> Self;
  fn get_bit(&self, bit: u8) -> bool;
  fn set_bit(&self, bit: u8) -> Self;
  fn toggle_bit(&self, bit: u8) -> Self;
  fn reset_bit(&self, bit: u8) -> Self;
  fn get_lower_byte(&self) -> u8;
  fn get_upper_byte(&self) -> u8;
  fn crumbs(&self) -> Self::CrumbIterator;
}

pub trait ByteUtil {
  fn interleave_with(&self, byte: u8) -> u16;
  fn reverse(&self) -> u8;
}

pub trait WordUtil {
  fn get_high_byte(&self) -> u8;
  fn get_low_byte(&self) -> u8;
  fn set_high_byte(&self, byte: u8) -> Self;
  fn set_low_byte(&self, byte: u8) -> Self;
}

impl BitUtil for u8 {
  type CrumbIterator = UnsignedCrumbIterator<u8, 8>;

  fn compose(bits: &[(bool, u8)]) -> Self {
    bits.iter().map(|a| {
      (a.0 as u8) << a.1
    }).reduce(|a, b| {
      a | b
    }).unwrap()
  }

  fn get_bit(&self, bit: u8) -> bool {
    (self & (1u8 << bit)) != 0
  }

  fn set_bit(&self, bit: u8) -> Self {
    self | (1u8 << bit)
  }

  fn toggle_bit(&self, bit: u8) -> Self {
    self ^ (1u8 << bit)
  }

  fn reset_bit(&self, bit: u8) -> Self {
    self & !(1u8 << bit)
  }

  fn get_lower_byte(&self) -> u8 {
    *self
  }

  fn get_upper_byte(&self) -> u8 {
    0
  }

  fn crumbs(&self) -> Self::CrumbIterator {
    UnsignedCrumbIterator::new(*self)
  }
}

impl ByteUtil for u8 {
  fn interleave_with(&self, byte: u8) -> u16 {
    let mut x = *self as u16;
    let mut y = byte as u16;
    x = (x | (x << 4)) & 0x0F0F;
    y = (y | (y << 4)) & 0x0F0F;
    x = (x | (x << 2)) & 0x3333;
    y = (y | (y << 2)) & 0x3333;
    x = (x | (x << 1)) & 0x5555;
    y = (y | (y << 1)) & 0x5555;
    y = y << 1;
    x | y
  }

  fn reverse(&self) -> u8 {
    BYTE_REVERSAL_TABLE[*self as usize]
  }
}

impl BitUtil for u16 {
  type CrumbIterator = UnsignedCrumbIterator<u16, 16>;

  fn compose(bits: &[(bool, u8)]) -> Self {
    bits.iter().map(|a| {
      (a.0 as u16) << a.1
    }).reduce(|a, b| {
      a | b
    }).unwrap()
  }

  fn get_bit(&self, bit: u8) -> bool {
    (self & (1u16 << bit)) != 0
  }

  fn set_bit(&self, bit: u8) -> Self {
    self | (1u16 << bit)
  }

  fn reset_bit(&self, bit: u8) -> Self {
    self & !(1u16 << bit)
  }

  fn toggle_bit(&self, bit: u8) -> Self {
    self ^ (1u16 << bit)
  }

  fn get_lower_byte(&self) -> u8 {
    *self as u8
  }

  fn get_upper_byte(&self) -> u8 {
    (*self >> 8) as u8
  }

  fn crumbs(&self) -> Self::CrumbIterator {
    UnsignedCrumbIterator::new(*self)
  }
}

impl WordUtil for u16 {
  fn get_high_byte(&self) -> u8 {
    (self >> 8) as u8
  }

  fn get_low_byte(&self) -> u8 {
    *self as u8
  }

  fn set_high_byte(&self, byte: u8) -> Self {
    (self & 0x00FF) | ((byte as u16) << 8)
  }

  fn set_low_byte(&self, byte: u8) -> Self {
    (0xFF00 & self) | (byte as u16)
  }
}

impl BitUtil for usize {
  type CrumbIterator = UnsignedCrumbIterator<usize, { usize::BITS }>;

  fn compose(bits: &[(bool, u8)]) -> Self {
    bits.iter().map(|a| {
      (a.0 as usize) << a.1
    }).reduce(|a, b| {
      a | b
    }).unwrap()
  }

  fn get_bit(&self, bit: u8) -> bool {
    (self & (1usize << bit)) != 0
  }

  fn set_bit(&self, bit: u8) -> Self {
    self | (1usize << bit)
  }

  fn reset_bit(&self, bit: u8) -> Self {
    self & !(1usize << bit)
  }

  fn toggle_bit(&self, bit: u8) -> Self {
    self ^ (1usize << bit)
  }

  fn get_lower_byte(&self) -> u8 {
    *self as u8
  }

  fn get_upper_byte(&self) -> u8 {
    (*self >> 8) as u8
  }

  fn crumbs(&self) -> Self::CrumbIterator {
    UnsignedCrumbIterator::new(*self)
  }
}

#[cfg(test)]
mod tests {
  use assert_hex::assert_eq_hex;

  use super::*;

  #[test]
  fn interleave_bytes() {
    let x: u8 = 0x7C; // 0111 1100
    let y: u8 = 0x56; // 0101 0110 => 0011 0111 0111 1000

    assert_eq_hex!(x.interleave_with(y), 0x3778u16);
  }

  #[test]
  fn interleave_and_iterate_over_crumbs() {
    let byte1 = 0x3C; // 0011 1100
    let byte2 = 0x7E; // 0111 1110 => 0010 1111 1111 1000
    let interleaved = byte1.interleave_with(byte2);
    assert_eq_hex!(interleaved, 0x2FF8);
    let bytes: Vec<u8> = interleaved.crumbs().rev().collect();
    assert_eq_hex!(bytes[0], 0x00);
    assert_eq_hex!(bytes[1], 0x02);
    assert_eq_hex!(bytes[2], 0x03);
    assert_eq_hex!(bytes[3], 0x03);
    assert_eq_hex!(bytes[4], 0x03);
    assert_eq_hex!(bytes[5], 0x03);
    assert_eq_hex!(bytes[6], 0x02);
    assert_eq_hex!(bytes[7], 0x00);
  }

  #[test]
  fn toggle_bit() {
      assert_eq_hex!(0xFFu8.toggle_bit(5), 0xDF);
      assert_eq_hex!(0x00u8.toggle_bit(5), 0x20);
  }
}