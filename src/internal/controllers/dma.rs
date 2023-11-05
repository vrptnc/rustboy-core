use log::info;
use serde::{Deserialize, Serialize};

use crate::internal::controllers::lcd::{LCDController, LCDMode};
use crate::internal::cpu::cpu::CPU;
use crate::internal::infrastructure::toggle::Toggle;
use crate::internal::memory::memory::{Memory, MemoryAddress};
use crate::internal::util::bit_util::BitUtil;

#[derive(PartialEq, Serialize, Deserialize, Debug)]
enum DMATransferType {
    Inactive,
    Legacy,
    GeneralPurpose,
    HBlank,
}

#[derive(Serialize, Deserialize)]
struct DMATransfer {
    transfer_type: DMATransferType,
    source_address: u16,
    destination_address: u16,
    bytes_transferred: u16,
    bytes_to_transfer: u16,
}

impl DMATransfer {
    pub fn inactive() -> DMATransfer {
        DMATransfer {
            transfer_type: DMATransferType::Inactive,
            source_address: 0,
            destination_address: 0,
            bytes_transferred: 0,
            bytes_to_transfer: 0,
        }
    }

    pub fn new(source_address: u16, destination_address: u16, bytes_to_transfer: u16, transfer_type: DMATransferType) -> DMATransfer {
        DMATransfer {
            transfer_type,
            source_address,
            destination_address,
            bytes_to_transfer,
            bytes_transferred: 0,
        }
    }

    pub fn legacy(source_address: u16) -> DMATransfer {
        DMATransfer {
            transfer_type: DMATransferType::Legacy,
            source_address,
            destination_address: 0,
            bytes_transferred: 0,
            bytes_to_transfer: 0,
        }
    }
}

pub trait DMAController {
    fn tick(&mut self, memory: &mut dyn Memory, cpu: &mut dyn CPU, lcd: &dyn LCDController, double_speed: bool);
}

#[derive(Serialize, Deserialize)]
pub struct DMAControllerImpl {
    dma: u8,
    high_source_address: u8,
    low_source_address: u8,
    high_destination_address: u8,
    low_destination_address: u8,
    hdma5: u8,
    active_transfer: DMATransfer,
    cancel_requested: Toggle,
    double_speed_toggle: Toggle,
}

impl DMAControllerImpl {
    pub fn new() -> DMAControllerImpl {
        DMAControllerImpl {
            dma: 0,
            high_source_address: 0,
            low_source_address: 0,
            high_destination_address: 0,
            low_destination_address: 0,
            hdma5: 0xFF,
            active_transfer: DMATransfer::inactive(),
            cancel_requested: Toggle(false),
            double_speed_toggle: Toggle(false),
        }
    }

    fn handle_legacy_transfer(&mut self, memory: &mut dyn Memory) {
        let mut bytes_transferred = self.active_transfer.bytes_transferred;
        let current_byte = memory.read(self.active_transfer.source_address + bytes_transferred);
        memory.write(0xFE00 + bytes_transferred, current_byte);
        bytes_transferred += 1;
        self.active_transfer.bytes_transferred = bytes_transferred;
        if bytes_transferred == 160 {
            self.active_transfer.transfer_type = DMATransferType::Inactive
        }
    }

    fn handle_general_purpose_transfer(&mut self, memory: &mut dyn Memory, cpu: &mut dyn CPU, double_speed: bool) {
        if double_speed && self.double_speed_toggle.inspect_and_toggle() {
            return;
        }
        cpu.disable();
        let mut bytes_transferred = self.active_transfer.bytes_transferred;
        let DMATransfer { source_address, destination_address, bytes_to_transfer, .. } = self.active_transfer;
        let current_byte = memory.read(source_address + bytes_transferred);
        memory.write(destination_address + (bytes_transferred as u16), current_byte);
        bytes_transferred += 1;
        self.active_transfer.bytes_transferred = bytes_transferred;
        if bytes_transferred == (bytes_to_transfer as u16) {
            self.active_transfer.transfer_type = DMATransferType::Inactive;
            self.hdma5 = 0xFF;
            cpu.enable();
        }
    }

    fn should_cancel_hblank_transfer(&self, cpu: &dyn CPU) -> bool {
        cpu.enabled() && self.cancel_requested.checked()
    }

    fn cancel_hblank_transfer(&mut self) {
        self.cancel_requested.clear();
        self.active_transfer.transfer_type = DMATransferType::Inactive;
        self.hdma5 = self.hdma5.set_bit(7);
    }

    fn handle_hblank_transfer(&mut self, memory: &mut dyn Memory, cpu: &mut dyn CPU, lcd: &dyn LCDController, double_speed: bool) {
        if double_speed && self.double_speed_toggle.inspect_and_toggle() {
            return;
        }
        let mut bytes_transferred = self.active_transfer.bytes_transferred;
        let DMATransfer { source_address, destination_address, bytes_to_transfer, .. } = self.active_transfer;
        if let LCDMode::HBlank = lcd.get_mode() {
            if self.should_cancel_hblank_transfer(cpu) {
                self.cancel_hblank_transfer();
                return;
            }
            cpu.disable();
            let current_byte = memory.read(source_address + bytes_transferred as u16);
            memory.write(destination_address + bytes_transferred as u16, current_byte);
            bytes_transferred += 1;
            self.active_transfer.bytes_transferred = bytes_transferred;
            if bytes_transferred == bytes_to_transfer {
                cpu.enable();
                self.active_transfer.transfer_type = DMATransferType::Inactive;
                self.cancel_requested.clear();
                self.hdma5 = 0xFF;
            } else {
                let lines_to_transfer = bytes_to_transfer / 16;
                let lines_transferred = bytes_transferred / 16;
                let lines_remaining = lines_to_transfer - lines_transferred;
                self.hdma5 = (lines_remaining - 1) as u8;
            }
        } else {
            cpu.enable();
        }
    }
}

impl DMAController for DMAControllerImpl {
    fn tick(&mut self, memory: &mut dyn Memory, cpu: &mut dyn CPU, lcd: &dyn LCDController, double_speed: bool) {
        match self.active_transfer.transfer_type {
            DMATransferType::Inactive => cpu.enable(),
            DMATransferType::Legacy => self.handle_legacy_transfer(memory),
            DMATransferType::GeneralPurpose => self.handle_general_purpose_transfer(memory, cpu, double_speed),
            DMATransferType::HBlank => self.handle_hblank_transfer(memory, cpu, lcd, double_speed),
        }
    }
}

impl Memory for DMAControllerImpl {
    fn read(&self, address: u16) -> u8 {
        match address {
            MemoryAddress::DMA => self.dma,
            MemoryAddress::HDMA1 => self.high_source_address,
            MemoryAddress::HDMA2 => self.low_source_address,
            MemoryAddress::HDMA3 => self.high_destination_address,
            MemoryAddress::HDMA4 => self.low_destination_address,
            MemoryAddress::HDMA5 => self.hdma5,
            _ => panic!("DMA can't read from address {}", address)
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            MemoryAddress::DMA => {
                self.dma = value;
                let source_address = (value as u16) * 0x100;
                info!("Setting up Legacy DMATransfer from source address {:#x}", source_address);
                self.active_transfer = DMATransfer::legacy(source_address);
            }
            MemoryAddress::HDMA1 => self.high_source_address = value,
            MemoryAddress::HDMA2 => self.low_source_address = value & 0xF0,
            MemoryAddress::HDMA3 => self.high_destination_address = (value & 0x1F) | (0x80),
            MemoryAddress::HDMA4 => self.low_destination_address = value & 0xF0,
            MemoryAddress::HDMA5 => {
                match self.active_transfer.transfer_type {
                    DMATransferType::Inactive => {
                        let source_address = ((self.high_source_address as u16) << 8) | (self.low_source_address as u16);
                        let destination_address = ((self.high_destination_address as u16) << 8) | (self.low_destination_address as u16);
                        let bytes_to_transfer = (((value & 0x7F) + 1) as u16) * 16;
                        let transfer_type = if value.get_bit(7) { DMATransferType::HBlank } else { DMATransferType::GeneralPurpose };
                        if value.get_bit(7) {
                            DMATransferType::HBlank
                        } else {
                            DMATransferType::GeneralPurpose
                        };
                        info!("Setting up {:?} DMATransfer from source address {:#x} to destination {:#x} of length {}", transfer_type, source_address, destination_address, bytes_to_transfer);
                        self.active_transfer = DMATransfer::new(
                            source_address,
                            destination_address,
                            bytes_to_transfer,
                            transfer_type,
                        );
                        self.hdma5 = 0x00;
                    }
                    DMATransferType::HBlank if !value.get_bit(7) => {
                        self.cancel_requested.check();
                    }
                    _ => {}
                }
            }
            _ => panic!("DMA can't write to address {}", address)
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_hex::assert_eq_hex;

    use crate::internal::controllers::lcd::MockLCDController;
    use crate::internal::cpu::cpu::MockCPU;
    use crate::internal::memory::memory::MemoryAddress;
    use crate::internal::memory::memory::test::MockMemory;

    use super::*;

    fn create_memory() -> MockMemory {
        let mut memory = MockMemory::new();
        for address in 0xC000u16..0xC100u16 {
            memory.write(address, address as u8);
        }
        memory
    }

    #[test]
    fn start_legacy_dma_transfer() {
        let mut dma = DMAControllerImpl::new();
        let mut memory = create_memory();
        let mut cpu = MockCPU::new();
        let mut lcd = MockLCDController::new();
        cpu.expect_enable().never();
        cpu.expect_disable().never();
        dma.write(MemoryAddress::DMA, 0xC0);
        for (index, address) in (0xFE00u16..=0xFE9Fu16).enumerate() {
            assert_eq_hex!(memory.read(address), 0x0000);
            dma.tick(&mut memory, &mut cpu, &mut lcd, false);
            assert_eq_hex!(memory.read(address), index as u8);
        }
        cpu.expect_enable().once().return_const(()); // Once DMA returns to inactive, the CPU should be (re)enabled on the next tick
        dma.tick(&mut memory, &mut cpu, &mut lcd, false);
        assert_eq_hex!(memory.read(0x8190), 0x0000);
    }

    #[test]
    fn start_general_purpose_dma_transfer() {
        let mut dma = DMAControllerImpl::new();
        let mut memory = create_memory();
        let mut cpu = MockCPU::new();
        let mut lcd = MockLCDController::new();
        dma.write(MemoryAddress::HDMA1, 0xC0);
        dma.write(MemoryAddress::HDMA2, 0x05); // 5 should be masked away
        dma.write(MemoryAddress::HDMA3, 0x01); // Should be masked with 0x1F so that result is 0x81
        dma.write(MemoryAddress::HDMA4, 0x23); // 3 should be masked away -> result is 0x20
        dma.write(MemoryAddress::HDMA5, 0x06); // Transfer 7 lines = 7 x 16 byte = 112 byte
        cpu.expect_disable().times(0x70).return_const(());
        cpu.expect_enable().once().return_const(());
        for (index, address) in (0x8120u16..=0x818Fu16).enumerate() {
            assert_eq_hex!(memory.read(address), 0x0000);
            dma.tick(&mut memory, &mut cpu, &mut lcd, false);
            assert_eq_hex!(memory.read(address), index as u8);
        }
        assert_eq_hex!(dma.read(MemoryAddress::HDMA5), 0xFF);
        cpu.expect_enable().once().return_const(()); // Once DMA returns to inactive, the CPU should be (re)enabled on the next tick
        dma.tick(&mut memory, &mut cpu, &mut lcd, false);
        assert_eq_hex!(memory.read(0x8190), 0x0000);
    }

    #[test]
    fn start_hblank_dma_transfer() {
        let mut dma = DMAControllerImpl::new();
        let mut memory = create_memory();
        let mut cpu = MockCPU::new();
        let mut lcd = MockLCDController::new();
        dma.write(MemoryAddress::HDMA1, 0xC0);
        dma.write(MemoryAddress::HDMA2, 0x05); // 5 should be masked away
        dma.write(MemoryAddress::HDMA3, 0x01); // Should be masked with 0x1F so that result is 0x81
        dma.write(MemoryAddress::HDMA4, 0x23); // 3 should be masked away -> result is 0x20
        dma.write(MemoryAddress::HDMA5, 0x86); // Transfer 7 lines = 7 x 16 byte = 112 byte

        lcd.expect_get_mode()
            .times(0x70)
            .return_const(LCDMode::Mode2);
        cpu.expect_enable()
            .times(0x70)
            .return_const(());
        for address in 0x8120u16..=0x818Fu16 {
            assert_eq_hex!(memory.read(address), 0x0000);
            dma.tick(&mut memory, &mut cpu, &mut lcd, false);
            assert_eq_hex!(memory.read(address), 0x0000);
        }

        lcd.expect_get_mode()
            .times(0x70)
            .return_const(LCDMode::HBlank);
        cpu.expect_disable()
            .times(0x70)
            .return_const(());
        cpu.expect_enabled()
            .times(0x70)
            .return_const(false); // Set the CPU to disabled for the duration of the HBlank DMA transfer
        for (index, address) in (0x8120u16..=0x818Fu16).enumerate() {
            if index == 0x6F {
                cpu.expect_enable()
                    .once()
                    .return_const(());
            }
            dma.tick(&mut memory, &mut cpu, &mut lcd, false);
            assert_eq_hex!(memory.read(address), index as u8);
        }
        assert_eq_hex!(dma.read(MemoryAddress::HDMA5), 0xFF);
        cpu.expect_enable().once().return_const(()); // Once DMA returns to inactive, the CPU should be (re)enabled on the next tick
        dma.tick(&mut memory, &mut cpu, &mut lcd, false);
        assert_eq_hex!(memory.read(0x8190), 0x0000);
    }

    #[test]
    fn cancel_hblank_dma_transfer() {
        let mut dma = DMAControllerImpl::new();
        let mut memory = create_memory();
        let mut cpu = MockCPU::new();
        let mut lcd = MockLCDController::new();

        dma.write(MemoryAddress::HDMA1, 0xC0);
        dma.write(MemoryAddress::HDMA2, 0x05); // 5 should be masked away
        dma.write(MemoryAddress::HDMA3, 0x01); // Should be masked with 0x1F so that result is 0x81
        dma.write(MemoryAddress::HDMA4, 0x23); // 3 should be masked away -> result is 0x20
        dma.write(MemoryAddress::HDMA5, 0x86); // Transfer 7 lines = 7 x 16 byte = 112 byte

        lcd.expect_get_mode()
            .times(0x20)
            .return_const(LCDMode::HBlank);
        cpu.expect_disable()
            .times(0x20)
            .return_const(());
        cpu.expect_enabled()
            .times(0x20)
            .return_const(false); // Set the CPU to disabled for the duration of the HBlank DMA transfer
        dma.tick(&mut memory, &mut cpu, &mut lcd, false); // Do a single tick to start the transfer during HBlank
        dma.write(MemoryAddress::HDMA5, 0x00); // Cancel the HBlank DMA transfer straight away
        for _ in 0usize..0x1F { // Do a number of ticks still during HBlank, during these ticks, data should still be transferred
            dma.tick(&mut memory, &mut cpu, &mut lcd, false);
        }

        lcd.expect_get_mode()
            .once()
            .return_const(LCDMode::Mode2); // Switch the mode to mode 2 to end HBlank for a minute
        cpu.expect_enable()
            .once()
            .return_const(());
        dma.tick(&mut memory, &mut cpu, &mut lcd, false);

        lcd.expect_get_mode().once().return_const(LCDMode::HBlank); // Start a new HBlank period. DMA transfer should now be cancelled
        cpu.expect_enabled()
            .once()
            .return_const(true);
        dma.tick(&mut memory, &mut cpu, &mut lcd, false);

        cpu.expect_enable()
            .once()
            .return_const(());
        dma.tick(&mut memory, &mut cpu, &mut lcd, false); // Do an extra tick to verify that there are no more writes

        assert_eq_hex!(dma.read(MemoryAddress::HDMA5), 0x84);
        assert_eq_hex!(memory.read(0x813F), 0x1F);
        assert_eq_hex!(memory.read(0x8140), 0x00);
    }
}