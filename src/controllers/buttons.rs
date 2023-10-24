use mockall::automock;
use serde::{Deserialize, Serialize};

use crate::cpu::interrupts::{Interrupt, InterruptController};
use crate::memory::memory::{Memory, MemoryAddress};
use crate::util::bit_util::BitUtil;

#[automock]
pub trait ButtonController {
  fn press_button(&mut self, button: Button, interrupt_controller: &mut dyn InterruptController);
  fn release_button(&mut self, button: Button);
}

#[derive(Serialize, Deserialize)]
struct ButtonRegister {
  deferred_interrupt: bool,
  button_enabled_bit: u8,
  buttons_pressed_flags: u8,
  buttons_enabled: bool,
}

impl ButtonRegister {
  pub fn new(button_type: ButtonType) -> Self {
    ButtonRegister {
      deferred_interrupt: false,
      button_enabled_bit: if let ButtonType::DIRECTION = button_type { 4 } else { 5 },
      buttons_pressed_flags: 0x00,
      buttons_enabled: false,
    }
  }

  pub fn press_button(&mut self, button: Button, interrupt_controller: &mut dyn InterruptController) {
    let old_buttons_pressed_flags = self.buttons_pressed_flags;
    self.buttons_pressed_flags = self.buttons_pressed_flags.set_bit(button.button_index() as u8);
    if self.buttons_enabled && old_buttons_pressed_flags != self.buttons_pressed_flags {
      interrupt_controller.request_interrupt(Interrupt::ButtonPressed);
    }
  }

  pub fn release_button(&mut self, button: Button) {
    self.buttons_pressed_flags = self.buttons_pressed_flags.reset_bit(button.button_index() as u8);
  }

  pub fn buttons_enabled(&mut self, enabled: bool) {
    if enabled && !self.buttons_enabled && self.buttons_pressed_flags != 0x00 {
      self.deferred_interrupt = true;
    }
    self.buttons_enabled = enabled;
  }

  pub fn pressed_buttons(&self) -> u8 {
    if self.buttons_enabled {
      (!self.buttons_pressed_flags & 0x3F).reset_bit(self.button_enabled_bit)
    } else {
      0x3F
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct ButtonControllerImpl {
  action_buttons_register: ButtonRegister,
  direction_buttons_register: ButtonRegister,
}

impl ButtonControllerImpl {
  pub fn new() -> ButtonControllerImpl {
    ButtonControllerImpl {
      action_buttons_register: ButtonRegister::new(ButtonType::ACTION),
      direction_buttons_register: ButtonRegister::new(ButtonType::DIRECTION),
    }
  }

  pub fn tick(&mut self, interrupt_controller: &mut dyn InterruptController) {
    if self.action_buttons_register.deferred_interrupt || self.direction_buttons_register.deferred_interrupt {
      interrupt_controller.request_interrupt(Interrupt::ButtonPressed);
      self.action_buttons_register.deferred_interrupt = false;
      self.direction_buttons_register.deferred_interrupt = false;
    }
  }
}

impl ButtonController for ButtonControllerImpl {
  fn press_button(&mut self, button: Button, interrupt_controller: &mut dyn InterruptController) {
    match button.button_type() {
      ButtonType::ACTION => self.action_buttons_register.press_button(button, interrupt_controller),
      ButtonType::DIRECTION => self.direction_buttons_register.press_button(button, interrupt_controller)
    }
  }

  fn release_button(&mut self, button: Button) {
    match button.button_type() {
      ButtonType::ACTION => self.action_buttons_register.release_button(button),
      ButtonType::DIRECTION => self.direction_buttons_register.release_button(button)
    }
  }
}

impl Memory for ButtonControllerImpl {
  fn read(&self, address: u16) -> u8 {
    match address {
      MemoryAddress::P1 => self.action_buttons_register.pressed_buttons() & self.direction_buttons_register.pressed_buttons(),
      _ => panic!("ButtonController can't read from address {}", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      MemoryAddress::P1 => {
        self.direction_buttons_register.buttons_enabled(!value.get_bit(4));
        self.action_buttons_register.buttons_enabled(!value.get_bit(5));
      }
      _ => panic!("ButtonController can't write to address {}", address)
    }
  }
}

pub enum ButtonType {
  ACTION,
  DIRECTION,
}

#[derive(Copy, Clone)]
pub enum Button {
  A,
  B,
  SELECT,
  START,
  RIGHT,
  LEFT,
  UP,
  DOWN,
}

impl Button {
  pub fn button_index(&self) -> usize {
    match self {
      Button::A => 0,
      Button::B => 1,
      Button::SELECT => 2,
      Button::START => 3,
      Button::RIGHT => 0,
      Button::LEFT => 1,
      Button::UP => 2,
      Button::DOWN => 3,
    }
  }

  pub fn button_type(&self) -> ButtonType {
    match self {
      Button::A => ButtonType::ACTION,
      Button::B => ButtonType::ACTION,
      Button::SELECT => ButtonType::ACTION,
      Button::START => ButtonType::ACTION,
      Button::RIGHT => ButtonType::DIRECTION,
      Button::LEFT => ButtonType::DIRECTION,
      Button::UP => ButtonType::DIRECTION,
      Button::DOWN => ButtonType::DIRECTION
    }
  }
}

#[cfg(test)]
mod tests {
  use assert_hex::assert_eq_hex;

  use crate::cpu::interrupts::MockInterruptController;
  use crate::memory::memory::MemoryAddress;

  use super::*;

  #[test]
  fn no_buttons_pressed_no_output_port_low() {
    let controller = ButtonControllerImpl::new();
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x3F);
  }

  #[test]
  fn action_buttons_pressed_no_output_port_low() {
    let mut controller = ButtonControllerImpl::new();
    let mut interrupt_controller = MockInterruptController::new();
    interrupt_controller.expect_request_interrupt().never();
    controller.press_button(Button::START, &mut interrupt_controller);
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x3F);
  }

  #[test]
  fn action_buttons_pressed_action_output_port_low() {
    let mut controller = ButtonControllerImpl::new();
    let mut interrupt_controller = MockInterruptController::new();
    interrupt_controller.expect_request_interrupt().never();
    controller.write(MemoryAddress::P1, 0x20);
    controller.press_button(Button::A, &mut interrupt_controller);
    controller.press_button(Button::START, &mut interrupt_controller);
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x2F);
    controller.release_button(Button::A);
    controller.release_button(Button::START);
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x2F);
    interrupt_controller.expect_request_interrupt().times(2).return_const(());
    controller.write(MemoryAddress::P1, 0x10);
    controller.press_button(Button::A, &mut interrupt_controller);
    controller.press_button(Button::START, &mut interrupt_controller);
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x16);
  }

  #[test]
  fn direction_buttons_pressed_direction_output_port_low() {
    let mut controller = ButtonControllerImpl::new();
    let mut interrupt_controller = MockInterruptController::new();
    interrupt_controller.expect_request_interrupt().never();
    controller.write(MemoryAddress::P1, 0x10);
    controller.press_button(Button::RIGHT, &mut interrupt_controller);
    controller.press_button(Button::DOWN, &mut interrupt_controller);
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x1F);
    controller.release_button(Button::RIGHT);
    controller.release_button(Button::DOWN);
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x1F);
    interrupt_controller.expect_request_interrupt().times(2).return_const(());
    controller.write(MemoryAddress::P1, 0x20);
    controller.press_button(Button::RIGHT, &mut interrupt_controller);
    controller.press_button(Button::DOWN, &mut interrupt_controller);
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x26);
  }

  #[test]
  fn action_buttons_released() {
    let mut controller = ButtonControllerImpl::new();
    let mut interrupt_controller = MockInterruptController::new();
    controller.press_button(Button::A, &mut interrupt_controller);
    controller.press_button(Button::START, &mut interrupt_controller);
    controller.release_button(Button::A);
    controller.write(MemoryAddress::P1, 0x10);
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x17);
  }

  #[test]
  fn button_press_triggers_interrupt() {
    let mut controller = ButtonControllerImpl::new();
    let mut interrupt_controller = MockInterruptController::new();
    controller.press_button(Button::A, &mut interrupt_controller);
    controller.press_button(Button::START, &mut interrupt_controller);
    controller.write(MemoryAddress::P1, 0x10);
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x16);
    controller.write(MemoryAddress::P1, 0x20);
    assert_eq_hex!(controller.read(MemoryAddress::P1), 0x2F);
  }

  #[test]
  fn button_enable_triggers_deferred_interrupt_on_tick() {
    let mut controller = ButtonControllerImpl::new();
    let mut interrupt_controller = MockInterruptController::new();
    interrupt_controller.expect_request_interrupt().never();
    controller.press_button(Button::A, &mut interrupt_controller);
    controller.press_button(Button::START, &mut interrupt_controller);
    controller.tick(&mut interrupt_controller);
    controller.write(MemoryAddress::P1, 0x10);
    interrupt_controller.expect_request_interrupt().once().return_const(());
    controller.tick(&mut interrupt_controller);
  }
}