use serde::{Deserialize, Serialize};

use crate::audio::audio_driver::Channel;

pub enum LengthTimerTickResult {
  Ok,
  Expired,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct LengthTimerSettings {
  pub initial_value: u16,
}

impl LengthTimerSettings {
  pub fn new() -> Self {
    LengthTimerSettings {
      initial_value: 0
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct LengthTimer {
  channel: Channel,
  current_value: u16,
  max_value: u16,
  current_settings: LengthTimerSettings,
  pub new_settings: LengthTimerSettings,
  pub enabled: bool,
  counting: bool,
}

impl LengthTimer {
  pub fn new(channel: Channel, max_value: u16) -> Self {
    LengthTimer {
      channel,
      current_value: 0,
      max_value,
      current_settings: LengthTimerSettings::new(),
      new_settings: LengthTimerSettings::new(),
      enabled: false,
      counting: false,
    }
  }

  pub fn stop(&mut self) {
    self.counting = false;
  }

  pub fn trigger(&mut self) {
    self.current_settings = self.new_settings;
    self.current_value = self.max_value - self.current_settings.initial_value;
    self.counting = true;
  }

  pub fn tick(&mut self) -> LengthTimerTickResult {
    if self.counting && self.enabled {
      self.current_value = self.current_value.saturating_sub(1);
      if self.current_value == 0 { LengthTimerTickResult::Expired } else { LengthTimerTickResult::Ok }
    } else {
      LengthTimerTickResult::Ok
    }
  }
}