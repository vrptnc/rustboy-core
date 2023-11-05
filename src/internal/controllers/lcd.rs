use std::cmp::Ordering;

use mockall::automock;
use serde::{Deserialize, Serialize};

use crate::memory::OAMObject;
use crate::internal::cpu::interrupts::{Interrupt, InterruptController};
use crate::internal::memory::cram::CRAM;
use crate::internal::memory::memory::{Memory, MemoryAddress};
use crate::internal::memory::oam::{OAM, ObjectReference};
use crate::internal::memory::vram::{BackgroundParams, ObjectParams, Point, TileAddressingMode, TileMapIndex, VRAM, WindowParams};
use crate::renderer::{Color, Renderer, RenderTarget};
use crate::internal::util::bit_util::BitUtil;

const DOTS_PER_FRAME: u32 = 70224;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum LCDMode {
  HBlank,
  VBlank,
  Mode2,
  Mode3,
}

#[derive(Serialize, Deserialize)]
struct Stat(u8);

impl Stat {
  pub fn lyc_interrupt_enabled(&self) -> bool {
    self.0.get_bit(6)
  }

  pub fn interrupt_enabled_for_mode(&self, mode: LCDMode) -> bool {
    match mode {
      LCDMode::HBlank => self.0.get_bit(3),
      LCDMode::VBlank => self.0.get_bit(4),
      LCDMode::Mode2 => self.0.get_bit(5),
      LCDMode::Mode3 => false
    }
  }

  pub fn lyc_equals_line(&self) -> bool {
    self.0.get_bit(2)
  }

  pub fn set_lyc_equals_line(&mut self, lyc_equals_line: bool) {
    self.0 = if lyc_equals_line { self.0.set_bit(2) } else { self.0.reset_bit(2) };
  }

  pub fn set_mode(&mut self, mode: LCDMode) {
    let bits: u8 = match mode {
      LCDMode::HBlank => 0x00,
      LCDMode::VBlank => 0x01,
      LCDMode::Mode2 => 0x02,
      LCDMode::Mode3 => 0x03
    };
    self.0 = (self.0 & 0xFC) | bits;
  }
}

#[derive(Serialize, Deserialize)]
struct LCDC(u8);

impl LCDC {
  pub fn bg_priority(&self) -> bool {
    self.0.get_bit(0)
  }

  pub fn obj_enabled(&self) -> bool {
    self.0.get_bit(1)
  }

  pub fn use_8_x_16_tiles(&self) -> bool {
    self.0.get_bit(2)
  }

  pub fn bg_tile_map_index(&self) -> TileMapIndex {
    if self.0.get_bit(3) { TileMapIndex::TileMap2 } else { TileMapIndex::TileMap1 }
  }

  pub fn window_tile_map_index(&self) -> TileMapIndex {
    if self.0.get_bit(6) { TileMapIndex::TileMap2 } else { TileMapIndex::TileMap1 }
  }

  pub fn bg_and_window_tile_addressing_mode(&self) -> TileAddressingMode {
    if self.0.get_bit(4) { TileAddressingMode::Mode8000 } else { TileAddressingMode::Mode8800 }
  }

  pub fn windowing_enabled(&self) -> bool {
    self.0.get_bit(5)
  }

  pub fn lcd_enabled(&self) -> bool {
    self.0.get_bit(7)
  }
}

#[automock]
pub trait LCDController {
  fn get_mode(&self) -> LCDMode;
}

#[derive(Serialize, Deserialize)]
pub struct LCDControllerImpl {
  current_object_index: u8,
  intersecting_object_references: Vec<ObjectReference>,
  dot: u32,
  line: u8,
  line_rendered: bool,
  column: u16,
  mode: LCDMode,
  lcdc: LCDC,
  stat: Stat,
  interrupt_line: bool,  // The STAT interrupt is triggered on the rising edge of this line (which is the OR'ed combination of the various sources that can trigger the input)
  opri: u8,
  scy: u8,
  scx: u8,
  lyc: u8,
  wy: u8,
  wx: u8,
}

impl LCDController for LCDControllerImpl {
  fn get_mode(&self) -> LCDMode {
    self.mode
  }
}

impl LCDControllerImpl {
  pub fn new() -> LCDControllerImpl {
    LCDControllerImpl {
      current_object_index: 0,
      intersecting_object_references: vec![],
      dot: 0,
      line: 0,
      line_rendered: false,
      column: 0,
      mode: LCDMode::Mode2,
      lcdc: LCDC(0x91),
      stat: Stat(0x02), // TODO: Implement writing these registers correctly
      interrupt_line: false,
      opri: 0,
      scy: 0,
      scx: 0,
      lyc: 0,
      wy: 0,
      wx: 0,
    }
  }

  pub fn use_8_x_16_tiles(&self) -> bool {
    self.lcdc.use_8_x_16_tiles()
  }

  fn find_intersecting_objects(&mut self, oam: &dyn OAM) {
    let use_8_x_16_tiles = self.lcdc.use_8_x_16_tiles();
    if self.intersecting_object_references.len() < 10 && self.column % 4 == 0 {
      let object_index = (self.column / 2) as u8;
      if let Some(object_reference) = oam.get_object_reference_if_intersects(object_index, self.line, use_8_x_16_tiles) {
        self.intersecting_object_references.push(object_reference);
      }
      if self.intersecting_object_references.len() < 10 {
        if let Some(object_reference) = oam.get_object_reference_if_intersects(object_index + 1, self.line, use_8_x_16_tiles) {
          self.intersecting_object_references.push(object_reference);
        }
      }
    }
  }

  fn draw_background_line(&self, vram: &dyn VRAM, cram: &dyn CRAM, renderer: &mut dyn Renderer) {
    // Don't draw the background line if we're in monochrome mode and bg_priority bit is cleared
    if self.opri == 1 && !self.lcdc.bg_priority() {
      return;
    }
    let color_references = vram.background_line_colors(BackgroundParams {
      tile_map_index: self.lcdc.bg_tile_map_index(),
      tile_addressing_mode: self.lcdc.bg_and_window_tile_addressing_mode(),
      line: self.line,
      viewport_position: Point {
        x: self.scx,
        y: self.scy,
      },
    });
    color_references.into_iter()
      .map(|color_ref| (color_ref, if self.opri == 1 { cram.monochrome_background_color(color_ref) } else { cram.background_color(color_ref) }))
      .enumerate()
      .for_each(|(x, (color_ref, color))| {
        let background_draw_depth = if color_ref.color_index == 0 || !self.lcdc.bg_priority() {
          0
        } else if color_ref.foreground {
          6
        } else {
          3
        };
        renderer.draw_pixel(x, self.line as usize, background_draw_depth, color, RenderTarget::Main)
      });
  }

  fn should_draw_window_line(&self) -> bool {
    (self.opri == 0 || self.lcdc.bg_priority()) &&
      self.wy <= self.line &&
      self.wy <= 143 &&
      self.wx <= 166
  }

  fn draw_window_line(&self, vram: &dyn VRAM, cram: &dyn CRAM, renderer: &mut dyn Renderer) {
    if self.lcdc.windowing_enabled() && self.should_draw_window_line() {
      let color_references = vram.window_line_colors(WindowParams {
        tile_map_index: self.lcdc.window_tile_map_index(),
        tile_addressing_mode: self.lcdc.bg_and_window_tile_addressing_mode(),
        line: self.line,
        window_position: Point {
          x: self.wx,
          y: self.wy,
        },
      });
      color_references.into_iter()
        .map(|color_ref| {
          let color = if self.opri == 1 {
            cram.monochrome_background_color(color_ref)
          } else {
            cram.background_color(color_ref)
          };
          color
        })
        .enumerate()
        .skip(if self.wx < 7 { 7 - self.wx as usize } else { 0 })
        .for_each(|(x, color)| {
          renderer.draw_pixel(x + self.wx as usize - 7, self.line as usize, 0xFF, color, RenderTarget::Main);
        });
    }
  }

  fn draw_obj_line(&self, vram: &dyn VRAM, cram: &dyn CRAM, oam: &dyn OAM, renderer: &mut dyn Renderer) {
    if !self.lcdc.obj_enabled() {
      return;
    }
    let mut objects: Vec<OAMObject> = self.intersecting_object_references.iter()
      .map(|object_reference| oam.get_object(*object_reference, self.lcdc.use_8_x_16_tiles()))
      .collect();
    if self.opri == 1 {
      objects.sort_by(|a, b| {
        if a.lcd_x < b.lcd_x {
          Ordering::Less
        } else if a.lcd_x > b.lcd_x {
          Ordering::Greater
        } else {
          Ordering::Equal
        }
      });
    }

    objects.into_iter()
      .filter(|object| object.lcd_x != 0 && object.lcd_x <= 168)
      .for_each(|object| {
        let params = ObjectParams {
          object,
          row: self.line + 16 - object.lcd_y,
          monochrome: self.opri == 1,
        };
        let colors = vram.object_line_colors(params);
        colors.into_iter()
          .map(|color_ref| (color_ref, if self.opri == 1 { cram.monochrome_object_color(color_ref) } else { cram.object_color(color_ref) }))
          .enumerate()
          .skip(if object.lcd_x < 8 { 8 - object.lcd_x } else { 0 } as usize)
          .take(if object.lcd_x > 160 { 168 - object.lcd_x } else { 8 } as usize)
          .for_each(|(pixel_offset, (color_ref, color))| {
            let obj_draw_depth = if color_ref.foreground {
              5
            } else {
              2
            };
            renderer.draw_pixel(object.lcd_x as usize + pixel_offset - 8, self.line as usize, obj_draw_depth, color, RenderTarget::Main);
          });
      });
  }

  fn draw_obj_atlas_line(&self, vram: &dyn VRAM, cram: &dyn CRAM, oam: &dyn OAM, renderer: &mut dyn Renderer) {
    if self.line < 32 {
      let row = self.line % 8;
      let object_range = if self.line < 8 || (self.line < 16 && self.lcdc.use_8_x_16_tiles()) {
        0..20u8
      } else if self.line < 24 || (self.line < 32 && self.lcdc.use_8_x_16_tiles()) {
        20..40u8
      } else {
        0..0u8
      };
      object_range.into_iter()
        .for_each(|object_index| {
          let object = oam.get_object(ObjectReference {
            object_index,
            use_bottom_tile: (self.line % 16) > 7,
          }, self.lcdc.use_8_x_16_tiles());
          let params = ObjectParams {
            object,
            row,
            monochrome: self.opri == 1,
          };
          let column_offset = (object_index % 20) * 8;
          let colors = vram.object_line_colors(params);
          colors.into_iter()
            .map(|color_ref| if self.opri == 1 { cram.monochrome_object_color(color_ref) } else { cram.object_color(color_ref) })
            .enumerate()
            .for_each(|(pixel_offset, color)| {
              renderer.draw_pixel(column_offset as usize + pixel_offset, self.line as usize, 5, color, RenderTarget::ObjectAtlas);
            });
        })
    }
  }

  fn draw_tile_atlas_line(&self, vram: &dyn VRAM, renderer: &mut dyn Renderer) {
    if self.line == 0 {
      (0..192u8)
        .map(|line| (line, vram.tile_atlas_line_colors(line)))
        .for_each(|(line, colors)| {
          colors.into_iter()
            .map(|color_ref| match color_ref {
              0 => Color::white(),
              1 => Color::light_grey(),
              2 => Color::dark_grey(),
              _ => Color::black()
            })
            .enumerate()
            .for_each(|(pixel_offset, color)| {
              renderer.draw_pixel(pixel_offset, line as usize, 5, color, RenderTarget::TileAtlas);
            });
        });
    }
  }

  fn draw_line(&self, vram: &dyn VRAM, cram: &dyn CRAM, oam: &dyn OAM, renderer: &mut dyn Renderer) {
    if renderer.render_target_is_enabled(RenderTarget::Main) {
      self.draw_background_line(vram, cram, renderer);
      self.draw_window_line(vram, cram, renderer);
      self.draw_obj_line(vram, cram, oam, renderer);
    }
    if renderer.render_target_is_enabled(RenderTarget::ObjectAtlas) {
      self.draw_obj_atlas_line(vram, cram, oam, renderer);
    }
    if renderer.render_target_is_enabled(RenderTarget::TileAtlas) {
      self.draw_tile_atlas_line(vram, renderer);
    }
  }

  fn update_mode(&mut self) {
    self.mode = if self.line >= 144 {
      LCDMode::VBlank
    } else {
      match self.column {
        0..=79 => LCDMode::Mode2,
        80..=247 => LCDMode::Mode3,
        _ => LCDMode::HBlank
      }
    };
    self.stat.set_mode(self.mode);
  }

  fn maybe_request_interrupt(&mut self, interrupt_controller: &mut dyn InterruptController) {
    let new_interrupt_line =
      self.stat.interrupt_enabled_for_mode(self.mode) ||
        (self.stat.lyc_equals_line() && self.stat.lyc_interrupt_enabled());
    if new_interrupt_line && !self.interrupt_line {
      interrupt_controller.request_interrupt(Interrupt::Stat);
    }
    self.interrupt_line = new_interrupt_line;
  }

  pub fn tick(&mut self, vram: &dyn VRAM, cram: &dyn CRAM, oam: &dyn OAM,
              renderer: &mut dyn Renderer,
              interrupt_controller: &mut dyn InterruptController,
              double_speed: bool) {
    /*
     * The LCD works with a dot clock, that ticks at the clock frequency.
     * The LCD works with 154 scanlines of 456 dots each = 70224 dots per frame
     * The LCD is only 160 x 144 pixels wide, so scanlines 144-153 are the VBlank period.
     * The 456 dots per scanline consist of 80 dots spent in mode 2 (searching the OAM for viable objects that intersect the current scanline),
     * 168-291 dots spent in mode 3 (rendering the image), and the remaining dots spent in HBlank
     */
    let number_of_dots_for_tick = if double_speed { 2u32 } else { 4u32 };
    self.dot = (self.dot + number_of_dots_for_tick) % DOTS_PER_FRAME;
    if !self.lcdc.lcd_enabled() {
      return;
    }
    self.line = (self.dot / 456) as u8;
    self.column = (self.dot % 456) as u16;


    self.stat.set_lyc_equals_line(self.line == self.lyc);


    self.update_mode();
    self.maybe_request_interrupt(interrupt_controller);


    match self.mode {
      LCDMode::HBlank => {
        if self.column == 248 {
          self.intersecting_object_references.clear();
          self.current_object_index = 0;
        }
      }
      LCDMode::VBlank => {
        if self.column == 0 && self.line == 144 {
          interrupt_controller.request_interrupt(Interrupt::VerticalBlank);
          renderer.flush();
        }
      }
      LCDMode::Mode2 => {
        self.line_rendered = false;
        self.find_intersecting_objects(oam);
      }
      LCDMode::Mode3 => {
        if !self.line_rendered {
          self.draw_line(vram, cram, oam, renderer);
          self.line_rendered = true;
        }
      }
    }
  }
}

impl Memory for LCDControllerImpl {
  fn read(&self, address: u16) -> u8 {
    match address {
      MemoryAddress::LCDC => self.lcdc.0,
      MemoryAddress::STAT => 0x80 | self.stat.0,
      MemoryAddress::SCY => self.scy,
      MemoryAddress::SCX => self.scx,
      MemoryAddress::LY => self.line,
      MemoryAddress::LYC => self.lyc,
      MemoryAddress::WY => self.wy,
      MemoryAddress::WX => self.wx,
      MemoryAddress::OPRI => self.opri,
      _ => panic!("Unable to read address {:#x} from LCD Controller", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      MemoryAddress::LCDC => self.lcdc.0 = value,
      MemoryAddress::STAT => self.stat.0 = (self.stat.0 & 0x7) | (value & 0xF8),
      MemoryAddress::SCY => self.scy = value,
      MemoryAddress::SCX => self.scx = value,
      MemoryAddress::LYC => self.lyc = value,
      MemoryAddress::WY => self.wy = value,
      MemoryAddress::WX => self.wx = value,
      MemoryAddress::OPRI => self.opri = value,
      _ => panic!("Unable to write to address {:#x} in LCD Controller", address)
    }
  }
}

#[cfg(test)]
pub mod tests {
  use mockall::predicate::eq;

  use crate::internal::cpu::interrupts::MockInterruptController;
  use crate::internal::memory::cram::{ColorReference, MockCRAM};
  use crate::internal::memory::oam::MockOAM;
  use crate::internal::memory::vram::MockVRAM;
  use crate::renderer::MockRenderer;

  use super::*;

  #[test]
  fn stat_blocking() {
    let mut controller = LCDControllerImpl::new();
    let mut renderer = MockRenderer::new();
    let mut interrupt_controller = MockInterruptController::new();
    interrupt_controller.expect_request_interrupt().never();
    let mut vram = MockVRAM::new();
    let mut cram = MockCRAM::new();
    let mut oam = MockOAM::new();
    let mocked_colors = vec![ColorReference {
      color_index: 0,
      palette_index: 0,
      foreground: false,
    }; 160];
    renderer.expect_render_target_is_enabled().with(eq(RenderTarget::Main)).return_const(true);
    renderer.expect_render_target_is_enabled().with(eq(RenderTarget::TileAtlas)).return_const(false);
    renderer.expect_render_target_is_enabled().with(eq(RenderTarget::ObjectAtlas)).return_const(false);
    renderer.expect_draw_pixel().return_const(());
    vram.expect_background_line_colors().return_const(mocked_colors);
    cram.expect_background_color().return_const(Color::white());
    oam.expect_get_object_reference_if_intersects().return_const(None);
    // Advance to right before HBlank
    for _ in 0..62 {
      controller.tick(&vram, &cram, &oam, &mut renderer, &mut interrupt_controller, false);
    }
    controller.write(MemoryAddress::STAT, 0x28); // Enable STAT interrupt for Mode 2 and HBlank

    // We expect the interrupt to only be requested once when going into HBlank, and not when we transition to Mode2
    interrupt_controller.expect_request_interrupt().with(eq(Interrupt::Stat)).once().return_const(());

    controller.tick(&vram, &cram, &oam, &mut renderer, &mut interrupt_controller, false); // Enter HBlank
    // Advance to well within Mode 2 of the next line. No additional interrupt should be requested due to STAT blocking
    for _ in 62..120 {
      controller.tick(&vram, &cram, &oam, &mut renderer, &mut interrupt_controller, false);
    }
  }
}