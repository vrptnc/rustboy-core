use serde::{Deserialize, Serialize};

#[derive(Copy, Clone)]
pub struct PulseOptions {
  pub frequency: f32,
  pub duty_cycle: f32,
}

#[derive(Copy, Clone)]
pub struct CustomWaveOptions {
  pub data: [u8;16]
}

#[derive(Copy, Clone)]
pub struct NoiseOptions {
  pub frequency: f32,
  pub short: bool
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Channel {
  CH1,
  CH2,
  CH3,
  CH4,
}

#[derive(Copy, Clone)]
pub enum StereoChannel {
  Left,
  Right,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum DutyCycle {
  Duty125,
  Duty250,
  Duty500,
  Duty750,
}

impl DutyCycle {
  pub fn to_ratio(&self) -> f32 {
    match self {
      DutyCycle::Duty125 => 0.125,
      DutyCycle::Duty250 => 0.250,
      DutyCycle::Duty500 => 0.500,
      DutyCycle::Duty750 => 0.250
    }
  }
}

pub trait AudioDriver {
  fn play_pulse(&mut self, channel: Channel, pulse_options: PulseOptions);
  fn play_custom_wave(&mut self, channel: Channel, wave_options: CustomWaveOptions);
  fn play_noise(&mut self, channel: Channel, noise_options: NoiseOptions);
  fn stop(&mut self, channel: Channel);
  fn set_gain(&mut self, channel: Channel, gain: f32);
  fn set_stereo_gain(&mut self, channel: Channel, stereo_channel: StereoChannel, gain: f32);
  fn set_frequency(&mut self, channel: Channel, frequency: f32);

  fn mute_all(&mut self);
  fn unmute_all(&mut self);
  fn set_master_volume(&mut self, value: u8);
}