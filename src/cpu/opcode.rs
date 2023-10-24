#[derive(Copy, Clone)]
pub struct Opcode(pub u8);

// Opcode bit structure: xxyy yzzz
// Opcode bit structure: **dd ****
// Opcode bit structure: ***c c***
impl Opcode {

  pub fn value(&self) -> u8 {
    self.0
  }

  pub fn y_bits(&self) -> u8 {
    self.0 >> 3 & 7
  }

  pub fn z_bits(&self) -> u8 {
    self.0 & 7
  }

  pub fn cc_bits(&self) -> u8 {
    self.0 >> 3 & 3
  }

  pub fn dd_bits(&self) -> u8 {
    self.0 >> 4 & 3
  }

  pub fn qq_bits(&self) -> u8 {
    self.0 >> 4 & 3
  }
}