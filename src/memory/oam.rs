use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::memory::memory::Memory;
use crate::util::bit_util::BitUtil;

#[derive(Copy, Clone)]
pub struct ObjectAttributes(u8);

impl ObjectAttributes {
  pub fn value(&self) -> u8 {
    self.0
  }

  pub fn render_bg_over_obj(&self) -> bool {
    self.0.get_bit(7)
  }

  pub fn flip_vertical(&self) -> bool {
    self.0.get_bit(6)
  }

  pub fn flip_horizontal(&self) -> bool {
    self.0.get_bit(5)
  }

  pub fn tile_bank_index(&self) -> u8 {
    self.0.get_bit(3) as u8
  }

  pub fn palette_index(&self, monochrome: bool) -> u8 {
    if monochrome { self.0.get_bit(4) as u8 } else { self.0 & 0x7 }
  }
}

#[derive(Copy, Clone)]
pub struct OAMObject {
  pub lcd_y: u8,
  pub lcd_x: u8,
  pub tile_index: u8,
  pub attributes: ObjectAttributes,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ObjectReference {
  pub object_index: u8,
  pub use_bottom_tile: bool
}

#[automock]
pub trait OAM {
  fn get_object_reference_if_intersects(&self, object_index: u8, line: u8, use_8_x_16_tiles: bool) -> Option<ObjectReference>;
  fn get_object(&self, object_reference: ObjectReference, use_8_x_16_tiles: bool) -> OAMObject;
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct OAMImpl {
  #[serde_as(as = "[_;160]")]
  bytes: [u8; 160],
}

impl OAMImpl {
  const START_ADDRESS: usize = 0xFE00;

  pub fn new() -> OAMImpl {
    OAMImpl {
      bytes: [0; 160]
    }
  }
}

impl OAM for OAMImpl {
  fn get_object_reference_if_intersects(&self, object_index: u8, line: u8, use_8_x_16_tiles: bool) -> Option<ObjectReference> {
    let object_lcd_y = self.bytes[4 * object_index as usize];
    let top_tile_intersects = object_lcd_y <= line + 16 && object_lcd_y > line + 8;
    let bottom_tile_intersects = object_lcd_y <= line + 16 && object_lcd_y > line;
    if top_tile_intersects {
      Some(ObjectReference {
        object_index,
        use_bottom_tile: false
      })
    } else if use_8_x_16_tiles && bottom_tile_intersects {
      Some(ObjectReference {
        object_index,
        use_bottom_tile: true
      })
    } else {
      None
    }
  }

  fn get_object(&self, object_reference: ObjectReference, use_8_x_16_tiles: bool) -> OAMObject {
    let byte_offset = 4 * object_reference.object_index as usize;
    let object_bytes = &self.bytes[byte_offset..(byte_offset + 4)];
    let attributes = ObjectAttributes(object_bytes[3]);
    OAMObject {
      lcd_y: if object_reference.use_bottom_tile {
        object_bytes[0] + 8
      } else {
        object_bytes[0]
      },
      lcd_x: object_bytes[1],
      tile_index: if use_8_x_16_tiles {
        let top_tile_index = object_bytes[2] & 0xFE;
        let bottom_tile_index = top_tile_index + 1;
        match (object_reference.use_bottom_tile, attributes.flip_vertical()) {
          (false, false) => top_tile_index,
          (false, true) => bottom_tile_index,
          (true, false) => bottom_tile_index,
          (true, true) => top_tile_index
        }
      } else {
        object_bytes[2]
      },
      attributes,
    }
  }
}

impl Memory for OAMImpl {
  fn read(&self, address: u16) -> u8 {
    self.bytes[address as usize - OAMImpl::START_ADDRESS]
  }

  fn write(&mut self, address: u16, value: u8) {
    self.bytes[address as usize - OAMImpl::START_ADDRESS] = value;
  }
}