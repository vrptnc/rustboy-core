use crate::internal::memory::oam::ObjectAttributes;

#[derive(Copy, Clone)]
pub struct OAMObject {
    pub lcd_y: u8,
    pub lcd_x: u8,
    pub tile_index: u8,
    pub attributes: ObjectAttributes,
}