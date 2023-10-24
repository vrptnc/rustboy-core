use serde::{Deserialize, Serialize};

use crate::audio::audio_driver::{AudioDriver, Channel, CustomWaveOptions};
use crate::util::request_flag::RequestFlag;

pub enum CustomWavePlayerTickResult {
  Ok,
  DacShutOff,
}

#[derive(Serialize, Deserialize)]
pub struct CustomWavePlayer {
  channel: Channel,
  pub waveform: [u8; 16],
  triggered: RequestFlag,
  frequency_changed: RequestFlag,
  gain_changed: RequestFlag,
  dac_enabled_changed: RequestFlag,
  pub wavelength: u16,
  pub gain: u8,
  pub playing: bool,
  pub dac_enabled: bool,
}

impl CustomWavePlayer {
  pub fn new(channel: Channel) -> Self {
    CustomWavePlayer {
      channel,
      waveform: [0; 16],
      triggered: RequestFlag::new(),
      frequency_changed: RequestFlag(true),
      gain_changed: RequestFlag(true),
      dac_enabled_changed: RequestFlag(true),
      wavelength: 0,
      gain: 0,
      playing: false,
      dac_enabled: false,
    }
  }

  pub fn trigger(&mut self) {
    self.triggered.set();
  }

  pub fn stop(&mut self, audio_driver: &mut dyn AudioDriver) {
    self.playing = false;
    audio_driver.stop(self.channel);
  }

  pub fn get_lower_wavelength_bits(&self) -> u8 {
    (self.wavelength & 0xFF) as u8
  }

  pub fn get_upper_wavelength_bits(&self) -> u8 {
    ((self.wavelength & 0xFF00) >> 8) as u8
  }

  pub fn set_lower_wavelength_bits(&mut self, value: u8) {
    self.wavelength = (self.wavelength & 0xFF00) | (value as u16);
    self.frequency_changed.set();
  }

  pub fn set_upper_wavelength_bits(&mut self, value: u8) {
    self.wavelength = (self.wavelength & 0x00FF) | ((value as u16 & 0x7) << 8);
    self.frequency_changed.set();
  }

  pub fn set_gain(&mut self, value: u8) {
    self.gain = value;
    self.gain_changed.set()
  }

  pub fn set_dac_enabled(&mut self, enabled: bool) {
    if enabled != self.dac_enabled {
      self.dac_enabled_changed.set();
    }
    self.dac_enabled = enabled;
  }

  pub fn tick(&mut self, audio_driver: &mut dyn AudioDriver) -> CustomWavePlayerTickResult {
    if self.dac_enabled_changed.get_and_clear() && !self.dac_enabled {
      return CustomWavePlayerTickResult::DacShutOff;
    }
    if self.frequency_changed.get_and_clear() {
      let frequency = 65536.0f32 / (2048.0 - self.wavelength as f32);
      audio_driver.set_frequency(self.channel, frequency);
    }
    if self.gain_changed.get_and_clear() {
      let gain = match self.gain {
        1 => 1.0f32,
        2 => 0.5f32,
        3 => 0.25f32,
        _ => 0.0f32,
      };
      audio_driver.set_gain(self.channel, gain);
    }
    if self.triggered.get_and_clear() {
      self.playing = true;
      audio_driver.play_custom_wave(self.channel, CustomWaveOptions {
        data: self.waveform,
      });
    }
    CustomWavePlayerTickResult::Ok
  }
}