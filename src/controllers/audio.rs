use mockall::automock;
use serde::{Deserialize, Serialize};

use crate::audio::audio_driver::{AudioDriver, Channel, DutyCycle, StereoChannel};
use crate::audio::custom_wave_player::{CustomWavePlayer, CustomWavePlayerTickResult};
use crate::audio::gain_controller::{GainController, GainControllerTickResult};
use crate::audio::length_timer::{LengthTimer, LengthTimerTickResult};
use crate::audio::noise_player::NoisePlayer;
use crate::audio::pulse_player::{PulsePlayer, PulsePlayerTickResult};
use crate::controllers::timer::TimerController;
use crate::memory::memory::{Memory, MemoryAddress};
use crate::util::bit_util::BitUtil;
use crate::util::request_flag::RequestFlag;

//Note: Frequencies expressed in binary in the register can be converted to Hz using the formula:
// f = 131072 / (2048 - X)

#[automock]
pub trait AudioController {}

#[derive(Serialize, Deserialize)]
pub struct AudioControllerImpl {
  enabled: bool,
  disabled_request: RequestFlag,
  previous_timer_div: u8,
  div_apu: u16,
  ch1_length_timer: LengthTimer,
  ch2_length_timer: LengthTimer,
  ch3_length_timer: LengthTimer,
  ch4_length_timer: LengthTimer,
  ch1_gain_controller: GainController,
  ch1_pulse_player: PulsePlayer,
  ch2_gain_controller: GainController,
  ch2_pulse_player: PulsePlayer,
  ch3_custom_wave_player: CustomWavePlayer,
  ch4_gain_controller: GainController,
  ch4_noise_player: NoisePlayer,
  master_volume: u8,
  mixing_control: u8,
  mixing_control_changed: RequestFlag,
  waveform_ram: [u8; 16],
}

impl AudioControllerImpl {
  pub fn new() -> Self {
    let controller_impl = AudioControllerImpl {
      enabled: false,
      disabled_request: RequestFlag::new(),
      previous_timer_div: 0,
      div_apu: 0,
      ch1_length_timer: LengthTimer::new(Channel::CH1, 64),
      ch1_gain_controller: GainController::new(Channel::CH1),
      ch1_pulse_player: PulsePlayer::new(Channel::CH1),
      ch2_gain_controller: GainController::new(Channel::CH2),
      ch2_pulse_player: PulsePlayer::new(Channel::CH2),
      ch2_length_timer: LengthTimer::new(Channel::CH2, 64),
      ch3_length_timer: LengthTimer::new(Channel::CH3, 256),
      ch4_length_timer: LengthTimer::new(Channel::CH4, 64),
      ch3_custom_wave_player: CustomWavePlayer::new(Channel::CH3),
      ch4_gain_controller: GainController::new(Channel::CH4),
      ch4_noise_player: NoisePlayer::new(Channel::CH4),
      master_volume: 0,
      mixing_control: 0,
      mixing_control_changed: RequestFlag(true),
      waveform_ram: [0; 16],
    };
    controller_impl
  }

  fn length_timer_tick(&mut self, audio_driver: &mut dyn AudioDriver) {
    if let LengthTimerTickResult::Expired = self.ch1_length_timer.tick() {
      self.stop(Channel::CH1, audio_driver);
    }
    if let LengthTimerTickResult::Expired = self.ch2_length_timer.tick() {
      self.stop(Channel::CH2, audio_driver);
    }
    if let LengthTimerTickResult::Expired = self.ch3_length_timer.tick() {
      self.stop(Channel::CH3, audio_driver);
    }
    if let LengthTimerTickResult::Expired = self.ch4_length_timer.tick() {
      self.stop(Channel::CH4, audio_driver);
    }
  }

  fn gain_controller_tick(&mut self, audio_driver: &mut dyn AudioDriver) {
    if let GainControllerTickResult::DacShutOff = self.ch1_gain_controller.tick(audio_driver) {
      self.stop(Channel::CH1, audio_driver)
    }
    if let GainControllerTickResult::DacShutOff = self.ch2_gain_controller.tick(audio_driver) {
      self.stop(Channel::CH2, audio_driver)
    }
    if let GainControllerTickResult::DacShutOff = self.ch4_gain_controller.tick(audio_driver) {
      self.stop(Channel::CH4, audio_driver)
    }
  }

  fn player_tick(&mut self, audio_driver: &mut dyn AudioDriver) {
    if let PulsePlayerTickResult::WavelengthOverflowed = self.ch1_pulse_player.tick(audio_driver) {
      self.stop(Channel::CH1, audio_driver);
    }
  }

  fn set_stereo_gains(&mut self, audio_driver: &mut dyn AudioDriver) {
    [Channel::CH1, Channel::CH2, Channel::CH3, Channel::CH4].into_iter()
      .enumerate()
      .for_each(|(channel_index, channel)| {
        audio_driver.set_stereo_gain(channel, StereoChannel::Right, if self.mixing_control.get_bit(channel_index as u8) { 1.0 } else { 0.0 });
        audio_driver.set_stereo_gain(channel, StereoChannel::Left, if self.mixing_control.get_bit((channel_index + 4) as u8) { 1.0 } else { 0.0 });
      });
  }

  pub fn tick(&mut self, audio_driver: &mut dyn AudioDriver, timer: &dyn TimerController, double_speed: bool) {
    if self.disabled_request.get_and_clear() {
      self.disable(audio_driver);
    }
    if self.mixing_control_changed.get_and_clear() {
      self.set_stereo_gains(audio_driver);
    }
    if !self.enabled {
      return;
    }
    let new_timer_div = timer.get_divider().get_upper_byte();
    let divider_bit = if double_speed { 5 } else { 4 };
    if self.previous_timer_div.get_bit(divider_bit) && !new_timer_div.get_bit(divider_bit) {
      self.div_apu = self.div_apu.wrapping_add(1);
      if self.div_apu % 2 == 0 {
        self.length_timer_tick(audio_driver);
      }
      if self.div_apu % 4 == 0 {
        self.player_tick(audio_driver);
      }
      if self.div_apu % 8 == 0 {
        self.gain_controller_tick(audio_driver);
      }
    }
    if let PulsePlayerTickResult::WavelengthOverflowed = self.ch2_pulse_player.tick(audio_driver) {
      self.stop(Channel::CH2, audio_driver);
    }
    if let CustomWavePlayerTickResult::DacShutOff = self.ch3_custom_wave_player.tick(audio_driver) {
      self.stop(Channel::CH3, audio_driver);
    }
    self.ch4_noise_player.tick(audio_driver);
    self.previous_timer_div = new_timer_div;
  }

  fn trigger(&mut self, channel: Channel) {
    match channel {
      Channel::CH1 => {
        self.ch1_length_timer.trigger();
        self.ch1_gain_controller.trigger();
        self.ch1_pulse_player.trigger();
      }
      Channel::CH2 => {
        self.ch2_length_timer.trigger();
        self.ch2_gain_controller.trigger();
        self.ch2_pulse_player.trigger();
      }
      Channel::CH3 => {
        self.ch3_length_timer.trigger();
        self.ch3_custom_wave_player.trigger();
      }
      Channel::CH4 => {
        self.ch4_length_timer.trigger();
        self.ch4_gain_controller.trigger();
        self.ch4_noise_player.trigger();
      }
    }
  }

  fn stop(&mut self, channel: Channel, audio_driver: &mut dyn AudioDriver) {
    match channel {
      Channel::CH1 => {
        self.ch1_pulse_player.stop(audio_driver);
        self.ch1_length_timer.stop();
        self.ch1_gain_controller.stop();
      }
      Channel::CH2 => {
        self.ch2_pulse_player.stop(audio_driver);
        self.ch2_length_timer.stop();
        self.ch2_gain_controller.stop();
      }
      Channel::CH3 => {
        self.ch3_length_timer.stop();
        self.ch3_custom_wave_player.stop(audio_driver);
      }
      Channel::CH4 => {
        self.ch4_length_timer.stop();
        self.ch4_gain_controller.stop();
        self.ch4_noise_player.stop(audio_driver);
      }
    }
  }

  fn disable(&mut self, audio_driver: &mut dyn AudioDriver) {
    self.enabled = false;
    self.stop(Channel::CH1, audio_driver);
    self.stop(Channel::CH2, audio_driver);
    self.stop(Channel::CH3, audio_driver);
    self.stop(Channel::CH4, audio_driver);
  }
}

impl AudioController for AudioControllerImpl {}

impl Memory for AudioControllerImpl {
  fn read(&self, address: u16) -> u8 {
    match address {
      MemoryAddress::NR10 => {
        self.ch1_pulse_player.new_settings.shift |
          ((self.ch1_pulse_player.new_settings.decrease as u8) << 3) |
          (self.ch1_pulse_player.new_settings.pace << 4)
      }
      MemoryAddress::NR11 => {
        let duty_cycle_bits: u8 = match self.ch1_pulse_player.new_settings.duty_cycle {
          DutyCycle::Duty125 => 0,
          DutyCycle::Duty250 => 1,
          DutyCycle::Duty500 => 2,
          DutyCycle::Duty750 => 3
        };
        (duty_cycle_bits << 6) | (self.ch1_length_timer.new_settings.initial_value as u8)
      }
      MemoryAddress::NR12 => {
        self.ch1_gain_controller.new_settings.pace |
          ((self.ch1_gain_controller.new_settings.ascending as u8) << 3) |
          (self.ch1_gain_controller.new_settings.initial_value << 4)
      }
      MemoryAddress::NR13 => self.ch1_pulse_player.new_settings.get_lower_wavelength_bits(),
      MemoryAddress::NR14 => {
        self.ch1_pulse_player.new_settings.get_upper_wavelength_bits() |
          ((self.ch1_length_timer.enabled as u8) << 6)
      }
      0xFF15 => 0,
      MemoryAddress::NR21 => {
        let duty_cycle_bits: u8 = match self.ch2_pulse_player.new_settings.duty_cycle {
          DutyCycle::Duty125 => 0,
          DutyCycle::Duty250 => 1,
          DutyCycle::Duty500 => 2,
          DutyCycle::Duty750 => 3
        };
        (duty_cycle_bits << 6) | (self.ch2_length_timer.new_settings.initial_value as u8)
      }
      MemoryAddress::NR22 => {
        self.ch2_gain_controller.new_settings.pace |
          ((self.ch2_gain_controller.new_settings.ascending as u8) << 3) |
          (self.ch2_gain_controller.new_settings.initial_value << 4)
      }
      MemoryAddress::NR23 => self.ch2_pulse_player.new_settings.get_lower_wavelength_bits(),
      MemoryAddress::NR24 => {
        self.ch2_pulse_player.new_settings.get_upper_wavelength_bits() |
          ((self.ch2_length_timer.enabled as u8) << 6)
      }
      MemoryAddress::NR30 => if self.ch3_custom_wave_player.dac_enabled { 0x80 } else { 0 },
      MemoryAddress::NR31 => self.ch3_length_timer.new_settings.initial_value as u8,
      MemoryAddress::NR32 => self.ch3_custom_wave_player.gain << 5,
      MemoryAddress::NR33 => self.ch3_custom_wave_player.get_lower_wavelength_bits(),
      MemoryAddress::NR34 => {
        self.ch3_custom_wave_player.get_upper_wavelength_bits() |
          ((self.ch3_length_timer.enabled as u8) << 6)
      }
      0xFF1F => 0,
      MemoryAddress::NR41 => self.ch4_length_timer.new_settings.initial_value as u8,
      MemoryAddress::NR42 => self.ch4_gain_controller.new_settings.pace |
        ((self.ch4_gain_controller.new_settings.ascending as u8) << 3) |
        (self.ch4_gain_controller.new_settings.initial_value << 4),
      MemoryAddress::NR43 => (self.ch4_noise_player.clock_shift << 4) |
        ((self.ch4_noise_player.short as u8) << 3) |
        self.ch4_noise_player.clock_divider,
      MemoryAddress::NR44 => (self.ch4_length_timer.enabled as u8) << 6,
      MemoryAddress::NR50 => self.master_volume,
      MemoryAddress::NR51 => self.mixing_control,
      MemoryAddress::NR52 => {
        (self.ch1_pulse_player.playing as u8) |
          ((self.ch2_pulse_player.playing as u8) << 1) |
          ((self.ch3_custom_wave_player.playing as u8) << 2) |
          ((self.ch4_noise_player.playing as u8) << 3) |
          ((self.enabled as u8) << 7)
      }
      0xFF27..=0xFF2F => 0,
      0xFF30..=0xFF3F => self.waveform_ram[address as usize - 0xFF30],
      _ => panic!("AudioController can't read from address {}", address)
    }
  }

  fn write(&mut self, address: u16, value: u8) {
    match address {
      MemoryAddress::NR10 => {
        self.ch1_pulse_player.new_settings.shift = value & 0x7;
        self.ch1_pulse_player.new_settings.decrease = value.get_bit(3);
        self.ch1_pulse_player.set_pace((value >> 4) & 0x7);
      }
      MemoryAddress::NR11 => {
        let duty_cycle_bits = value >> 6;
        self.ch1_pulse_player.new_settings.duty_cycle = match duty_cycle_bits {
          0 => DutyCycle::Duty125,
          1 => DutyCycle::Duty250,
          2 => DutyCycle::Duty500,
          _ => DutyCycle::Duty750,
        };
        self.ch1_length_timer.new_settings.initial_value = (value & 0x3F) as u16;
      }
      MemoryAddress::NR12 => {
        self.ch1_gain_controller.new_settings.pace = value & 0x7;
        self.ch1_gain_controller.new_settings.ascending = value.get_bit(3);
        self.ch1_gain_controller.new_settings.initial_value = value >> 4;
      }
      MemoryAddress::NR13 => {
        self.ch1_pulse_player.new_settings.set_lower_wavelength_bits(value);
      }
      MemoryAddress::NR14 => {
        self.ch1_pulse_player.new_settings.set_upper_wavelength_bits(value);
        self.ch1_length_timer.enabled = value.get_bit(6);
        if value.get_bit(7) {
          self.trigger(Channel::CH1);
        }
      }
      0xFF15 => {}
      MemoryAddress::NR21 => {
        let duty_cycle_bits = value >> 6;
        self.ch2_pulse_player.new_settings.duty_cycle = match duty_cycle_bits {
          0 => DutyCycle::Duty125,
          1 => DutyCycle::Duty250,
          2 => DutyCycle::Duty500,
          _ => DutyCycle::Duty750,
        };
        self.ch2_length_timer.new_settings.initial_value = (value & 0x3F) as u16;
      }
      MemoryAddress::NR22 => {
        self.ch2_gain_controller.new_settings.pace = value & 0x7;
        self.ch2_gain_controller.new_settings.ascending = value.get_bit(3);
        self.ch2_gain_controller.new_settings.initial_value = value >> 4;
      }
      MemoryAddress::NR23 => {
        self.ch2_pulse_player.new_settings.set_lower_wavelength_bits(value);
      }
      MemoryAddress::NR24 => {
        self.ch2_pulse_player.new_settings.set_upper_wavelength_bits(value);
        self.ch2_length_timer.enabled = value.get_bit(6);
        if value.get_bit(7) {
          self.trigger(Channel::CH2);
        }
      }
      MemoryAddress::NR30 => {
        self.ch3_custom_wave_player.set_dac_enabled(value.get_bit(7));
      }
      MemoryAddress::NR31 => {
        self.ch3_length_timer.new_settings.initial_value = value as u16;
      }
      MemoryAddress::NR32 => {
        let gain = (value >> 5) & 0x3;
        self.ch3_custom_wave_player.set_gain(gain);
      }
      MemoryAddress::NR33 => {
        self.ch3_custom_wave_player.set_lower_wavelength_bits(value);
      }
      MemoryAddress::NR34 => {
        self.ch3_custom_wave_player.set_upper_wavelength_bits(value);
        self.ch3_length_timer.enabled = value.get_bit(6);
        if value.get_bit(7) {
          self.trigger(Channel::CH3);
        }
      }
      0xFF1F => {}
      MemoryAddress::NR41 => {
        self.ch4_length_timer.new_settings.initial_value = (value & 0x3F) as u16;
      }
      MemoryAddress::NR42 => {
        self.ch4_gain_controller.new_settings.pace = value & 0x7;
        self.ch4_gain_controller.new_settings.ascending = value.get_bit(3);
        self.ch4_gain_controller.new_settings.initial_value = value >> 4;
      }
      MemoryAddress::NR43 => {
        self.ch4_noise_player.clock_divider = value & 0x7;
        self.ch4_noise_player.short = value.get_bit(3);
        self.ch4_noise_player.clock_shift = value >> 4;
      }
      MemoryAddress::NR44 => {
        self.ch4_length_timer.enabled = value.get_bit(6);
        if value.get_bit(7) {
          self.trigger(Channel::CH4);
        }
      }
      MemoryAddress::NR50 => self.master_volume = value,
      MemoryAddress::NR51 => {
        self.mixing_control = value;
        self.mixing_control_changed.set();
      }
      MemoryAddress::NR52 => {
        if !value.get_bit(7) {
          self.disabled_request.set();
        } else {
          self.enabled = true;
        }
      }
      0xFF27..=0xFF2F => {}
      0xFF30..=0xFF3F => self.ch3_custom_wave_player.waveform[address as usize - 0xFF30] = value,
      _ => panic!("AudioController can't write to address {}", address)
    }
  }
}