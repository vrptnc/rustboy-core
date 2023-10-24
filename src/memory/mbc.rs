use mockall::mock;

use crate::memory::memory::{CGBMode, Memory};

pub trait Loadable {
  fn load_byte(&mut self, address: usize, value: u8);
  fn load_bytes(&mut self, address: usize, values: &[u8]);
}

pub trait MBC: Memory + Loadable {
  fn compatibility_byte(&self) -> u8 {
    self.read(0x0143)
  }

  fn is_licensed_by_nintendo(&self) -> bool {
    let old_licensee_code = self.read(0x014B);
    let licensee_code = if old_licensee_code == 0x33 {
      let upper = (self.read(0x0144) as char).to_digit(16).unwrap() as u8;
      let lower = (self.read(0x0145) as char).to_digit(16).unwrap() as u8;
      let new_licensee_code = (upper << 4) | lower;
      new_licensee_code
    } else {
      old_licensee_code
    };
    licensee_code == 0x01
  }

  fn fourth_title_letter(&self) -> u8 {
    self.read(0x0137)
  }

  fn title_checksum(&self) -> u8 {
    (0x0134u16..=0x0143).map(|address| self.read(address))
      .reduce(|checksum, value| checksum.wrapping_add(value)).unwrap_or(0)
  }

  fn cgb_mode(&self) -> CGBMode {
    CGBMode::from_byte(self.read(0x0143))
  }

  fn tick(&mut self, _double_speed: bool) {

  }
}

mock! {
  pub ROM {}

  impl MBC for ROM {
    fn compatibility_byte(&self) -> u8;
    fn is_licensed_by_nintendo(&self) -> bool;
    fn fourth_title_letter(&self) -> u8;
    fn title_checksum(&self) -> u8;
    fn cgb_mode(&self) -> CGBMode;
  }

  impl Loadable for ROM {
      fn load_byte(&mut self, address: usize, value: u8);
      fn load_bytes(&mut self, address: usize, values: &[u8]);
  }

  impl Memory for ROM {
      fn read(&self, address: u16) -> u8;
      fn write(&mut self, address: u16, value: u8);
  }
}

#[cfg(test)]
mod tests {
  use assert_hex::assert_eq_hex;

  #[test]
  fn test_byte_parsing() {
    let first = 0x31u8;
    let second = 0x32u8;
    let upper = (first as char).to_digit(16).unwrap() as u8;
    let lower = (second as char).to_digit(16).unwrap() as u8;
    let result = (upper << 4) | (lower);
    assert_eq_hex!(0x12, result);
  }
}