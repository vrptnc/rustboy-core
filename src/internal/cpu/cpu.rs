use std::collections::VecDeque;

use byteorder::{LittleEndian, ReadBytesExt};
use mockall::automock;
use serde::{Deserialize, Serialize};

use crate::cpu::CPUInfo;
use crate::internal::cpu::decoder::{InstructionDecoder, InstructionScheduler};
use crate::internal::cpu::instruction::{ByteArithmeticParams, ByteCastingParams, ByteLocation, ByteLogicParams, ByteOperationParams, ByteRotationParams, ByteShiftParams, Instruction, WordArithmeticParams, WordLocation, WordOperationParams};
use crate::internal::cpu::interrupts::Interrupt;
use crate::internal::cpu::opcode::Opcode;
use crate::internal::cpu::register::{ByteRegister, Registers, WordRegister};
use crate::internal::memory::memory::{Memory, MemoryAddress};
use crate::internal::util::bit_util::BitUtil;

#[automock]
pub trait CPU {
    fn tick<'b, 'a>(&'a mut self, memory: &'b mut dyn Memory);
    fn enabled(&self) -> bool;
    fn enable(&mut self);
    fn disable(&mut self);
    fn stopped(&self) -> bool;
    fn resume(&mut self);
    fn cpu_info(&self) -> CPUInfo;
}

#[derive(Serialize, Deserialize)]
struct InstructionContext {
    byte_buffer: u8,
    word_buffer: u16,
    address_buffer: u16,
}

#[derive(Serialize, Deserialize)]
pub struct CPUImpl {
    enabled: bool,
    stopped: bool,
    context: InstructionContext,
    instructions: VecDeque<Instruction>,
    registers: Registers,
}

impl CPU for CPUImpl {
    fn tick<'a, 'b>(&'a mut self, memory: &'b mut dyn Memory) {
        if self.stopped {
            let optional_interrupt = Interrupt::from_bit(memory.read(MemoryAddress::RI));
            if let Some(Interrupt::ButtonPressed) = optional_interrupt {
                self.resume();
                InstructionDecoder::schedule_call_interrupt_routine(self, Interrupt::ButtonPressed);
            }
        } else if !self.instructions.is_empty() {
            self.execute_machine_cycle(memory);
        } else if self.enabled {
            let optional_interrupt = Interrupt::from_bit(memory.read(MemoryAddress::RI));
            if let Some(interrupt) = optional_interrupt {
                if let Interrupt::ButtonPressed = interrupt {
                    self.resume();
                }
                InstructionDecoder::schedule_call_interrupt_routine(self, interrupt);
            } else {
                self.decode_instruction(memory);
            }
            self.execute_machine_cycle(memory);
        }
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn enable(&mut self) {
        self.enabled = true;
    }

    fn disable(&mut self) {
        self.enabled = false;
    }

    fn stopped(&self) -> bool {
        self.stopped
    }

    fn resume(&mut self) {
        self.stopped = false;
    }

    fn cpu_info(&self) -> CPUInfo {
        CPUInfo {
            af: self.registers.read_word(WordRegister::AF),
            bc: self.registers.read_word(WordRegister::BC),
            de: self.registers.read_word(WordRegister::DE),
            hl: self.registers.read_word(WordRegister::HL),
            sp: self.registers.read_word(WordRegister::SP),
            pc: self.registers.read_word(WordRegister::PC),
            stopped: self.stopped,
            enabled: self.enabled,
        }
    }
}

impl InstructionScheduler for CPUImpl {
    fn schedule(&mut self, instruction: Instruction) {
        self.instructions.push_back(instruction);
    }
}

impl CPUImpl {
    pub fn new() -> CPUImpl {
        CPUImpl {
            enabled: true,
            stopped: false,
            context: InstructionContext {
                byte_buffer: 0u8,
                word_buffer: 0u16,
                address_buffer: 0u16,
            },
            instructions: VecDeque::with_capacity(20),
            registers: Registers::new(),
        }
    }

    pub fn init(&mut self) {
        self.registers.write_word(WordRegister::PC, 0x0100);
    }

    fn pop_branch_instructions(&mut self) {
        while let Some(instruction) = self.instructions.pop_front() {
            if let Instruction::EndBranch = instruction {
                break;
            }
        }
    }

    fn execute_instruction(&mut self, instruction: Instruction, memory: &mut dyn Memory) {
        match instruction {
            Instruction::Noop => {}
            Instruction::Defer => panic!("Defer instructions should never be executed"),
            Instruction::BranchIfZero => {
                if !self.registers.read_byte(ByteRegister::F).get_bit(7) {
                    self.pop_branch_instructions();
                }
            }
            Instruction::BranchIfNotZero => {
                if self.registers.read_byte(ByteRegister::F).get_bit(7) {
                    self.pop_branch_instructions();
                }
            }
            Instruction::BranchIfCarry => {
                if !self.registers.read_byte(ByteRegister::F).get_bit(4) {
                    self.pop_branch_instructions();
                }
            }
            Instruction::BranchIfNotCarry => {
                if self.registers.read_byte(ByteRegister::F).get_bit(4) {
                    self.pop_branch_instructions();
                }
            }
            Instruction::EndBranch => {}
            Instruction::MoveByte(params) => { self.move_byte(params, memory); }
            Instruction::CastByteToSignedWord(params) => { self.cast_byte_to_signed_word(params, memory); }
            Instruction::MoveWord(params) => { self.move_word(params); }
            Instruction::IncrementWord(location) => { self.increment_word(location); }
            Instruction::DecrementWord(location) => { self.decrement_word(location); }
            Instruction::AddBytes(params) => { self.add_bytes(params, memory); }
            Instruction::SubtractBytes(params) => { self.subtract_bytes(params, memory); }
            Instruction::AndBytes(params) => { self.and_bytes(params, memory); }
            Instruction::OrBytes(params) => { self.or_bytes(params, memory); }
            Instruction::XorBytes(params) => { self.xor_bytes(params, memory); }
            Instruction::OnesComplementByte(params) => { self.ones_complement_byte(params, memory); }
            Instruction::RotateByteLeft(params) => { self.rotate_byte_left(params, memory); }
            Instruction::RotateByteLeftThroughCarry(params) => { self.rotate_byte_left_through_carry(params, memory); }
            Instruction::ShiftByteLeft(params) => { self.shift_byte_left(params, memory); }
            Instruction::RotateByteRight(params) => { self.rotate_byte_right(params, memory); }
            Instruction::RotateByteRightThroughCarry(params) => { self.rotate_byte_right_through_carry(params, memory); }
            Instruction::ShiftByteRight(params) => { self.shift_byte_right(params, memory); }
            Instruction::SwapByte(params) => { self.swap_byte(params, memory); }
            Instruction::AddWords(params) => { self.add_words(params); }
            Instruction::DecimalAdjust => { self.decimal_adjust_reg_a(memory); }
            Instruction::GetBitFromByte(location, bit_number) => { self.get_bit_from_byte(location, bit_number, memory); }
            Instruction::SetBitOnByte(params, bit_number) => { self.set_bit_on_byte(params, bit_number, memory); }
            Instruction::ResetBitOnByte(params, bit_number) => { self.reset_bit_on_byte(params, bit_number, memory); }
            Instruction::ClearInterrupt(interrupt) => {
                let interrupt_request = memory.read(MemoryAddress::IF);
                memory.write(MemoryAddress::IF, interrupt_request.reset_bit(interrupt.get_bit()));
            }
            Instruction::EnableInterrupts => { memory.write(MemoryAddress::IME, 0x01); }
            Instruction::DisableInterrupts => { memory.write(MemoryAddress::IME, 0x00); }
            Instruction::FlipCarry => { self.flip_carry_flag(); }
            Instruction::SetCarry => { self.set_carry_flag(); }
            Instruction::Halt => { self.halt(); }
            Instruction::Stop => { self.stop(); }
            Instruction::DecodeCBInstruction => {
                let opcode = Opcode(self.read_next_byte(memory));
                InstructionDecoder::decode_cb(self, opcode);
            }
        }
    }

    fn execute_machine_cycle(&mut self, memory: &mut dyn Memory) {
        while let Some(instruction) = self.instructions.pop_front() {
            if let Instruction::Defer = instruction {
                return;
            }
            self.execute_instruction(instruction, memory);
        }
    }

    fn decode_instruction(&mut self, memory: &mut dyn Memory) {
        let opcode = Opcode(self.read_next_byte(memory));
        InstructionDecoder::decode(self, opcode);
    }

    fn read_next_byte(&mut self, memory: &dyn Memory) -> u8 {
        let address = self.registers.read_word(WordRegister::PC);
        self.registers.write_word(WordRegister::PC, address + 1);
        memory.read(address)
    }

    fn read_byte(&mut self, memory: &dyn Memory, location: ByteLocation) -> u8 {
        match location {
            ByteLocation::Value(value) => value,
            ByteLocation::Register(register) => self.registers.read_byte(register),
            ByteLocation::ByteBuffer => self.context.byte_buffer,
            ByteLocation::LowerAddressBuffer => self.context.address_buffer as u8,
            ByteLocation::UpperAddressBuffer => (self.context.address_buffer >> 8) as u8,
            ByteLocation::LowerWordBuffer => self.context.word_buffer as u8,
            ByteLocation::UpperWordBuffer => (self.context.word_buffer >> 8) as u8,
            ByteLocation::MemoryReferencedByAddressBuffer => memory.read(self.context.address_buffer),
            ByteLocation::MemoryReferencedByRegister(register) => memory.read(self.registers.read_word(register)),
            ByteLocation::NextMemoryByte => self.read_next_byte(memory),
        }
    }

    fn write_byte(&mut self, memory: &mut dyn Memory, location: ByteLocation, value: u8) {
        match location {
            ByteLocation::Register(register) => self.registers.write_byte(register, value),
            ByteLocation::ByteBuffer => self.context.byte_buffer = value,
            ByteLocation::LowerAddressBuffer => self.context.address_buffer = (self.context.address_buffer & 0xFF00) + (value as u16),
            ByteLocation::UpperAddressBuffer => self.context.address_buffer = (self.context.address_buffer & 0x00FF) + ((value as u16) << 8),
            ByteLocation::LowerWordBuffer => self.context.word_buffer = (self.context.word_buffer & 0xFF00) + (value as u16),
            ByteLocation::UpperWordBuffer => self.context.word_buffer = (self.context.word_buffer & 0x00FF) + ((value as u16) << 8),
            ByteLocation::MemoryReferencedByAddressBuffer => memory.write(self.context.address_buffer, value),
            ByteLocation::MemoryReferencedByRegister(register) => memory.write(self.registers.read_word(register), value),
            ByteLocation::NextMemoryByte => panic!("Can't write byte to next memory location"),
            ByteLocation::Value(_) => panic!("Can't write to passed value")
        }
    }

    fn read_word(&self, location: WordLocation) -> u16 {
        match location {
            WordLocation::Value(value) => value,
            WordLocation::Register(register) => self.registers.read_word(register),
            WordLocation::WordBuffer => self.context.word_buffer,
            WordLocation::AddressBuffer => self.context.address_buffer,
        }
    }

    fn write_word(&mut self, location: WordLocation, value: u16) {
        match location {
            WordLocation::Register(register) => self.registers.write_word(register, value),
            WordLocation::WordBuffer => self.context.word_buffer = value,
            WordLocation::AddressBuffer => self.context.address_buffer = value,
            WordLocation::Value(_) => panic!("Can't write to passed value")
        }
    }

    fn move_byte(&mut self, params: ByteOperationParams, memory: &mut dyn Memory) {
        let byte = self.read_byte(memory, params.source);
        self.write_byte(memory, params.destination, byte);
    }

    fn move_word(&mut self, params: WordOperationParams) {
        let word = self.read_word(params.source);
        self.write_word(params.destination, word);
    }

    fn cast_byte_to_signed_word(&mut self, params: ByteCastingParams, memory: &mut dyn Memory) {
        let signed_word = self.read_byte(memory, params.source) as i8 as u16;
        self.write_word(params.destination, signed_word)
    }

    fn add_bytes(&mut self, params: ByteArithmeticParams, memory: &mut dyn Memory) {
        let first_value = self.read_byte(memory, params.first) as u16;
        let second_value = self.read_byte(memory, params.second) as u16;
        let carry = if params.use_carry { self.registers.read_byte(ByteRegister::F).get_bit(4) as u16 } else { 0u16 };
        let result = first_value + second_value + carry;
        let carry_result = first_value ^ second_value ^ result;
        let truncated_result = result as u8;
        let zero = truncated_result == 0;
        if params.flag_mask != 0 {
            let flag =
                ((zero as u8) << 7) |
                    ((carry_result.get_bit(4) as u8) << 5) |
                    ((carry_result.get_bit(8) as u8) << 4);
            self.registers.write_byte_masked(ByteRegister::F, flag, params.flag_mask);
        }
        self.write_byte(memory, params.destination, truncated_result);
    }

    fn add_words(&mut self, params: WordArithmeticParams) {
        let first_value = self.read_word(params.first);
        let second_value = self.read_word(params.second);
        let le_bytes1 = first_value.to_le_bytes();
        let le_bytes2 = second_value.to_le_bytes();
        let (result1, carry1) = le_bytes1[0].overflowing_add(le_bytes2[0]);
        let result2 = (le_bytes1[1] as u16) + (le_bytes2[1] as u16) + (carry1 as u16);
        let carry_result2 = (le_bytes1[1] as u16) ^ (le_bytes2[1] as u16) ^ result2;
        let result = (&[result1, result2 as u8][..]).read_u16::<LittleEndian>().unwrap();
        if params.set_flag {
            let flag = ((carry_result2.get_bit(4) as u8) << 5) | ((carry_result2.get_bit(8) as u8) << 4);
            self.registers.write_byte_masked(ByteRegister::F, flag, if params.reset_zero_flag { 0xF0 } else { 0x70 });
        }
        self.write_word(params.destination, result);
    }

    fn subtract_bytes(&mut self, params: ByteArithmeticParams, memory: &mut dyn Memory) {
        let first_value = self.read_byte(memory, params.first);
        let second_value = self.read_byte(memory, params.second);
        let borrow = if params.use_carry { self.registers.read_byte(ByteRegister::F).get_bit(4) as u16 } else { 0u16 };
        let result = 0x100u16 + (first_value as u16) - (second_value as u16) - borrow;
        let borrow_result = (0x100u16 + first_value as u16) ^ (second_value as u16) ^ result;
        let truncated_result = result as u8;
        let zero = truncated_result == 0;
        if params.flag_mask != 0 {
            let flag =
                ((zero as u8) << 7) |
                    (1u8 << 6) |
                    ((borrow_result.get_bit(4) as u8) << 5) |
                    ((borrow_result.get_bit(8) as u8) << 4);
            self.registers.write_byte_masked(ByteRegister::F, flag, params.flag_mask);
        }
        self.write_byte(memory, params.destination, truncated_result);
    }

    fn and_bytes(&mut self, params: ByteLogicParams, memory: &mut dyn Memory) {
        let first_value = self.read_byte(memory, params.first);
        let second_value = self.read_byte(memory, params.second);
        let result = first_value & second_value;
        let zero = result == 0;
        let flag = ((zero as u8) << 7) | (1u8 << 5);
        self.registers.write_byte(ByteRegister::F, flag);
        self.write_byte(memory, params.destination, result);
    }

    fn or_bytes(&mut self, params: ByteLogicParams, memory: &mut dyn Memory) {
        let first_value = self.read_byte(memory, params.first);
        let second_value = self.read_byte(memory, params.second);
        let result = first_value | second_value;
        let flag = if result == 0 { 0x80u8 } else { 0x00u8 };
        self.registers.write_byte(ByteRegister::F, flag);
        self.write_byte(memory, params.destination, result);
    }

    fn xor_bytes(&mut self, params: ByteLogicParams, memory: &mut dyn Memory) {
        let first_value = self.read_byte(memory, params.first);
        let second_value = self.read_byte(memory, params.second);
        let result = first_value ^ second_value;
        let flag = if result == 0 { 0x80u8 } else { 0x00u8 };
        self.registers.write_byte(ByteRegister::F, flag);
        self.write_byte(memory, params.destination, result);
    }

    fn ones_complement_byte(&mut self, params: ByteOperationParams, memory: &mut dyn Memory) {
        let byte = self.read_byte(memory, params.source);
        self.write_byte(memory, params.destination, !byte);
        self.registers.write_byte_masked(ByteRegister::F, 0x60, 0x60);
    }

    fn get_bit_from_byte(&mut self, location: ByteLocation, bit_number: u8, memory: &mut dyn Memory) {
        let value = self.read_byte(memory, location);
        self.registers.write_byte_masked(ByteRegister::F, u8::compose(&[(!value.get_bit(bit_number), 7), (false, 6), (true, 5)]), 0xE0);
    }

    fn set_bit_on_byte(&mut self, params: ByteOperationParams, bit_number: u8, memory: &mut dyn Memory) {
        let value = self.read_byte(memory, params.source);
        self.write_byte(memory, params.destination, value.set_bit(bit_number));
    }

    fn reset_bit_on_byte(&mut self, params: ByteOperationParams, bit_number: u8, memory: &mut dyn Memory) {
        let value = self.read_byte(memory, params.source);
        self.write_byte(memory, params.destination, value.reset_bit(bit_number));
    }

    fn rotate_byte_left(&mut self, params: ByteRotationParams, memory: &mut dyn Memory) {
        let value = self.read_byte(memory, params.source);
        let result = value.rotate_left(1);
        let zero = !params.unset_zero && result == 0;
        let flag =
            ((zero as u8) << 7) | ((value.get_bit(7) as u8) << 4);
        self.registers.write_byte(ByteRegister::F, flag);
        self.write_byte(memory, params.destination, result);
    }

    fn rotate_byte_left_through_carry(&mut self, params: ByteRotationParams, memory: &mut dyn Memory) {
        let value = self.read_byte(memory, params.source);
        let carry = self.registers.read_byte(ByteRegister::F).get_bit(4);
        let result = (value << 1) | (carry as u8);
        let zero = !params.unset_zero && result == 0;
        let flag =
            ((zero as u8) << 7) | ((value.get_bit(7) as u8) << 4);
        self.registers.write_byte(ByteRegister::F, flag);
        self.write_byte(memory, params.destination, result);
    }

    fn rotate_byte_right(&mut self, params: ByteRotationParams, memory: &mut dyn Memory) {
        let value = self.read_byte(memory, params.source);
        let result = value.rotate_right(1);
        let zero = !params.unset_zero && result == 0;
        let flag =
            ((zero as u8) << 7) | ((value.get_bit(0) as u8) << 4);
        self.registers.write_byte(ByteRegister::F, flag);
        self.write_byte(memory, params.destination, result);
    }

    fn rotate_byte_right_through_carry(&mut self, params: ByteRotationParams, memory: &mut dyn Memory) {
        let value = self.read_byte(memory, params.source);
        let carry = self.registers.read_byte(ByteRegister::F).get_bit(4);
        let result = (value >> 1) | (if carry { 0x80u8 } else { 0x00u8 });
        let zero = !params.unset_zero && result == 0;
        let flag =
            ((zero as u8) << 7) | ((value.get_bit(0) as u8) << 4);
        self.registers.write_byte(ByteRegister::F, flag);
        self.write_byte(memory, params.destination, result);
    }

    fn shift_byte_left(&mut self, params: ByteShiftParams, memory: &mut dyn Memory) {
        let value = self.read_byte(memory, params.source);
        let result = value << 1;
        let zero = result == 0;
        let flag =
            ((zero as u8) << 7) | ((value.get_bit(7) as u8) << 4);
        self.registers.write_byte(ByteRegister::F, flag);
        self.write_byte(memory, params.destination, result);
    }

    fn shift_byte_right(&mut self, params: ByteShiftParams, memory: &mut dyn Memory) {
        let value = self.read_byte(memory, params.source);
        let result = if params.arithmetic { (value >> 1) | (value & 0x80) } else { value >> 1 };
        let zero = result == 0;
        let flag =
            ((zero as u8) << 7) | ((value.get_bit(0) as u8) << 4);
        self.registers.write_byte(ByteRegister::F, flag);
        self.write_byte(memory, params.destination, result);
    }

    fn swap_byte(&mut self, params: ByteOperationParams, memory: &mut dyn Memory) {
        let value = self.read_byte(memory, params.source);
        let result = value.rotate_left(4);
        let flag = if result == 0 { 0x80u8 } else { 0x00u8 };
        self.registers.write_byte(ByteRegister::F, flag);
        self.write_byte(memory, params.destination, result);
    }

    fn increment_word(&mut self, location: WordLocation) {
        let word = self.read_word(location);
        self.write_word(location, word.wrapping_add(1));
    }

    fn decrement_word(&mut self, location: WordLocation) {
        let word = self.read_word(location);
        self.write_word(location, word.wrapping_sub(1));
    }

    fn decimal_adjust_reg_a(&mut self, memory: &mut dyn Memory) {
        let a = self.registers.read_byte(ByteRegister::A);
        let f = self.registers.read_byte(ByteRegister::F);
        let n = f.get_bit(6);
        let carry = f.get_bit(4);
        let half_carry = f.get_bit(5);
        if n {
            let lower = if half_carry { 6u8 } else { 0u8 };
            let upper = if carry { 0x60u8 } else { 0u8 };
            self.subtract_bytes(ByteArithmeticParams {
                first: ByteLocation::Value(a),
                second: ByteLocation::Value(upper | lower),
                destination: ByteLocation::Register(ByteRegister::A),
                use_carry: false,
                flag_mask: 0xB0,
            }, memory);
        } else {
            let lower = if half_carry || ((a & 0x0F) >= 0x0A) { 6u8 } else { 0u8 };
            let upper = if carry || (a > 0x99) { 0x60u8 } else { 0u8 };
            self.add_bytes(ByteArithmeticParams {
                first: ByteLocation::Value(a),
                second: ByteLocation::Value(upper | lower),
                destination: ByteLocation::Register(ByteRegister::A),
                use_carry: false,
                flag_mask: 0xB0,
            }, memory);
        };
        if carry {
            self.registers.write_byte_masked(ByteRegister::F, 0x10, 0x30);
        } else {
            self.registers.write_byte_masked(ByteRegister::F, 0x00, 0x20);
        }
    }

    fn flip_carry_flag(&mut self) {
        self.registers.write_byte_masked(ByteRegister::F, (self.registers.read_byte(ByteRegister::F) ^ 0x10) & 0x90, 0x70);
    }

    fn set_carry_flag(&mut self) {
        self.registers.write_byte_masked(ByteRegister::F, 0x10, 0x70);
    }

    fn halt(&mut self) {
        //TODO: Implement halt
    }

    fn stop(&mut self) {
        self.stopped = true;
    }
}

#[cfg(test)]
pub mod test {
    use assert_hex::assert_eq_hex;
    use test_case::test_case;

    use crate::internal::memory::memory::test::MockMemory;

    use super::*;

    fn perform_ticks(cpu: &mut dyn CPU, memory: &mut dyn Memory, number_of_ticks: u32) {
        for _ in 0..number_of_ticks {
            cpu.tick(memory);
        }
    }

    #[test]
    fn reg_to_reg_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        memory.write(0x0000, 0x45);
        cpu.registers.write_byte(ByteRegister::LowerHL, 0xAB);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::B), 0xAB);
    }

    #[test]
    fn immediate_to_reg_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        memory.write(0x0000, 0x06);
        memory.write(0x0001, 0xAB);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::B), 0xAB);
    }

    #[test]
    fn indirect_to_reg_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        memory.write(0x0000, 0x6E);
        memory.write(0xABCD, 0xEF);
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::LowerHL), 0xEF);
    }

    #[test]
    fn reg_to_indirect_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        cpu.registers.write_byte(ByteRegister::A, 0xEF);
        memory.write(0x0000, 0x77);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(memory.read(0xABCD), 0xEF);
    }

    #[test]
    fn immediate_to_indirect_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x36);
        memory.write(0x0001, 0xEF);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(memory.read(0xABCD), 0xEF);
    }

    #[test]
    fn indirect_bc_to_reg_a_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::BC, 0xABCD);
        memory.write(0x0000, 0x0A);
        memory.write(0xABCD, 0x5A);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), 0x5A);
    }

    #[test]
    fn indirect_de_to_reg_a_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::DE, 0xABCD);
        memory.write(0x0000, 0x1A);
        memory.write(0xABCD, 0x5A);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), 0x5A);
    }

    #[test]
    fn indirect_c_with_offset_to_reg_a_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::C, 0xCD);
        memory.write(0x0000, 0xF2);
        memory.write(0xFFCD, 0x5A);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), 0x5A);
    }

    #[test]
    fn reg_a_to_indirect_c_with_offset_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x5A);
        cpu.registers.write_byte(ByteRegister::C, 0xCD);
        memory.write(0x0000, 0xE2);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(memory.read(0xFFCD), 0x5A);
    }

    #[test]
    fn immediate_indirect_with_offset_to_reg_a_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        memory.write(0x0000, 0xF0);
        memory.write(0x0001, 0xCD);
        memory.write(0xFFCD, 0x5A);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), 0x5A);
    }

    #[test]
    fn reg_a_to_immediate_indirect_with_offset_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x5A);
        memory.write(0x0000, 0xE0);
        memory.write(0x0001, 0xCD);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(memory.read(0xFFCD), 0x5A);
    }

    #[test]
    fn immediate_indirect_to_reg_a_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        memory.write(0x0000, 0xFA);
        memory.write(0x0001, 0xCD);
        memory.write(0x0002, 0xAB);
        memory.write(0xABCD, 0x5A);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), 0x5A);
    }

    #[test]
    fn reg_a_to_immediate_indirect_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x5A);
        memory.write(0x0000, 0xEA);
        memory.write(0x0001, 0xCD);
        memory.write(0x0002, 0xAB);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(0xABCD), 0x5A);
    }


    #[test]
    fn indirect_hl_to_reg_a_ld_and_increment() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x2A);
        memory.write(0xABCD, 0x5A);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(memory.read(0xABCD), 0x5A);
        assert_eq!(cpu.registers.read_word(WordRegister::HL), 0xABCE);
    }

    #[test]
    fn indirect_hl_to_reg_a_ld_and_decrement() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x3A);
        memory.write(0xABCD, 0x5A);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(memory.read(0xABCD), 0x5A);
        assert_eq!(cpu.registers.read_word(WordRegister::HL), 0xABCC);
    }

    #[test]
    fn reg_a_to_indirect_bc_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x5A);
        cpu.registers.write_word(WordRegister::BC, 0xABCD);
        memory.write(0x0000, 0x02);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(memory.read(0xABCD), 0x5A);
    }

    #[test]
    fn reg_a_to_indirect_de_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x5A);
        cpu.registers.write_word(WordRegister::DE, 0xABCD);
        memory.write(0x0000, 0x12);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(memory.read(0xABCD), 0x5A);
    }

    #[test]
    fn reg_a_to_indirect_hl_ld_and_increment() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x5A);
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x22);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(memory.read(0xABCD), 0x5A);
        assert_eq!(cpu.registers.read_word(WordRegister::HL), 0xABCE);
    }

    #[test]
    fn reg_a_to_indirect_hl_ld_and_decrement() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x5A);
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x32);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(memory.read(0xABCD), 0x5A);
        assert_eq!(cpu.registers.read_word(WordRegister::HL), 0xABCC);
    }


    #[test]
    fn immediate_to_reg_pair_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x5A);
        memory.write(0x0000, 0x21);
        memory.write(0x0001, 0x5A);
        memory.write(0x0002, 0x7B);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(cpu.registers.read_word(WordRegister::HL), 0x7B5A);
    }

    #[test]
    fn reg_hl_to_reg_sp_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xF9);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_word(WordRegister::SP), 0xABCD);
    }

    #[test]
    fn push_reg_pair_to_stack() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, 0xFFFE);
        cpu.registers.write_word(WordRegister::DE, 0xABCD);
        memory.write(0x0000, 0xD5);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(0xFFFD), 0xAB);
        assert_eq!(memory.read(0xFFFC), 0xCD);
        assert_eq!(cpu.registers.read_word(WordRegister::SP), 0xFFFC);
    }

    #[test]
    fn pop_stack_to_reg_pair() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, 0xFFFC);
        memory.write(0x0000, 0xD1);
        memory.write(0xFFFC, 0xCD);
        memory.write(0xFFFD, 0xAB);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(cpu.registers.read_word(WordRegister::DE), 0xABCD);
        assert_eq!(cpu.registers.read_word(WordRegister::SP), 0xFFFE);
    }

    #[test]
    fn reg_sp_plus_signed_immediate_to_hl_ld_writes_correct_result() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        // Check if carry flag is set correctly
        cpu.registers.write_word(WordRegister::SP, 0x0005);
        memory.write(0x0000, 0xF8);
        memory.write(0x0001, 0xFD);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(cpu.registers.read_word(WordRegister::HL), 0x0002);
    }

    #[test_case(0x0FF8, 0x07, 0x00; "no flags")]
    #[test_case(0x0FF8, 0x08, 0x20; "only half carry")]
    #[test_case(0xFFF8, 0x08, 0x30; "both carry flags")]
    fn reg_sp_plus_signed_immediate_to_hl_ld_writes_correct_flags(sp: u16, e: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, sp);
        memory.write(0x0000, 0xF8);
        memory.write(0x0001, e);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test]
    fn reg_sp_to_immediate_indirect_ld() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, 0x7B5A);
        memory.write(0x0000, 0x08);
        memory.write(0x0001, 0xCD);
        memory.write(0x0002, 0xAB);
        perform_ticks(&mut cpu, &mut memory, 5);
        assert_eq!(memory.read(0xABCD), 0x5A);
        assert_eq!(memory.read(0xABCE), 0x7B);
    }

    #[test_case(0xFC, 0x04, 0x00, 0xB0; "zero flag set correctly")]
    #[test_case(0xF0, 0xF0, 0xE0, 0x10; "carry set correctly")]
    #[test_case(0x08, 0x08, 0x10, 0x20; "half carry set correctly")]
    fn add_reg_to_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0x82);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0x04, 0x00, 0xB0; "zero flag set correctly")]
    #[test_case(0xF0, 0xF0, 0xE0, 0x10; "carry set correctly")]
    #[test_case(0x08, 0x08, 0x10, 0x20; "half carry set correctly")]
    fn add_immediate_to_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        memory.write(0x0000, 0xC6);
        memory.write(0x0001, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0x04, 0x00, 0xB0; "zero flag set correctly")]
    #[test_case(0xF0, 0xF0, 0xE0, 0x10; "carry set correctly")]
    #[test_case(0x08, 0x08, 0x10, 0x20; "half carry set correctly")]
    fn add_indirect_hl_to_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x86);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0x03, 0x00, 0xB0; "zero flag set correctly")]
    #[test_case(0xF0, 0xEF, 0xE0, 0x30; "carry set correctly")]
    #[test_case(0x08, 0x07, 0x10, 0x20; "half carry set correctly")]
    fn add_reg_with_carry_to_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, 0x10);
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0x8A);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0x03, 0x00, 0xB0; "zero flag set correctly")]
    #[test_case(0xF0, 0xEF, 0xE0, 0x30; "carry set correctly")]
    #[test_case(0x08, 0x07, 0x10, 0x20; "half carry set correctly")]
    fn add_immediate_with_carry_to_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::F, 0x10);

        memory.write(0x0000, 0xCE);
        memory.write(0x0001, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0x03, 0x00, 0xB0; "zero flag set correctly")]
    #[test_case(0xF0, 0x10, 0x01, 0x10; "carry set correctly")]
    #[test_case(0x08, 0x07, 0x10, 0x20; "half carry set correctly")]
    fn add_indirect_hl_with_carry_to_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::F, 0x10);

        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x8E);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0xFC, 0x00, 0xC0; "zero flag set correctly")]
    #[test_case(0x1F, 0x3F, 0xE0, 0x50; "carry set correctly")]
    #[test_case(0xF1, 0xE3, 0x0E, 0x60; "half carry set correctly")]
    fn subtract_reg_from_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0x92);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0xFC, 0x00, 0xC0; "zero flag set correctly")]
    #[test_case(0x1F, 0x3F, 0xE0, 0x50; "carry set correctly")]
    #[test_case(0xF1, 0xE3, 0x0E, 0x60; "half carry set correctly")]
    fn subtract_immediate_from_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        memory.write(0x0000, 0xD6);
        memory.write(0x0001, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0xFC, 0x00, 0xC0; "zero flag set correctly")]
    #[test_case(0x1F, 0x3F, 0xE0, 0x50; "carry set correctly")]
    #[test_case(0xF1, 0xE3, 0x0E, 0x60; "half carry set correctly")]
    fn subtract_indirect_hl_from_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x96);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0xFB, 0x00, 0xC0; "zero flag set correctly")]
    #[test_case(0x1F, 0x3E, 0xE0, 0x50; "carry set correctly")]
    #[test_case(0xF1, 0xE2, 0x0E, 0x60; "half carry set correctly")]
    fn subtract_reg_with_carry_from_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, 0x10);
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0x9A);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0xFB, 0x00, 0xC0; "zero flag set correctly")]
    #[test_case(0x1F, 0x3E, 0xE0, 0x50; "carry set correctly")]
    #[test_case(0xF1, 0xE2, 0x0E, 0x60; "half carry set correctly")]
    fn subtract_immediate_with_carry_from_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::F, 0x10);

        memory.write(0x0000, 0xDE);
        memory.write(0x0001, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0xFB, 0x00, 0xC0; "zero flag set correctly")]
    #[test_case(0x1F, 0x3E, 0xE0, 0x50; "carry set correctly")]
    #[test_case(0xF1, 0xE2, 0x0E, 0x60; "half carry set correctly")]
    fn subtract_indirect_hl_with_carry_from_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::F, 0x10);

        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x9E);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x5A, 0xA5, 0x00, 0xA0; "zero flag set correctly")]
    #[test_case(0xAC, 0xCA, 0x88, 0x20; "half carry set correctly")]
    fn and_reg_with_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0xA2);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x5A, 0xA5, 0x00, 0xA0; "zero flag set correctly")]
    #[test_case(0xAC, 0xCA, 0x88, 0x20; "half carry set correctly")]
    fn and_immediate_with_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::F, 0x10);

        memory.write(0x0000, 0xE6);
        memory.write(0x0001, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x5A, 0xA5, 0x00, 0xA0; "zero flag set correctly")]
    #[test_case(0xAC, 0xCA, 0x88, 0x20; "half carry set correctly")]
    fn and_indirect_hl_with_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::F, 0x10);

        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xA6);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x00, 0x00, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0xAC, 0xCA, 0xEE, 0x00; "calculates OR correctly")]
    fn or_reg_with_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0xB2);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x00, 0x00, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0xAC, 0xCA, 0xEE, 0x00; "calculates OR correctly")]
    fn or_immediate_with_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::F, 0x10);

        memory.write(0x0000, 0xF6);
        memory.write(0x0001, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x00, 0x00, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0xAC, 0xCA, 0xEE, 0x00; "calculates OR correctly")]
    fn or_indirect_hl_with_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::F, 0x10);

        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xB6);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xAE, 0xAE, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0xAC, 0xCA, 0x66, 0x00; "calculates XOR correctly")]
    fn xor_reg_with_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0xAA);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xAE, 0xAE, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0xAC, 0xCA, 0x66, 0x00; "calculates XOR correctly")]
    fn xor_immediate_with_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::F, 0x10);

        memory.write(0x0000, 0xEE);
        memory.write(0x0001, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xAE, 0xAE, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0xAC, 0xCA, 0x66, 0x00; "calculates XOR correctly")]
    fn xor_indirect_hl_with_reg_a_and_write_to_reg_a(a: u8, value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::F, 0x10);

        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xAE);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0xFC, 0xC0; "zero flag set correctly")]
    #[test_case(0x1F, 0x3F, 0x50; "carry set correctly")]
    #[test_case(0xF1, 0xE3, 0x60; "half carry set correctly")]
    fn compare_reg_with_reg_a(a: u8, value: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0xBA);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0xFC, 0xC0; "zero flag set correctly")]
    #[test_case(0x1F, 0x3F, 0x50; "carry set correctly")]
    #[test_case(0xF1, 0xE3, 0x60; "half carry set correctly")]
    fn compare_immediate_with_reg_a(a: u8, value: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        memory.write(0x0000, 0xFE);
        memory.write(0x0001, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFC, 0xFC, 0xC0; "zero flag set correctly")]
    #[test_case(0x1F, 0x3F, 0x50; "carry set correctly")]
    #[test_case(0xF1, 0xE3, 0x60; "half carry set correctly")]
    fn compare_indirect_hl_with_reg_a(a: u8, value: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, a);
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xBE);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFF, 0x00, 0x00, 0xA0; "zero flag set correctly and carry is not affected")]
    #[test_case(0x0F, 0x10, 0x10, 0x30; "half carry set correctly")]
    fn increment_reg(value: u8, result: u8, f_old: u8, f_new: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, f_old);
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0x14);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f_new);
    }

    #[test_case(0xFF, 0x00, 0x00, 0xA0; "zero flag set correctly and carry is not affected")]
    #[test_case(0x0F, 0x10, 0x10, 0x30; "half carry set correctly")]
    fn increment_indirect_hl(value: u8, result: u8, f_old: u8, f_new: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, f_old);
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x34);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(memory.read(0xABCD), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f_new);
    }

    #[test_case(0x01, 0x00, 0x10, 0xD0; "zero flag set correctly and carry not affected")]
    #[test_case(0x10, 0x0F, 0x00, 0x60; "half carry set correctly")]
    fn decrement_reg(value: u8, result: u8, f_old: u8, f_new: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, f_old);
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0x15);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f_new);
    }

    #[test_case(0x01, 0x00, 0x10, 0xD0; "zero flag set correctly and carry not affected")]
    #[test_case(0x10, 0x0F, 0x00, 0x60; "half carry set correctly")]
    fn decrement_indirect_hl(value: u8, result: u8, f_old: u8, f_new: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, f_old);
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0x35);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(memory.read(0xABCD), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f_new);
    }

    #[test_case(0xF01E, 0xF028, 0xE046, 0x80, 0x90; "carry set correctly and zero flag not affected")]
    #[test_case(0x1E1E, 0x2828, 0x4646, 0x80, 0xA0; "half carry set correctly")]
    fn add_reg_pair_to_reg_hl(hl: u16, value: u16, result: u16, f_old: u8, f_new: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, f_old);
        cpu.registers.write_word(WordRegister::HL, hl);
        cpu.registers.write_word(WordRegister::DE, value);
        memory.write(0x0000, 0x19);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_word(WordRegister::HL), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f_new);
    }

    #[test_case(0xFFDA, 0x26, 0x0000, 0x30; "carry set correctly and zero flag set to zero")]
    #[test_case(0x0FDA, 0x26, 0x1000, 0x20; "half carry set correctly")]
    fn add_immediate_to_reg_sp(sp: u16, value: u8, result: u16, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, sp);
        memory.write(0x0000, 0xE8);
        memory.write(0x0001, value);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq_hex!(cpu.registers.read_word(WordRegister::SP), result);
        assert_eq_hex!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0xFFFF, 0x0000; "performs wrapping correctly")]
    #[test_case(0x0FDA, 0x0FDB; "increments correctly")]
    fn increment_reg_pair(sp: u16, result: u16) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, 0xF0);
        cpu.registers.write_word(WordRegister::SP, sp);
        memory.write(0x0000, 0x33);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_word(WordRegister::SP), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0xF0);
    }

    #[test_case(0x0000, 0xFFFF; "performs wrapping correctly")]
    #[test_case(0x0FDA, 0x0FD9; "decrements correctly")]
    fn decrement_reg_pair(sp: u16, result: u16) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, 0xF0);
        cpu.registers.write_word(WordRegister::SP, sp);
        memory.write(0x0000, 0x3B);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_word(WordRegister::SP), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0xF0);
    }

    #[test]
    fn rotate_reg_a_left() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0xCA);
        memory.write(0x0000, 0x07);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), 0x95);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0x10);
    }

    #[test_case(0x00, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0xCA, 0x95, 0x10; "rotates left correctly and sets carry")]
    fn rotate_reg_left(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x02);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x00, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0xCA, 0x95, 0x10; "rotates left correctly and sets carry")]
    fn rotate_indirect_hl_left(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x06);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(0xABCD), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test]
    fn rotate_reg_a_right() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x53);
        memory.write(0x0000, 0x0F);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), 0xA9);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0x10);
    }

    #[test_case(0x00, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0x53, 0xA9, 0x10; "rotates right correctly and sets carry")]
    fn rotate_reg_right(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x0A);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }


    #[test_case(0x00, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0x53, 0xA9, 0x10; "rotates right correctly and sets carry")]
    fn rotate_indirect_hl_right(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x0E);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(0xABCD), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test]
    fn rotate_reg_a_left_through_carry() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x4A);
        cpu.registers.write_byte(ByteRegister::F, 0x10);
        memory.write(0x0000, 0x17);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), 0x95);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0x00);
    }

    #[test_case(0x80, 0x00, 0x00, 0x90; "zero flag set correctly")]
    #[test_case(0x4A, 0x95, 0x10, 0x00; "rotates left correctly and sets carry")]
    fn rotate_reg_left_through_carry(value: u8, result: u8, old_f: u8, new_f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::D, value);
        cpu.registers.write_byte(ByteRegister::F, old_f);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x12);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), new_f);
    }

    #[test_case(0x80, 0x00, 0x00, 0x90; "zero flag set correctly")]
    #[test_case(0x4A, 0x95, 0x10, 0x00; "rotates left correctly and sets carry")]
    fn rotate_indirect_hl_left_through_carry(value: u8, result: u8, old_f: u8, new_f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        cpu.registers.write_byte(ByteRegister::F, old_f);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x16);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(0xABCD), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), new_f);
    }

    #[test]
    fn rotate_reg_a_right_through_carry() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0x52);
        cpu.registers.write_byte(ByteRegister::F, 0x10);
        memory.write(0x0000, 0x1F);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::A), 0xA9);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0x00);
    }

    #[test_case(0x01, 0x00, 0x00, 0x90; "zero flag set correctly")]
    #[test_case(0x52, 0xA9, 0x10, 0x00; "rotates right correctly and sets carry")]
    fn rotate_reg_right_through_carry(value: u8, result: u8, old_f: u8, new_f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::D, value);
        cpu.registers.write_byte(ByteRegister::F, old_f);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x1A);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), new_f);
    }

    #[test_case(0x01, 0x00, 0x00, 0x90; "zero flag set correctly")]
    #[test_case(0x52, 0xA9, 0x10, 0x00; "rotates right correctly and sets carry")]
    fn rotate_indirect_hl_right_through_carry(value: u8, result: u8, old_f: u8, new_f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        cpu.registers.write_byte(ByteRegister::F, old_f);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x1E);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(0xABCD), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), new_f);
    }

    #[test_case(0x80, 0x00, 0x90; "zero flag set correctly")]
    #[test_case(0xCA, 0x94, 0x10; "shifts left correctly and sets carry")]
    fn shift_reg_left(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x22);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x80, 0x00, 0x90; "zero flag set correctly")]
    #[test_case(0xCA, 0x94, 0x10; "shifts left correctly and sets carry")]
    fn shift_indirect_hl_left(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x26);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(0xABCD), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x01, 0x00, 0x90; "zero flag set correctly")]
    #[test_case(0x53, 0x29, 0x10; "shifts right correctly and sets carry")]
    fn shift_reg_right(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x3A);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x01, 0x00, 0x90; "zero flag set correctly")]
    #[test_case(0x53, 0x29, 0x10; "shifts right correctly and sets carry")]
    fn shift_indirect_hl_right(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x3E);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(0xABCD), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x01, 0x00, 0x90; "zero flag set correctly")]
    #[test_case(0xA2, 0xD1, 0x00; "shifts right correctly")]
    fn shift_reg_right_arithmetic(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x2A);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x01, 0x00, 0x90; "zero flag set correctly")]
    #[test_case(0xA2, 0xD1, 0x00; "shifts right correctly")]
    fn shift_indirect_hl_right_arithmetic(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x2E);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(0xABCD), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x00, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0xA6, 0x6A, 0x00; "swaps correctly")]
    fn swap_reg(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::D, value);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x32);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test_case(0x00, 0x00, 0x80; "zero flag set correctly")]
    #[test_case(0xA6, 0x6A, 0x00; "swaps correctly")]
    fn swap_indirect_hl(value: u8, result: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xCB);
        memory.write(0x0001, 0x36);
        memory.write(0xABCD, value);
        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(0xABCD), result);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), f);
    }

    #[test]
    fn get_reg_bit() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::D, 0xA5);
        let bits: Vec<(bool, u8)> = (0u8..8u8).map(|bit| {
            memory.write((2 * bit) as u16, 0xCB);
            memory.write((2 * bit + 1) as u16, 0x42 | (bit << 3));
            perform_ticks(&mut cpu, &mut memory, 2);
            (!cpu.registers.read_byte(ByteRegister::F).get_bit(7), bit)
        }).collect();
        let result = u8::compose(&bits);
        assert_eq!(result, 0xA5);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0x20);
    }

    #[test]
    fn get_indirect_hl_bit() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0xABCD, 0xA5);
        let bits: Vec<(bool, u8)> = (0u8..8u8).map(|bit| {
            memory.write((2 * bit) as u16, 0xCB);
            memory.write((2 * bit + 1) as u16, 0x46 | (bit << 3));
            perform_ticks(&mut cpu, &mut memory, 3);
            (!cpu.registers.read_byte(ByteRegister::F).get_bit(7), bit)
        }).collect();
        let result = u8::compose(&bits);
        assert_eq_hex!(result, 0xA5);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0x20);
    }

    #[test]
    fn set_reg_bit() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, 0xB0);
        [0, 2, 5, 7].iter().enumerate().for_each(|(index, bit)| {
            memory.write((2 * index) as u16, 0xCB);
            memory.write((2 * index + 1) as u16, 0xC2 | (bit << 3));
            perform_ticks(&mut cpu, &mut memory, 2);
        });
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), 0xA5);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0xB0);
    }

    #[test]
    fn set_indirect_hl_bit() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        cpu.registers.write_byte(ByteRegister::F, 0xB0);
        [0, 2, 5, 7].iter().enumerate().for_each(|(index, bit)| {
            memory.write((2 * index) as u16, 0xCB);
            memory.write((2 * index + 1) as u16, 0xC6 | (bit << 3));
            perform_ticks(&mut cpu, &mut memory, 4);
        });
        assert_eq!(memory.read(0xABCD), 0xA5);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0xB0);
    }

    #[test]
    fn reset_reg_bit() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::D, 0xFF);
        cpu.registers.write_byte(ByteRegister::F, 0xB0);
        [1, 3, 4, 6].iter().enumerate().for_each(|(index, bit)| {
            memory.write((2 * index) as u16, 0xCB);
            memory.write((2 * index + 1) as u16, 0x82 | (bit << 3));
            perform_ticks(&mut cpu, &mut memory, 2);
        });
        assert_eq!(cpu.registers.read_byte(ByteRegister::D), 0xA5);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0xB0);
    }

    #[test]
    fn reset_indirect_hl_bit() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0xABCD, 0xFF);
        cpu.registers.write_byte(ByteRegister::F, 0xB0);
        [1, 3, 4, 6].iter().enumerate().for_each(|(index, bit)| {
            memory.write((2 * index) as u16, 0xCB);
            memory.write((2 * index + 1) as u16, 0x86 | (bit << 3));
            perform_ticks(&mut cpu, &mut memory, 4);
        });
        assert_eq_hex!(memory.read(0xABCD), 0xA5);
        assert_eq_hex!(cpu.registers.read_byte(ByteRegister::F), 0xB0);
    }

    #[test]
    fn jump() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        memory.write(0x0000, 0xC3);
        memory.write(0x0001, 0xCD);
        memory.write(0x0002, 0xAB);
        perform_ticks(&mut cpu, &mut memory, 4);

        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0xABCD);
    }

    #[test_case(0x00, 0x70; "jumps when zero flag not set")]
    #[test_case(0x01, 0x80; "jumps when zero flag set")]
    #[test_case(0x02, 0xE0; "jumps when carry not set")]
    #[test_case(0x03, 0x10; "jumps when carry set")]
    fn jump_conditional(condition: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, !f);
        memory.write(0x0000, 0xC2 | (condition << 3));
        memory.write(0x0001, 0xCD);
        memory.write(0x0002, 0xAB);
        memory.write(0x0003, 0xC2 | (condition << 3));
        memory.write(0x0004, 0xCD);
        memory.write(0x0005, 0xAB);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0x0003);

        cpu.registers.write_byte(ByteRegister::F, f);
        perform_ticks(&mut cpu, &mut memory, 4);

        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0xABCD);
    }

    #[test]
    fn jump_relative() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        memory.write(0x0000, 0x18);
        memory.write(0x0001, 0x08);
        memory.write(0x000A, 0x18);
        memory.write(0x000B, 0xFC);
        perform_ticks(&mut cpu, &mut memory, 6);

        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0x0008);
    }

    #[test_case(0x00, 0x70; "jumps when zero flag not set")]
    #[test_case(0x01, 0x80; "jumps when zero flag set")]
    #[test_case(0x02, 0xE0; "jumps when carry not set")]
    #[test_case(0x03, 0x10; "jumps when carry set")]
    fn jump_conditional_relative(condition: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, !f);
        memory.write(0x0000, 0x20 | (condition << 3));
        memory.write(0x0001, 0x08);
        memory.write(0x0002, 0x20 | (condition << 3));
        memory.write(0x0003, 0x08);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0x0002);

        cpu.registers.write_byte(ByteRegister::F, f);
        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0x000C);
    }

    #[test]
    fn jump_indirect_hl() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::HL, 0xABCD);
        memory.write(0x0000, 0xE9);
        cpu.tick(&mut memory);

        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0xABCD);
    }

    #[test]
    fn call() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, 0xFFFE);
        cpu.registers.write_word(WordRegister::PC, 0x1234);
        memory.write(0x1234, 0xCD);
        memory.write(0x1235, 0xCD);
        memory.write(0x1236, 0xAB);
        perform_ticks(&mut cpu, &mut memory, 6);

        assert_eq!(cpu.registers.read_word(WordRegister::SP), 0xFFFC);
        assert_eq!(memory.read(0xFFFD), 0x12);
        assert_eq!(memory.read(0xFFFC), 0x37);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0xABCD);
    }

    #[test_case(0x00, 0x70; "calls when zero flag not set")]
    #[test_case(0x01, 0x80; "calls when zero flag set")]
    #[test_case(0x02, 0xE0; "calls when carry not set")]
    #[test_case(0x03, 0x10; "calls when carry set")]
    fn call_conditional(condition: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, 0xFFFE);
        cpu.registers.write_word(WordRegister::PC, 0x1234);
        cpu.registers.write_byte(ByteRegister::F, !f);
        memory.write(0x1234, 0xC4 | (condition << 3));
        memory.write(0x1235, 0xCD);
        memory.write(0x1236, 0xAB);
        memory.write(0x1237, 0xC4 | (condition << 3));
        memory.write(0x1238, 0xCD);
        memory.write(0x1239, 0xAB);

        perform_ticks(&mut cpu, &mut memory, 3);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0x1237);

        cpu.registers.write_byte(ByteRegister::F, f);
        perform_ticks(&mut cpu, &mut memory, 6);

        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0xABCD);
        assert_eq!(cpu.registers.read_word(WordRegister::SP), 0xFFFC);
        assert_eq!(memory.read(0xFFFD), 0x12);
        assert_eq!(memory.read(0xFFFC), 0x3A);
    }

    #[test]
    fn return_from_call() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, 0xFFFE);
        cpu.registers.write_word(WordRegister::PC, 0x1234);
        memory.write(0x1234, 0xCD);
        memory.write(0x1235, 0xCD);
        memory.write(0x1236, 0xAB);
        memory.write(0xABCD, 0xC9);
        perform_ticks(&mut cpu, &mut memory, 6);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0xABCD);

        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0x1237);
        assert_eq!(cpu.registers.read_word(WordRegister::SP), 0xFFFE);
    }

    #[test]
    fn return_from_interrupt() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, 0xFFFE);
        cpu.registers.write_word(WordRegister::PC, 0x1234);
        memory.write(0x1234, 0xCD);
        memory.write(0x1235, 0xCD);
        memory.write(0x1236, 0xAB);
        memory.write(0xABCD, 0xF3);
        memory.write(0xABCE, 0xD9);
        perform_ticks(&mut cpu, &mut memory, 6);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0xABCD);

        cpu.tick(&mut memory);
        assert_eq!(memory.read(MemoryAddress::IME), 0x00);

        perform_ticks(&mut cpu, &mut memory, 4);
        assert_eq!(memory.read(MemoryAddress::IME), 0x01);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0x1237);
        assert_eq!(cpu.registers.read_word(WordRegister::SP), 0xFFFE);
    }

    #[test_case(0x00, 0x70; "returns when zero flag not set")]
    #[test_case(0x01, 0x80; "returns when zero flag set")]
    #[test_case(0x02, 0xE0; "returns when carry not set")]
    #[test_case(0x03, 0x10; "returns when carry set")]
    fn return_conditionally(condition: u8, f: u8) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, 0xFFFE);
        cpu.registers.write_word(WordRegister::PC, 0x1234);
        memory.write(0x1234, 0xCD);
        memory.write(0x1235, 0xCD);
        memory.write(0x1236, 0xAB);
        memory.write(0xABCD, 0xC0 | (condition << 3));
        memory.write(0xABCE, 0xC0 | (condition << 3));
        perform_ticks(&mut cpu, &mut memory, 6);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0xABCD);

        cpu.registers.write_byte(ByteRegister::F, !f);
        perform_ticks(&mut cpu, &mut memory, 2);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0xABCE);

        cpu.registers.write_byte(ByteRegister::F, f);
        perform_ticks(&mut cpu, &mut memory, 5);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), 0x1237);
        assert_eq!(cpu.registers.read_word(WordRegister::SP), 0xFFFE);
    }

    #[test_case(0, 0x0000; "restart to 0x0000")]
    #[test_case(1, 0x0008; "restart to 0x0008")]
    #[test_case(2, 0x0010; "restart to 0x0010")]
    #[test_case(3, 0x0018; "restart to 0x0018")]
    #[test_case(4, 0x0020; "restart to 0x0020")]
    #[test_case(5, 0x0028; "restart to 0x0028")]
    #[test_case(6, 0x0030; "restart to 0x0030")]
    #[test_case(7, 0x0038; "restart to 0x0038")]
    fn restart(operand: u8, address: u16) {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_word(WordRegister::SP, 0xFFFE);
        cpu.registers.write_word(WordRegister::PC, 0x1234);
        memory.write(0x1234, 0xC7 | (operand << 3));
        perform_ticks(&mut cpu, &mut memory, 4);

        assert_eq!(cpu.registers.read_word(WordRegister::SP), 0xFFFC);
        assert_eq!(memory.read(0xFFFD), 0x12);
        assert_eq!(memory.read(0xFFFC), 0x35);
        assert_eq!(cpu.registers.read_word(WordRegister::PC), address);
    }

    #[test]
    fn decimal_adjust_reg_a() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        let mut instruction_index = 0u16;
        (0u8..99u8).for_each(|x| {
            (0u8..99u8).for_each(|y| {
                let sum = x + y;
                let difference = 100 + x - y;
                let a = (x % 10) | ((x / 10) << 4);
                let d = (y % 10) | ((y / 10) << 4);
                let f = u8::compose(&[(sum % 100 == 0, 7), (sum >= 100, 4)]);
                if a == 0x1 && d == 0x9 {
                    println!("Invalid result. A: {:#x}, D: {:#x}, F: {:#x}", a, d, f);
                }

                cpu.registers.write_byte(ByteRegister::A, a);
                cpu.registers.write_byte(ByteRegister::D, d);
                memory.write(instruction_index, 0x82);
                instruction_index += 1;
                cpu.tick(&mut memory);
                memory.write(instruction_index, 0x27);
                instruction_index += 1;
                cpu.tick(&mut memory);
                let result_bcd_sum = cpu.registers.read_byte(ByteRegister::A);
                let result_decimal_sum = ((result_bcd_sum & 0xF0) >> 4) * 10 + (result_bcd_sum & 0x0F);
                assert_eq!(result_decimal_sum, sum % 100);
                assert_eq_hex!(cpu.registers.read_byte(ByteRegister::F) & 0xB0, f);

                cpu.registers.write_byte(ByteRegister::A, a);
                cpu.registers.write_byte(ByteRegister::D, d);
                memory.write(instruction_index, 0x92);
                instruction_index += 1;
                cpu.tick(&mut memory);
                memory.write(instruction_index, 0x27);
                instruction_index += 1;
                cpu.tick(&mut memory);
                let result_bcd_diff = cpu.registers.read_byte(ByteRegister::A);
                let result_decimal_diff = ((result_bcd_diff & 0xF0) >> 4) * 10 + (result_bcd_diff & 0x0F);
                let f = u8::compose(&[(difference % 100 == 0, 7), (difference < 100, 4)]);
                assert_eq_hex!(cpu.registers.read_byte(ByteRegister::F) & 0xB0, f);
                assert_eq!(result_decimal_diff, difference % 100);
            })
        })
    }

    #[test]
    fn ones_complement_reg_a() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::A, 0xA6);
        cpu.registers.write_byte(ByteRegister::F, 0x90);
        memory.write(0x0000, 0x2F);
        cpu.tick(&mut memory);

        assert_eq!(cpu.registers.read_byte(ByteRegister::A), 0x59);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0xF0);
    }

    #[test]
    fn flip_carry() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, 0x80);
        memory.write(0x0000, 0x3F);
        memory.write(0x0001, 0x3F);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0x90);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0x80);
    }

    #[test]
    fn set_carry() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        cpu.registers.write_byte(ByteRegister::F, 0x80);
        memory.write(0x0000, 0x37);
        cpu.tick(&mut memory);
        assert_eq!(cpu.registers.read_byte(ByteRegister::F), 0x90);
    }

    #[test]
    fn disable_enable_interrupts() {
        let mut cpu = CPUImpl::new();
        let mut memory = MockMemory::new();
        memory.write(MemoryAddress::RI, 0xFF); // Return no interrupts
        memory.write(MemoryAddress::IME, 0x01);
        memory.write(0x0000, 0xF3);
        memory.write(0x0001, 0xFB);
        cpu.tick(&mut memory);
        assert_eq!(memory.read(MemoryAddress::IME), 0x00);
        cpu.tick(&mut memory);
        assert_eq!(memory.read(MemoryAddress::IME), 0x01);
    }
}
