use crate::memory::{CartridgeType, CGBMode, Licensee};
use crate::memory::{RAMSize, ROMSize};

pub struct CartridgeInfo {
  pub title: String,
  pub title_checksum: u8,
  pub licensee: Licensee,
  pub cartridge_type: CartridgeType,
  pub rom_size: ROMSize,
  pub ram_size: RAMSize,
  pub cgb_mode: CGBMode,
}

impl CartridgeInfo {

  pub fn from_bytes(rom_bytes: &[u8]) -> Self {
    CartridgeInfo {
      title: CartridgeInfo::read_title(rom_bytes),
      title_checksum: CartridgeInfo::calculate_title_checksum(rom_bytes),
      licensee: Licensee::from_bytes(rom_bytes),
      cartridge_type: CartridgeType::from_byte(rom_bytes[0x0147]),
      rom_size: ROMSize::from_byte(rom_bytes[0x0148]),
      ram_size: RAMSize::from_byte(rom_bytes[0x0149]),
      cgb_mode: CGBMode::from_byte(rom_bytes[0x0143]),
    }
  }

  fn read_title(rom_bytes: &[u8]) -> String {
    let mut title = String::new();
    (0x134..=0x143)
      .map(|index| rom_bytes[index])
      .take_while(|byte| *byte != 0x00)
      .map(|byte| byte as char)
      .for_each(|character| title.push(character));
    title
  }

  fn calculate_title_checksum(rom_bytes: &[u8]) -> u8 {
    (0x134..=0x143)
      .map(|index| rom_bytes[index])
      .reduce(|checksum, byte| checksum.wrapping_add(byte))
      .unwrap_or(0u8)
  }

  pub fn get_title(&self) -> &str {
    self.title.as_str()
  }
}
