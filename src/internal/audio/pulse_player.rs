use serde::{Deserialize, Serialize};

use crate::audio::{AudioDriver, Channel, PulseOptions};
use crate::internal::controllers::audio::DutyCycle;
use crate::internal::util::request_flag::RequestFlag;

pub enum PulsePlayerTickResult {
  Ok,
  WavelengthOverflowed,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct PulsePlayerSettings {
  pub initial_wavelength: u16,
  pub shift: u8,
  pub pace: u8,
  pub decrease: bool,
  pub duty_cycle: DutyCycle,
}

impl PulsePlayerSettings {
  pub fn new() -> Self {
    PulsePlayerSettings {
      initial_wavelength: 0,
      shift: 0,
      pace: 0,
      decrease: false,
      duty_cycle: DutyCycle::Duty125,
    }
  }

  pub fn get_lower_wavelength_bits(&self) -> u8 {
    (self.initial_wavelength & 0xFF) as u8
  }

  pub fn get_upper_wavelength_bits(&self) -> u8 {
    ((self.initial_wavelength & 0xFF00) >> 8) as u8
  }

  pub fn set_lower_wavelength_bits(&mut self, value: u8) {
    self.initial_wavelength = (self.initial_wavelength & 0xFF00) | (value as u16);
  }

  pub fn set_upper_wavelength_bits(&mut self, value: u8) {
    self.initial_wavelength = (self.initial_wavelength & 0x00FF) | ((value as u16 & 0x7) << 8);
  }
}

#[derive(Serialize, Deserialize)]
pub struct PulsePlayer {
  channel: Channel,
  triggered: RequestFlag,
  current_tick: u8,
  wavelength: u16,
  current_settings: PulsePlayerSettings,
  pub new_settings: PulsePlayerSettings,
  pub playing: bool,
}

impl PulsePlayer {
  pub fn new(channel: Channel) -> Self {
    PulsePlayer {
      channel,
      triggered: RequestFlag::new(),
      current_tick: 0,
      wavelength: 0,
      current_settings: PulsePlayerSettings::new(),
      new_settings: PulsePlayerSettings::new(),
      playing: false,
    }
  }

  pub fn trigger(&mut self) {
    self.triggered.set();
    self.current_settings = self.new_settings;
    self.current_tick = 0;
    self.wavelength = self.current_settings.initial_wavelength;
    self.playing = true;
  }

  pub fn set_pace(&mut self, new_pace: u8) {
    self.new_settings.pace = new_pace;
    if self.current_settings.pace == 0 && new_pace != 0 {
      self.current_settings.pace = new_pace;
      self.triggered.set();
    }
  }

  fn wavelength_overflowed(&self) -> bool {
    self.wavelength > 0x7FF
  }

  fn play_pulse(&self, audio_driver: &mut dyn AudioDriver) {
    audio_driver.play_pulse(self.channel, PulseOptions {
      frequency: 131072.0f32 / (2048.0 - self.wavelength as f32),
      duty_cycle: self.current_settings.duty_cycle.to_ratio(),
    });
  }

  pub fn stop(&mut self, audio_driver: &mut dyn AudioDriver) {
    self.playing = false;
    audio_driver.stop(self.channel);
  }

  pub fn tick(&mut self, audio_driver: &mut dyn AudioDriver) -> PulsePlayerTickResult {
    if self.triggered.get_and_clear() {
      self.play_pulse(audio_driver);
    } else if self.playing && self.current_settings.pace != 0 && self.current_settings.shift != 0 {
      self.current_tick = (self.current_tick + 1) % self.current_settings.pace;
      if self.current_tick == 0 {
        if self.current_settings.decrease {
          self.wavelength -= self.wavelength >> self.current_settings.shift;
        } else {
          self.wavelength += self.wavelength >> self.current_settings.shift;
        }
        if !self.wavelength_overflowed() {
          self.play_pulse(audio_driver);
        }
      }
    }
    if self.wavelength_overflowed() {
      PulsePlayerTickResult::WavelengthOverflowed
    } else {
      PulsePlayerTickResult::Ok
    }
  }
}