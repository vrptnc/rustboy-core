use serde::{Deserialize, Serialize};

use crate::audio::audio_driver::{AudioDriver, Channel, NoiseOptions};
use crate::util::request_flag::RequestFlag;

#[derive(Serialize, Deserialize)]
pub struct NoisePlayer {
  channel: Channel,
  pub clock_shift: u8,
  pub short: bool,
  pub clock_divider: u8,
  triggered: RequestFlag,
  pub playing: bool,
}

impl NoisePlayer {
  pub fn new(channel: Channel) -> Self {
    NoisePlayer {
      channel,
      clock_shift: 0,
      short: false,
      clock_divider: 0,
      triggered: RequestFlag::new(),
      playing: false,
    }
  }

  pub fn stop(&mut self, audio_driver: &mut dyn AudioDriver) {
    self.playing = false;
    audio_driver.stop(self.channel);
  }

  pub fn tick(&mut self, audio_driver: &mut dyn AudioDriver) {
    if self.triggered.get_and_clear() {
      self.playing = true;
      audio_driver.play_noise(self.channel, NoiseOptions {
        frequency: 262144.0 / (if self.clock_divider == 0 { 0.5 } else { self.clock_divider as f32 } * (1u8 << self.clock_shift) as f32),
        short: self.short,
      })
    }
  }

  pub fn trigger(&mut self) {
    self.triggered.set();
  }
}