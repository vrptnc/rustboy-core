use itertools::Either;
use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::memory::cram::ColorReference;
use crate::memory::memory::{Memory, MemoryAddress};
use crate::memory::oam::OAMObject;
use crate::renderer::renderer::{Point, TileAddressingMode, TileMapIndex};
use crate::util::bit_util::{BitUtil, ByteUtil};

#[derive(Copy, Clone)]
pub struct TileAttributes(u8);

impl TileAttributes {
    pub fn bg_and_window_priority_over_oam(&self) -> bool {
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

    pub fn palette_index(&self) -> u8 {
        self.0 & 0x7
    }
}

#[derive(Copy, Clone)]
pub struct Tile {
    pub chr_code: u8,
    pub attributes: TileAttributes,
}

#[derive(Copy, Clone)]
pub struct TileData<'a> {
    bytes: &'a [u8],
}

impl<'a> TileData<'a> {
    pub fn get_color_indices(&self, row_offset: u8, flip_horizontal: bool, flip_vertical: bool) -> impl Iterator<Item=u8> + 'a {
        let (byte1, byte2) = if flip_vertical {
            (self.bytes[14 - 2 * row_offset as usize], self.bytes[15 - 2 * row_offset as usize])
        } else {
            (self.bytes[2 * row_offset as usize], self.bytes[2 * row_offset as usize + 1])
        };
        let interleaved_word = byte1.interleave_with(byte2);
        if flip_horizontal {
            Either::Left(interleaved_word.crumbs())
        } else {
            Either::Right(interleaved_word.crumbs().rev())
        }
    }
}

#[derive(Copy, Clone)]
pub struct TileDataView<'a> {
    block_1: [&'a [u8]; 2],
    block_2: [&'a [u8]; 2],
}

impl<'a> TileDataView<'a> {
    pub fn get_tile_data(&self, tile_bank_index: u8, tile_index: u8) -> TileData {
        if let 0..=127 = tile_index {
            let byte_offset = 16 * tile_index as usize;
            TileData {
                bytes: &self.block_1[tile_bank_index as usize][byte_offset..byte_offset + 16]
            }
        } else {
            let byte_offset = 16 * (tile_index - 128) as usize;
            TileData {
                bytes: &self.block_2[tile_bank_index as usize][byte_offset..byte_offset + 16]
            }
        }
    }
}

pub struct TileMapView<'a> {
    bytes: [&'a [u8]; 2],
}

impl<'a> TileMapView<'a> {
    const TILES_PER_ROW: usize = 32;

    pub fn row(&'a self, row: u8) -> impl Iterator<Item=Tile> + Clone + 'a {
        let tile_offset = row as usize * TileMapView::TILES_PER_ROW;
        (0..TileMapView::TILES_PER_ROW)
            .map(move |tile_index| Tile {
                chr_code: self.bytes[0][tile_offset + tile_index],
                attributes: TileAttributes(self.bytes[1][tile_offset + tile_index]),
            })
    }
}

#[derive(Copy, Clone)]
pub struct ObjectParams {
    pub object: OAMObject,
    pub row: u8,
    pub monochrome: bool,
}

#[derive(Copy, Clone)]
pub struct BackgroundParams {
    pub tile_map_index: TileMapIndex,
    pub tile_addressing_mode: TileAddressingMode,
    pub line: u8,
    pub viewport_position: Point,
}

#[derive(Copy, Clone)]
pub struct WindowParams {
    pub tile_map_index: TileMapIndex,
    pub tile_addressing_mode: TileAddressingMode,
    pub line: u8,
    pub window_position: Point,
}

#[automock]
pub trait VRAM {
    fn object_line_colors(&self, params: ObjectParams) -> Vec<ColorReference>;
    fn background_line_colors(&self, params: BackgroundParams) -> Vec<ColorReference>;
    fn window_line_colors(&self, params: WindowParams) -> Vec<ColorReference>;
    fn tile_atlas_line_colors(&self, line: u8) -> Vec<u8>;
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct VRAMImpl {
    bank_index: u8,
    #[serde_as(as = "[[_;VRAMImpl::BANK_SIZE]; 2]")]
    bytes: [[u8; VRAMImpl::BANK_SIZE]; 2],
}

impl VRAMImpl {
    const START_ADDRESS: u16 = 0x8000;
    const END_ADDRESS: u16 = 0x9FFF;
    const BANK_SIZE: usize = 0x2000;

    pub fn new() -> VRAMImpl {
        VRAMImpl {
            bank_index: 0,
            bytes: [[0; VRAMImpl::BANK_SIZE]; 2],
        }
    }

    fn tile_map(&self, tile_map_index: TileMapIndex) -> TileMapView {
        match tile_map_index {
            TileMapIndex::TileMap1 => TileMapView {
                bytes: [&self.bytes[0][0x1800..0x1C00], &self.bytes[1][0x1800..0x1C00]]
            },
            TileMapIndex::TileMap2 => TileMapView {
                bytes: [&self.bytes[0][0x1C00..0x2000], &self.bytes[1][0x1C00..0x2000]]
            }
        }
    }

    fn tile_data(&self, addressing_mode: TileAddressingMode) -> TileDataView {
        match addressing_mode {
            TileAddressingMode::Mode8000 => TileDataView {
                block_1: [&self.bytes[0][0..0x800], &self.bytes[1][0..0x800]],
                block_2: [&self.bytes[0][0x800..0x1000], &self.bytes[1][0x800..0x1000]],
            },
            TileAddressingMode::Mode8800 => TileDataView {
                block_1: [&self.bytes[0][0x1000..0x1800], &self.bytes[1][0x1000..0x1800]],
                block_2: [&self.bytes[0][0x800..0x1000], &self.bytes[1][0x800..0x1000]],
            }
        }
    }
}

impl VRAM for VRAMImpl {
    fn object_line_colors(&self, params: ObjectParams) -> Vec<ColorReference> {
        let tile_data_view = self.tile_data(TileAddressingMode::Mode8000);
        let OAMObject { attributes, tile_index, .. } = params.object;
        tile_data_view.get_tile_data(attributes.tile_bank_index(), tile_index)
            .get_color_indices(params.row, attributes.flip_horizontal(), attributes.flip_vertical())
            .map(|color_index| ColorReference {
                foreground: !attributes.render_bg_over_obj(),
                color_index,
                palette_index: attributes.palette_index(params.monochrome),
            })
            .collect()
    }

    fn background_line_colors(&self, params: BackgroundParams) -> Vec<ColorReference> {
        let tile_map = self.tile_map(params.tile_map_index);
        let tile_data_view = self.tile_data(params.tile_addressing_mode);

        let tile_column_offset = params.viewport_position.x / 8;
        let pixel_column_offset = params.viewport_position.x % 8;
        let pixel_row = (params.line + params.viewport_position.y) % 255;
        let tile_row = pixel_row / 8;
        let pixel_row_offset = pixel_row % 8;

        tile_map.row(tile_row)
            .cycle()
            .skip(tile_column_offset as usize)
            .enumerate()
            .flat_map(|(tile_index, Tile { chr_code, attributes })| tile_data_view
                .get_tile_data(attributes.tile_bank_index(), chr_code)
                .get_color_indices(pixel_row_offset, attributes.flip_horizontal(), attributes.flip_vertical())
                .skip(if tile_index == 0 { pixel_column_offset as usize } else { 0 })
                .map(move |color_index| ColorReference {
                    foreground: attributes.bg_and_window_priority_over_oam(),
                    color_index,
                    palette_index: attributes.palette_index(),
                })
            )
            .take(160)
            .collect()
    }

    fn window_line_colors(&self, params: WindowParams) -> Vec<ColorReference> {
        let tile_map = self.tile_map(params.tile_map_index);
        let tile_data_view = self.tile_data(params.tile_addressing_mode);

        let pixel_row = params.line - params.window_position.y;
        let tile_row = pixel_row / 8;
        let pixel_row_offset = pixel_row % 8;
        let pixels_to_draw = if params.window_position.x < 7 {
            160
        } else {
            160 - params.window_position.x + 7
        };

        tile_map.row(tile_row)
            .flat_map(|Tile { chr_code, attributes }| tile_data_view
                .get_tile_data(attributes.tile_bank_index(), chr_code)
                .get_color_indices(pixel_row_offset, attributes.flip_horizontal(), attributes.flip_vertical())
                .map(move |color_index| ColorReference {
                    foreground: attributes.bg_and_window_priority_over_oam(),
                    color_index,
                    palette_index: attributes.palette_index(),
                })
            )
            .take(pixels_to_draw as usize)
            .collect()
    }

    fn tile_atlas_line_colors(&self, line: u8) -> Vec<u8> {
        let tile_row = line / 8;
        let pixel_row = line % 8;
        let tile_offset = if line < 128 {
            tile_row * 16
        } else {
            (tile_row - 16) * 16
        };
        let tile_data_view = if line < 128 {
            self.tile_data(TileAddressingMode::Mode8000)
        } else {
            self.tile_data(TileAddressingMode::Mode8800)
        };
        let bank_0_colors = (tile_offset..=(tile_offset + 15))
            .flat_map(|tile_index| {
                tile_data_view.get_tile_data(0, tile_index).get_color_indices(pixel_row, false, false)
            });
        let bank_1_colors = (tile_offset..=(tile_offset + 15))
            .flat_map(|tile_index| {
                tile_data_view.get_tile_data(1, tile_index).get_color_indices(pixel_row, false, false)
            });
        bank_0_colors.chain(bank_1_colors).collect()
    }
}

impl Memory for VRAMImpl {
    fn read(&self, address: u16) -> u8 {
        match address {
            VRAMImpl::START_ADDRESS..=VRAMImpl::END_ADDRESS => {
                self.bytes[self.bank_index as usize][(address - VRAMImpl::START_ADDRESS) as usize]
            }
            MemoryAddress::VBK => self.bank_index,
            _ => panic!("Can't read address {} from VRAM", address)
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            VRAMImpl::START_ADDRESS..=VRAMImpl::END_ADDRESS => {
                self.bytes[self.bank_index as usize][(address - VRAMImpl::START_ADDRESS) as usize] = value
            }
            MemoryAddress::VBK => {
                self.bank_index = value & 0x01
            }
            _ => panic!("Can't write to address {} in VRAM", address)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use assert_hex::assert_eq_hex;

    use crate::memory::memory::MemoryAddress;

    use super::*;

    #[test]
    fn set_vram_bank() {
        let mut vram = VRAMImpl::new();
        vram.write(MemoryAddress::VBK, 0);
        vram.write(VRAMImpl::START_ADDRESS, 0xAB);
        vram.write(MemoryAddress::VBK, 1);
        vram.write(VRAMImpl::START_ADDRESS, 0xCD);
        assert_eq_hex!(vram.read(VRAMImpl::START_ADDRESS), 0xCD);
        vram.write(MemoryAddress::VBK, 0);
        assert_eq_hex!(vram.read(VRAMImpl::START_ADDRESS), 0xAB);
    }
}

