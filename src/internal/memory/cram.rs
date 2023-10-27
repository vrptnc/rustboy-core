use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::internal::util::compatibility_palette::CompatibilityPalettes;
use crate::internal::memory::memory::{Memory, MemoryAddress};
use crate::renderer::Color;
use crate::internal::util::bit_util::BitUtil;

const COLORS_PER_PALETTE: usize = 4;
const NUMBER_OF_PALETTES: usize = 8;

#[derive(Copy, Clone)]
pub struct ColorReference {
  pub color_index: u8,
  pub palette_index: u8,
  pub foreground: bool,
}

#[automock]
pub trait CRAM {
  fn write_compatibility_palettes(&mut self, compatibility_palettes: CompatibilityPalettes);
  fn monochrome_background_color(&self, color_ref: ColorReference) -> Color;
  fn background_color(&self, color_ref: ColorReference) -> Color;
  fn monochrome_object_color(&self, color_ref: ColorReference) -> Color;
  fn object_color(&self, color_ref: ColorReference) -> Color;
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct CRAMImpl {
  monochrome_background_palette: u8,
  monochrome_object_palette_0: u8,
  monochrome_object_palette_1: u8,
  background_palette_index: u8,
  #[serde_as(as = "[_;64]")]
  background_palettes: [u8; 2 * COLORS_PER_PALETTE * NUMBER_OF_PALETTES],
  object_palette_index: u8,
  #[serde_as(as = "[_;64]")]
  object_palettes: [u8; 2 * COLORS_PER_PALETTE * NUMBER_OF_PALETTES],
}

impl CRAMImpl {
  pub fn new() -> CRAMImpl {
    CRAMImpl {
      monochrome_background_palette: 0xFC,
      monochrome_object_palette_0: 0xE4,
      monochrome_object_palette_1: 0xE4,
      background_palette_index: 0,
      background_palettes: [0; 2 * COLORS_PER_PALETTE * NUMBER_OF_PALETTES],
      object_palette_index: 0,
      object_palettes: [0; 2 * COLORS_PER_PALETTE * NUMBER_OF_PALETTES],
    }
  }
}

impl CRAM for CRAMImpl {
  fn write_compatibility_palettes(&mut self, compatibility_palettes: CompatibilityPalettes) {
    compatibility_palettes.bgp.into_iter()
      .enumerate()
      .for_each(|(color_index, color)| {
        (&mut self.background_palettes[(2 * color_index)..]).write_u16::<LittleEndian>(color.to_word()).unwrap();
      });
    compatibility_palettes.obj0.into_iter()
      .chain(compatibility_palettes.obj1.into_iter())
      .enumerate()
      .for_each(|(color_index, color)| {
        (&mut self.object_palettes[(2 * color_index)..]).write_u16::<LittleEndian>(color.to_word()).unwrap();
      });
  }

  fn monochrome_background_color(&self, color_ref: ColorReference) -> Color {
    let real_color_index = (self.monochrome_background_palette >> (2 * color_ref.color_index)) & 0x3;
    self.background_color(ColorReference {
      palette_index: color_ref.palette_index,
      color_index: real_color_index,
      foreground: color_ref.foreground,
    })
  }

  fn background_color(&self, color_ref: ColorReference) -> Color {
    let lower_byte_address = (8 * color_ref.palette_index + 2 * color_ref.color_index) as usize;
    let color_word = (&self.background_palettes[lower_byte_address..=lower_byte_address + 1]).read_u16::<LittleEndian>().unwrap();
    Color::from_word(color_word)
  }

  fn monochrome_object_color(&self, color_ref: ColorReference) -> Color {
    if color_ref.color_index == 0 {
      return Color::transparent();
    }
    let real_color_index = (if color_ref.palette_index == 0 { self.monochrome_object_palette_0 } else { self.monochrome_object_palette_1 } >> (2 * color_ref.color_index)) & 0x3;
    let lower_byte_address = (8 * color_ref.palette_index + 2 * real_color_index) as usize;
    let color_word = (&self.object_palettes[lower_byte_address..=lower_byte_address + 1]).read_u16::<LittleEndian>().unwrap();
    Color::from_word(color_word)
  }

  fn object_color(&self, color_ref: ColorReference) -> Color {
    if color_ref.color_index == 0 {
      return Color::transparent();
    }
    let lower_byte_address = (8 * color_ref.palette_index + 2 * color_ref.color_index) as usize;
    let color_word = (&self.object_palettes[lower_byte_address..=lower_byte_address + 1]).read_u16::<LittleEndian>().unwrap();
    Color::from_word(color_word)
  }
}

impl Memory for CRAMImpl {
  fn read(&self, address: u16) -> u8 {
    match address {
      MemoryAddress::BGP => self.monochrome_background_palette,
      MemoryAddress::OBP0 => self.monochrome_object_palette_0,
      MemoryAddress::OBP1 => self.monochrome_object_palette_1,
      MemoryAddress::BCPS => self.background_palette_index,
      MemoryAddress::BCPD => self.background_palettes[(self.background_palette_index & 0x3F) as usize],
      MemoryAddress::OCPS => self.object_palette_index,
      MemoryAddress::OCPD => self.object_palettes[(self.object_palette_index & 0x3F) as usize],
      _ => panic!("Unable to read address {:#x} from CRAM", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      MemoryAddress::BGP => self.monochrome_background_palette = value,
      MemoryAddress::OBP0 => self.monochrome_object_palette_0 = value,
      MemoryAddress::OBP1 => self.monochrome_object_palette_1 = value,
      MemoryAddress::BCPS => self.background_palette_index = value & 0xBF,
      MemoryAddress::BCPD => {
        self.background_palettes[(self.background_palette_index & 0x3F) as usize] = value;
        if self.background_palette_index.get_bit(7) { // Auto-increment bcps
          // By clearing bit 6 (which is unused) after increment,
          // we prevent incrementing into the higher bits and allow the index to wrap back to 0
          self.background_palette_index = (self.background_palette_index + 1).reset_bit(6);
        }
      }
      MemoryAddress::OCPS => self.object_palette_index = value & 0xBF,
      MemoryAddress::OCPD => {
        self.object_palettes[(self.object_palette_index & 0x3F) as usize] = value;
        if self.object_palette_index.get_bit(7) { // Auto-increment ocps
          // By clearing bit 6 (which is unused) after increment,
          // we prevent incrementing into the higher bits and allow the index to wrap back to 0
          self.object_palette_index = (self.object_palette_index + 1).reset_bit(6);
        }
      }
      _ => panic!("Unable to write to address {:#x} in CRAM", address)
    }
  }
}

#[cfg(test)]
mod tests {
  use test_case::test_case;

  use super::*;

  #[test_case(0xFF68, 0xFF69; "background color")]
  #[test_case(0xFF6A, 0xFF6B; "object color")]
  fn writes_color_to_correct_location(index_address: u16, data_address: u16) {
    let mut cram = CRAMImpl::new();
    cram.write(index_address, 0x34);
    cram.write(data_address, 0xD5);
    cram.write(index_address, 0x35);
    cram.write(data_address, 0x2B);
    cram.write(index_address, 0x34);
    assert_eq!(cram.read(data_address), 0xD5);
    cram.write(index_address, 0x35);
    assert_eq!(cram.read(data_address), 0x2B);
  }

  #[test_case(0xFF68, 0xFF69; "background color")]
  #[test_case(0xFF6A, 0xFF6B; "object color")]
  fn writes_color_with_auto_increment(index_address: u16, data_address: u16) {
    let mut cram = CRAMImpl::new();
    cram.write(index_address, 0xB4);
    cram.write(data_address, 0xD5);
    cram.write(data_address, 0x2B);
    cram.write(index_address, 0x34);
    assert_eq!(cram.read(data_address), 0xD5);
    cram.write(index_address, 0x35);
    assert_eq!(cram.read(data_address), 0x2B);
  }

  #[test]
  fn get_background_color_returns_correct_color() {
    let mut cram = CRAMImpl::new();
    cram.write(0xFF68, 0xB4);
    cram.write(0xFF69, 0xD5);
    cram.write(0xFF69, 0x2B);
    let color = cram.background_color(ColorReference { color_index: 2, palette_index: 6, foreground: false });
    assert_eq!(color.red, 0x15); // Red
    assert_eq!(color.green, 0x1E); // Green
    assert_eq!(color.blue, 0x0A); // Blue
  }

  #[test]
  fn get_object_color_returns_correct_color() {
    let mut cram = CRAMImpl::new();
    cram.write(0xFF6A, 0xB4);
    cram.write(0xFF6B, 0xD5);
    cram.write(0xFF6B, 0x2B);
    let color = cram.object_color(ColorReference { color_index: 2, palette_index: 6, foreground: false });
    assert_eq!(color.red, 0x15); // Red
    assert_eq!(color.green, 0x1E); // Green
    assert_eq!(color.blue, 0x0A); // Blue
  }

  #[test]
  fn write_compatibility_palettes() {
    let mut cram = CRAMImpl::new();
    let color0 = Color::from_rgb(0x01, 0x02, 0x03);
    let color1 = Color::from_rgb(0x04, 0x05, 0x06);
    let color2 = Color::from_rgb(0x07, 0x08, 0x09);
    let color3 = Color::from_rgb(0x0A, 0x0B, 0x0C);
    let color4 = Color::from_rgb(0x11, 0x12, 0x13);
    let color5 = Color::from_rgb(0x14, 0x15, 0x16);
    let color6 = Color::from_rgb(0x17, 0x18, 0x19);
    let color7 = Color::from_rgb(0x1A, 0x1B, 0x1C);
    let color8 = Color::from_rgb(0x1F, 0x1E, 0x1D);
    let color9 = Color::from_rgb(0x1C, 0x1B, 0x1A);
    let color10 = Color::from_rgb(0x19, 0x18, 0x17);
    let color11 = Color::from_rgb(0x16, 0x15, 0x14);
    let compatibility_palettes = CompatibilityPalettes {
      bgp: [color0, color1, color2, color3],
      obj0: [color4, color5, color6, color7],
      obj1: [color8, color9, color10, color11],
    };
    cram.write_compatibility_palettes(compatibility_palettes);
    assert_eq!(cram.background_color(ColorReference { foreground: false, color_index: 0, palette_index: 0 }), color0);
    assert_eq!(cram.background_color(ColorReference { foreground: false, color_index: 1, palette_index: 0 }), color1);
    assert_eq!(cram.background_color(ColorReference { foreground: false, color_index: 2, palette_index: 0 }), color2);
    assert_eq!(cram.background_color(ColorReference { foreground: false, color_index: 3, palette_index: 0 }), color3);
    assert_eq!(cram.object_color(ColorReference { foreground: false, color_index: 0, palette_index: 0 }), Color::transparent());
    assert_eq!(cram.object_color(ColorReference { foreground: false, color_index: 1, palette_index: 0 }), color5);
    assert_eq!(cram.object_color(ColorReference { foreground: false, color_index: 2, palette_index: 0 }), color6);
    assert_eq!(cram.object_color(ColorReference { foreground: false, color_index: 3, palette_index: 0 }), color7);
    assert_eq!(cram.object_color(ColorReference { foreground: false, color_index: 0, palette_index: 1 }), Color::transparent());
    assert_eq!(cram.object_color(ColorReference { foreground: false, color_index: 1, palette_index: 1 }), color9);
    assert_eq!(cram.object_color(ColorReference { foreground: false, color_index: 2, palette_index: 1 }), color10);
    assert_eq!(cram.object_color(ColorReference { foreground: false, color_index: 3, palette_index: 1 }), color11);
  }
}

