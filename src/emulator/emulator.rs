use std::cell::RefCell;
use std::io::Cursor;
use std::panic;
use std::rc::Rc;

use bincode::{deserialize_from, serialize_into};

use crate::audio::audio_driver::AudioDriver;
use crate::controllers::audio::AudioControllerImpl;
use crate::controllers::buttons::{Button, ButtonController, ButtonControllerImpl};
use crate::controllers::dma::{DMAController, DMAControllerImpl};
use crate::controllers::lcd::LCDControllerImpl;
use crate::controllers::speed::{SpeedController, SpeedControllerImpl};
use crate::controllers::timer::{TimerController, TimerControllerImpl};
use crate::cpu::cpu::{CPU, CPUImpl, CPUInfo};
use crate::cpu::interrupts::InterruptControllerImpl;
use crate::emulator::compatibility_palette::CompatibilityPaletteLoader;
use crate::memory::bus::MemoryBus;
use crate::memory::control::ControlRegisters;
use crate::memory::cram::{CRAM, CRAMImpl};
use crate::memory::dma_bus::DMAMemoryBus;
use crate::memory::linear_memory::LinearMemory;
use crate::memory::mbc0::MBC0;
use crate::memory::mbc1::MBC1;
use crate::memory::mbc2::MBC2;
use crate::memory::mbc3::MBC3;
use crate::memory::mbc5::MBC5;
use crate::memory::mbc::MBC;
use crate::memory::memory::{CGBMode, Memory, MemoryAddress, RAMSize, ROMSize};
use crate::memory::oam::{OAM, OAMImpl, OAMObject, ObjectReference};
use crate::memory::stack::Stack;
use crate::memory::unmapped::UnmappedMemory;
use crate::memory::vram::VRAMImpl;
use crate::memory::wram::WRAMImpl;
use crate::renderer::renderer::{Renderer, RenderTarget};
use crate::util::instruction_label_provider::InstructionLabelProvider;

pub struct Emulator<A: AudioDriver, R: Renderer> {
    rom: Rc<RefCell<dyn MBC>>,
    cpu: CPUImpl,
    cram: CRAMImpl,
    vram: VRAMImpl,
    wram: WRAMImpl,
    oam: OAMImpl,
    lcd: LCDControllerImpl,
    timer: TimerControllerImpl,
    dma: DMAControllerImpl,
    renderer: R,
    interrupt_controller: InterruptControllerImpl,
    speed_controller: SpeedControllerImpl,
    button_controller: ButtonControllerImpl,
    audio_controller: AudioControllerImpl,
    stack: Stack,
    control_registers: ControlRegisters,
    reserved_area_1: LinearMemory<0x1E00, 0xE000>,
    reserved_area_2: LinearMemory<0x0060, 0xFEA0>,
    unmapped_memory: UnmappedMemory,
    audio_driver: A,
    paused: bool,
}

impl<A: AudioDriver, R: Renderer> Emulator<A, R> {
    pub fn new(rom_bytes: &[u8], audio_driver: A, renderer: R) -> Self {
        let rom_size = ROMSize::from_byte(rom_bytes[0x0148]);
        let ram_size = RAMSize::from_byte(rom_bytes[0x0149]);
        let rom = Emulator::<A, R>::create_rom(rom_bytes, rom_size, ram_size);
        let compatibility_byte = (*rom).borrow().compatibility_byte();
        let cgb_mode = CGBMode::from_byte(compatibility_byte);
        let mut cpu = CPUImpl::new();
        cpu.init();
        let mut cram = CRAMImpl::new();
        let vram = VRAMImpl::new();
        let wram = WRAMImpl::new();
        let oam = OAMImpl::new();
        let mut lcd = LCDControllerImpl::new(cgb_mode);
        let mut timer = TimerControllerImpl::new();
        timer.write(MemoryAddress::TAC, 0xF8);
        let dma = DMAControllerImpl::new();
        let button_controller = ButtonControllerImpl::new();
        let audio_controller = AudioControllerImpl::new();
        let stack = Stack::new();
        let mut control_registers = ControlRegisters::new();
        let reserved_area_1 = LinearMemory::<0x1E00, 0xE000>::new();
        let reserved_area_2 = LinearMemory::<0x0060, 0xFEA0>::new();
        let interrupt_controller = InterruptControllerImpl::new();
        let speed_controller = SpeedControllerImpl::new();
        let unmapped_memory = UnmappedMemory::new();

        // If we're in compatibility/color mode, write the compatibility flag as is to KEY0
        // otherwise, write 0x04 to KEY0 and set the OPRI flag on the LCD to 0x01
        if matches!(cgb_mode, CGBMode::Color) {
            control_registers.write(MemoryAddress::KEY0, compatibility_byte);
        } else {
            let compatibility_palettes = CompatibilityPaletteLoader::get_compatibility_palettes(Rc::clone(&rom));
            cram.write_compatibility_palettes(compatibility_palettes);
            control_registers.write(MemoryAddress::KEY0, 0x04);
            lcd.write(MemoryAddress::OPRI, 0x01);
        }

        // Write 0x11 to BANK to indicate we're unmapping the boot rom
        control_registers.write(MemoryAddress::BANK, 0x11);

        Emulator {
            cpu,
            rom,
            cram,
            vram,
            wram,
            oam,
            lcd,
            timer,
            dma,
            stack,
            button_controller,
            audio_controller,
            control_registers,
            reserved_area_1,
            reserved_area_2,
            interrupt_controller,
            speed_controller,
            renderer,
            unmapped_memory,
            audio_driver,
            paused: false,
        }
    }

    pub fn get_state(&self) -> Result<Vec<u8>, String> {
        let mut buffer: Vec<u8> = Vec::new();

        fn stringify_error(error: bincode::Error) -> String { format!("Error while serializing: {:?}", error) }

        serialize_into(&mut buffer, &self.cpu).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.cram).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.vram).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.wram).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.oam).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.lcd).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.timer).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.dma).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.stack).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.button_controller).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.audio_controller).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.control_registers).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.reserved_area_1).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.reserved_area_2).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.interrupt_controller).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.speed_controller).map_err(stringify_error)?;
        serialize_into(&mut buffer, &self.unmapped_memory).map_err(stringify_error)?;
        Ok(buffer)
    }

    pub fn load_state(&mut self, buffer: &[u8]) {
        let mut cursor = Cursor::new(buffer);
        self.cpu = deserialize_from(&mut cursor).unwrap();
        self.cram = deserialize_from(&mut cursor).unwrap();
        self.vram = deserialize_from(&mut cursor).unwrap();
        self.wram = deserialize_from(&mut cursor).unwrap();
        self.oam = deserialize_from(&mut cursor).unwrap();
        self.lcd = deserialize_from(&mut cursor).unwrap();
        self.timer = deserialize_from(&mut cursor).unwrap();
        self.dma = deserialize_from(&mut cursor).unwrap();
        self.stack = deserialize_from(&mut cursor).unwrap();
        self.button_controller = deserialize_from(&mut cursor).unwrap();
        self.audio_controller = deserialize_from(&mut cursor).unwrap();
        self.control_registers = deserialize_from(&mut cursor).unwrap();
        self.reserved_area_1 = deserialize_from(&mut cursor).unwrap();
        self.reserved_area_2 = deserialize_from(&mut cursor).unwrap();
        self.interrupt_controller = deserialize_from(&mut cursor).unwrap();
        self.speed_controller = deserialize_from(&mut cursor).unwrap();
        self.unmapped_memory = deserialize_from(&mut cursor).unwrap();
    }

    fn create_rom(rom_bytes: &[u8], rom_size: ROMSize, ram_size: RAMSize) -> Rc<RefCell<dyn MBC>> {
        let rom: Rc<RefCell<dyn MBC>> = match rom_bytes[0x0147] {
            0x00 => Rc::new(RefCell::new(MBC0::new(rom_size))),
            0x01..=0x03 => Rc::new(RefCell::new(MBC1::new(rom_size, ram_size))),
            0x05..=0x06 => Rc::new(RefCell::new(MBC2::new(rom_size))),
            0x0B..=0x0D => panic!("This emulator currently does not support MMM01 cartridges"),
            0x0F..=0x13 => Rc::new(RefCell::new(MBC3::new(rom_size, ram_size))),
            0x19..=0x1E => Rc::new(RefCell::new(MBC5::new(rom_size, ram_size))),
            0x20 => panic!("This emulator currently does not support MBC6 cartridges"),
            0x22 => panic!("This emulator currently does not support MBC7 cartridges"),
            0xFC => panic!("This emulator currently does not support Pocket Camera cartridges"),
            0xFD => panic!("This emulator currently does not support Bandai cartridges"),
            0xFE => panic!("This emulator currently does not support HuC3 cartridges"),
            0xFF => panic!("This emulator currently does not support HuC1 cartridges"),
            _ => panic!("This emulator does not support cartridges with a type byte of {:#x}", rom_bytes[0x0147])
        };
        (*rom).borrow_mut().load_bytes(0x0000, rom_bytes);
        rom
    }

    pub fn press_button(&mut self, button: Button) {
        self.button_controller.press_button(button, &mut self.interrupt_controller);
    }

    pub fn release_button(&mut self, button: Button) {
        self.button_controller.release_button(button);
    }

    pub fn set_tile_atlas_rendering_enabled(&mut self, enabled: bool) {
        self.renderer.set_render_target_enabled(RenderTarget::TileAtlas, enabled);
    }

    pub fn set_object_atlas_rendering_enabled(&mut self, enabled: bool) {
        self.renderer.set_render_target_enabled(RenderTarget::ObjectAtlas, enabled);
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn cpu_info(&self) -> CPUInfo {
        self.cpu.cpu_info()
    }

    pub fn get_instruction_label(&mut self, address: u16) -> String {
        let memory_bus = MemoryBus {
            rom: Rc::clone(&self.rom),
            vram: &mut self.vram,
            wram: &mut self.wram,
            reserved_area_1: &mut self.reserved_area_1,
            oam: &mut self.oam,
            reserved_area_2: &mut self.reserved_area_2,
            button_controller: &mut self.button_controller,
            timer: &mut self.timer,
            interrupt_controller: &mut self.interrupt_controller,
            speed_controller: &mut self.speed_controller,
            audio_controller: &mut self.audio_controller,
            lcd: &mut self.lcd,
            dma: &mut self.dma,
            cram: &mut self.cram,
            control_registers: &mut self.control_registers,
            stack: &mut self.stack,
            unmapped_memory: &mut self.unmapped_memory,
        };
        InstructionLabelProvider::get_label(&memory_bus, address)
    }

    pub fn get_object(&self, object_index: u8) -> OAMObject {
        self.oam.get_object(ObjectReference {
            object_index,
            use_bottom_tile: false,
        }, self.lcd.use_8_x_16_tiles())
    }

    pub fn tick(&mut self) {
        let double_speed = self.speed_controller.double_speed();
        let mut memory_bus = MemoryBus {
            rom: Rc::clone(&self.rom),
            vram: &mut self.vram,
            wram: &mut self.wram,
            reserved_area_1: &mut self.reserved_area_1,
            oam: &mut self.oam,
            reserved_area_2: &mut self.reserved_area_2,
            button_controller: &mut self.button_controller,
            timer: &mut self.timer,
            interrupt_controller: &mut self.interrupt_controller,
            speed_controller: &mut self.speed_controller,
            audio_controller: &mut self.audio_controller,
            lcd: &mut self.lcd,
            dma: &mut self.dma,
            cram: &mut self.cram,
            control_registers: &mut self.control_registers,
            stack: &mut self.stack,
            unmapped_memory: &mut self.unmapped_memory,
        };
        self.cpu.tick(&mut memory_bus);
        (*self.rom).borrow_mut().tick(double_speed);
        self.speed_controller.tick(&mut self.cpu);
        self.button_controller.tick(&mut self.interrupt_controller);
        self.audio_controller.tick(&mut self.audio_driver, &mut self.timer, double_speed);
        self.timer.tick(&mut self.interrupt_controller);
        self.lcd.tick(&self.vram, &self.cram, &self.oam, &mut self.renderer, &mut self.interrupt_controller, double_speed);
        let mut dma_memory_bus = DMAMemoryBus {
            rom: Rc::clone(&self.rom),
            vram: &mut self.vram,
            wram: &mut self.wram,
            oam: &mut self.oam,
        };
        self.dma.tick(&mut dma_memory_bus, &mut self.cpu, &self.lcd, double_speed);
    }

    pub fn execute_machine_cycle(&mut self) {
        self.tick();
    }

    pub fn run_for_nanos(&mut self, nanos: u64) {
        if !self.paused {
            let mut remaining_nanos = nanos;
            while remaining_nanos > 0 {
                let double_speed = self.speed_controller.double_speed();
                remaining_nanos = remaining_nanos.saturating_sub(if double_speed { 500 } else { 1000 });
                self.tick();
            }
        }
    }
}