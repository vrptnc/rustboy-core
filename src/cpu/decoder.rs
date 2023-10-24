use crate::cpu::instruction::{ByteArithmeticParams, ByteCastingParams, ByteLocation, ByteLogicParams, ByteOperationParams, ByteRotationParams, ByteShiftParams, Instruction, WordArithmeticParams, WordLocation, WordOperationParams};
use crate::cpu::instruction::Instruction::{AddBytes, AddWords, AndBytes, BranchIfCarry, BranchIfNotCarry, BranchIfNotZero, BranchIfZero, CastByteToSignedWord, ClearInterrupt, DecimalAdjust, DecodeCBInstruction, DecrementWord, Defer, DisableInterrupts, EnableInterrupts, EndBranch, FlipCarry, GetBitFromByte, Halt, IncrementWord, MoveByte, MoveWord, Noop, OnesComplementByte, OrBytes, ResetBitOnByte, RotateByteLeft, RotateByteLeftThroughCarry, RotateByteRight, RotateByteRightThroughCarry, SetBitOnByte, SetCarry, ShiftByteLeft, ShiftByteRight, Stop, SubtractBytes, SwapByte, XorBytes};
use crate::cpu::interrupts::Interrupt;
use crate::cpu::opcode::Opcode;
use crate::cpu::register::{ByteRegister, WordRegister};

pub trait InstructionScheduler {
    fn schedule(&mut self, instruction: Instruction);
}

pub struct InstructionDecoder {}

impl InstructionDecoder {
    pub fn decode(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        match opcode.value() {
            0x00 => {}
            0x01 => InstructionDecoder::immediate_to_reg_pair_ld(scheduler, opcode),
            0x02 => InstructionDecoder::reg_a_to_indirect_bc_ld(scheduler),
            0x03 => InstructionDecoder::increment_reg_pair(scheduler, opcode),
            0x04 => InstructionDecoder::increment_reg(scheduler, opcode),
            0x05 => InstructionDecoder::decrement_reg(scheduler, opcode),
            0x06 => InstructionDecoder::immediate_to_reg_ld(scheduler, opcode),
            0x07 => InstructionDecoder::rotate_reg_a_left(scheduler),
            0x08 => InstructionDecoder::reg_sp_to_immediate_indirect_ld(scheduler),
            0x09 => InstructionDecoder::add_reg_pair_to_reg_hl(scheduler, opcode),
            0x0A => InstructionDecoder::indirect_bc_to_reg_a_ld(scheduler),
            0x0B => InstructionDecoder::decrement_reg_pair(scheduler, opcode),
            0x0C => InstructionDecoder::increment_reg(scheduler, opcode),
            0x0D => InstructionDecoder::decrement_reg(scheduler, opcode),
            0x0E => InstructionDecoder::immediate_to_reg_ld(scheduler, opcode),
            0x0F => InstructionDecoder::rotate_reg_a_right(scheduler),
            0x10 => InstructionDecoder::stop(scheduler),
            0x11 => InstructionDecoder::immediate_to_reg_pair_ld(scheduler, opcode),
            0x12 => InstructionDecoder::reg_a_to_indirect_de_ld(scheduler),
            0x13 => InstructionDecoder::increment_reg_pair(scheduler, opcode),
            0x14 => InstructionDecoder::increment_reg(scheduler, opcode),
            0x15 => InstructionDecoder::decrement_reg(scheduler, opcode),
            0x16 => InstructionDecoder::immediate_to_reg_ld(scheduler, opcode),
            0x17 => InstructionDecoder::rotate_reg_a_left_through_carry(scheduler),
            0x18 => InstructionDecoder::jump_relative(scheduler),
            0x19 => InstructionDecoder::add_reg_pair_to_reg_hl(scheduler, opcode),
            0x1A => InstructionDecoder::indirect_de_to_reg_a_ld(scheduler),
            0x1B => InstructionDecoder::decrement_reg_pair(scheduler, opcode),
            0x1C => InstructionDecoder::increment_reg(scheduler, opcode),
            0x1D => InstructionDecoder::decrement_reg(scheduler, opcode),
            0x1E => InstructionDecoder::immediate_to_reg_ld(scheduler, opcode),
            0x1F => InstructionDecoder::rotate_reg_a_right_through_carry(scheduler),
            0x20 => InstructionDecoder::jump_conditional_relative(scheduler, opcode),
            0x21 => InstructionDecoder::immediate_to_reg_pair_ld(scheduler, opcode),
            0x22 => InstructionDecoder::reg_a_to_indirect_hl_ld_and_increment(scheduler),
            0x23 => InstructionDecoder::increment_reg_pair(scheduler, opcode),
            0x24 => InstructionDecoder::increment_reg(scheduler, opcode),
            0x25 => InstructionDecoder::decrement_reg(scheduler, opcode),
            0x26 => InstructionDecoder::immediate_to_reg_ld(scheduler, opcode),
            0x27 => InstructionDecoder::decimal_adjust_reg_a(scheduler),
            0x28 => InstructionDecoder::jump_conditional_relative(scheduler, opcode),
            0x29 => InstructionDecoder::add_reg_pair_to_reg_hl(scheduler, opcode),
            0x2A => InstructionDecoder::indirect_hl_to_reg_a_ld_and_increment(scheduler),
            0x2B => InstructionDecoder::decrement_reg_pair(scheduler, opcode),
            0x2C => InstructionDecoder::increment_reg(scheduler, opcode),
            0x2D => InstructionDecoder::decrement_reg(scheduler, opcode),
            0x2E => InstructionDecoder::immediate_to_reg_ld(scheduler, opcode),
            0x2F => InstructionDecoder::ones_complement_reg_a(scheduler),
            0x30 => InstructionDecoder::jump_conditional_relative(scheduler, opcode),
            0x31 => InstructionDecoder::immediate_to_reg_pair_ld(scheduler, opcode),
            0x32 => InstructionDecoder::reg_a_to_indirect_hl_ld_and_decrement(scheduler),
            0x33 => InstructionDecoder::increment_reg_pair(scheduler, opcode),
            0x34 => InstructionDecoder::increment_indirect_hl(scheduler),
            0x35 => InstructionDecoder::decrement_indirect_hl(scheduler),
            0x36 => InstructionDecoder::immediate_to_indirect_ld(scheduler),
            0x37 => InstructionDecoder::set_carry_flag(scheduler),
            0x38 => InstructionDecoder::jump_conditional_relative(scheduler, opcode),
            0x39 => InstructionDecoder::add_reg_pair_to_reg_hl(scheduler, opcode),
            0x3A => InstructionDecoder::indirect_hl_to_reg_a_ld_and_decrement(scheduler),
            0x3B => InstructionDecoder::decrement_reg_pair(scheduler, opcode),
            0x3C => InstructionDecoder::increment_reg(scheduler, opcode),
            0x3D => InstructionDecoder::decrement_reg(scheduler, opcode),
            0x3E => InstructionDecoder::immediate_to_reg_ld(scheduler, opcode),
            0x3F => InstructionDecoder::flip_carry_flag(scheduler),
            0x40..=0x45 => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x46 => InstructionDecoder::indirect_to_reg_ld(scheduler, opcode),
            0x47..=0x4D => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x4E => InstructionDecoder::indirect_to_reg_ld(scheduler, opcode),
            0x4F => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x50..=0x55 => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x56 => InstructionDecoder::indirect_to_reg_ld(scheduler, opcode),
            0x57..=0x5D => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x5E => InstructionDecoder::indirect_to_reg_ld(scheduler, opcode),
            0x5F => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x60..=0x65 => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x66 => InstructionDecoder::indirect_to_reg_ld(scheduler, opcode),
            0x67..=0x6D => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x6E => InstructionDecoder::indirect_to_reg_ld(scheduler, opcode),
            0x6F => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x70..=0x75 => InstructionDecoder::reg_to_indirect_ld(scheduler, opcode),
            0x76 => InstructionDecoder::halt(scheduler),
            0x77 => InstructionDecoder::reg_to_indirect_ld(scheduler, opcode),
            0x78..=0x7D => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x7E => InstructionDecoder::indirect_to_reg_ld(scheduler, opcode),
            0x7F => InstructionDecoder::reg_to_reg_ld(scheduler, opcode),
            0x80..=0x85 => InstructionDecoder::add_reg_to_reg_a_and_write_to_reg_a(scheduler, opcode, false),
            0x86 => InstructionDecoder::add_indirect_hl_to_reg_a_and_write_to_reg_a(scheduler, false),
            0x87 => InstructionDecoder::add_reg_to_reg_a_and_write_to_reg_a(scheduler, opcode, false),
            0x88..=0x8D => InstructionDecoder::add_reg_to_reg_a_and_write_to_reg_a(scheduler, opcode, true),
            0x8E => InstructionDecoder::add_indirect_hl_to_reg_a_and_write_to_reg_a(scheduler, true),
            0x8F => InstructionDecoder::add_reg_to_reg_a_and_write_to_reg_a(scheduler, opcode, true),
            0x90..=0x95 => InstructionDecoder::subtract_reg_from_reg_a_and_write_to_reg_a(scheduler, opcode, false),
            0x96 => InstructionDecoder::subtract_indirect_hl_from_reg_a_and_write_to_reg_a(scheduler, false),
            0x97 => InstructionDecoder::subtract_reg_from_reg_a_and_write_to_reg_a(scheduler, opcode, false),
            0x98..=0x9D => InstructionDecoder::subtract_reg_from_reg_a_and_write_to_reg_a(scheduler, opcode, true),
            0x9E => InstructionDecoder::subtract_indirect_hl_from_reg_a_and_write_to_reg_a(scheduler, true),
            0x9F => InstructionDecoder::subtract_reg_from_reg_a_and_write_to_reg_a(scheduler, opcode, true),
            0xA0..=0xA5 => InstructionDecoder::and_reg_with_reg_a_and_write_to_reg_a(scheduler, opcode),
            0xA6 => InstructionDecoder::and_indirect_hl_with_reg_a_and_write_to_reg_a(scheduler),
            0xA7 => InstructionDecoder::and_reg_with_reg_a_and_write_to_reg_a(scheduler, opcode),
            0xA8..=0xAD => InstructionDecoder::xor_reg_with_reg_a_and_write_to_reg_a(scheduler, opcode),
            0xAE => InstructionDecoder::xor_indirect_hl_with_reg_a_and_write_to_reg_a(scheduler),
            0xAF => InstructionDecoder::xor_reg_with_reg_a_and_write_to_reg_a(scheduler, opcode),
            0xB0..=0xB5 => InstructionDecoder::or_reg_with_reg_a_and_write_to_reg_a(scheduler, opcode),
            0xB6 => InstructionDecoder::or_indirect_hl_with_reg_a_and_write_to_reg_a(scheduler),
            0xB7 => InstructionDecoder::or_reg_with_reg_a_and_write_to_reg_a(scheduler, opcode),
            0xB8..=0xBD => InstructionDecoder::compare_reg_with_reg_a(scheduler, opcode),
            0xBE => InstructionDecoder::compare_indirect_hl_with_reg_a(scheduler),
            0xBF => InstructionDecoder::compare_reg_with_reg_a(scheduler, opcode),
            0xC0 => InstructionDecoder::return_conditionally(scheduler, opcode),
            0xC1 => InstructionDecoder::pop_stack_to_reg_pair(scheduler, opcode),
            0xC2 => InstructionDecoder::jump_conditional(scheduler, opcode),
            0xC3 => InstructionDecoder::jump(scheduler),
            0xC4 => InstructionDecoder::call_conditional(scheduler, opcode),
            0xC5 => InstructionDecoder::push_reg_pair_to_stack(scheduler, opcode),
            0xC6 => InstructionDecoder::add_immediate_to_reg_a_and_write_to_reg_a(scheduler, false),
            0xC7 => InstructionDecoder::restart(scheduler, opcode),
            0xC8 => InstructionDecoder::return_conditionally(scheduler, opcode),
            0xC9 => InstructionDecoder::return_from_call(scheduler),
            0xCA => InstructionDecoder::jump_conditional(scheduler, opcode),
            0xCB => {
                scheduler.schedule(Defer);
                scheduler.schedule(DecodeCBInstruction);
            }
            0xCC => InstructionDecoder::call_conditional(scheduler, opcode),
            0xCD => InstructionDecoder::call(scheduler),
            0xCE => InstructionDecoder::add_immediate_to_reg_a_and_write_to_reg_a(scheduler, true),
            0xCF => InstructionDecoder::restart(scheduler, opcode),
            0xD0 => InstructionDecoder::return_conditionally(scheduler, opcode),
            0xD1 => InstructionDecoder::pop_stack_to_reg_pair(scheduler, opcode),
            0xD2 => InstructionDecoder::jump_conditional(scheduler, opcode),
            0xD4 => InstructionDecoder::call_conditional(scheduler, opcode),
            0xD5 => InstructionDecoder::push_reg_pair_to_stack(scheduler, opcode),
            0xD6 => InstructionDecoder::subtract_immediate_from_reg_a_and_write_to_reg_a(scheduler, false),
            0xD7 => InstructionDecoder::restart(scheduler, opcode),
            0xD8 => InstructionDecoder::return_conditionally(scheduler, opcode),
            0xD9 => InstructionDecoder::return_from_interrupt(scheduler),
            0xDA => InstructionDecoder::jump_conditional(scheduler, opcode),
            0xDC => InstructionDecoder::call_conditional(scheduler, opcode),
            0xDE => InstructionDecoder::subtract_immediate_from_reg_a_and_write_to_reg_a(scheduler, true),
            0xDF => InstructionDecoder::restart(scheduler, opcode),
            0xE0 => InstructionDecoder::reg_a_to_immediate_indirect_with_offset_ld(scheduler),
            0xE1 => InstructionDecoder::pop_stack_to_reg_pair(scheduler, opcode),
            0xE2 => InstructionDecoder::reg_a_to_indirect_c_ld(scheduler),
            0xE5 => InstructionDecoder::push_reg_pair_to_stack(scheduler, opcode),
            0xE6 => InstructionDecoder::and_immediate_with_reg_a_and_write_to_reg_a(scheduler),
            0xE7 => InstructionDecoder::restart(scheduler, opcode),
            0xE8 => InstructionDecoder::add_immediate_to_reg_sp(scheduler),
            0xE9 => InstructionDecoder::jump_to_indirect_hl(scheduler),
            0xEA => InstructionDecoder::reg_a_to_immediate_indirect_ld(scheduler),
            0xEE => InstructionDecoder::xor_immediate_with_reg_a_and_write_to_reg_a(scheduler),
            0xEF => InstructionDecoder::restart(scheduler, opcode),
            0xF0 => InstructionDecoder::immediate_indirect_with_offset_to_reg_a_ld(scheduler),
            0xF1 => InstructionDecoder::pop_stack_to_reg_pair(scheduler, opcode),
            0xF2 => InstructionDecoder::indirect_c_with_offset_to_reg_a_ld(scheduler),
            0xF3 => InstructionDecoder::disable_interrupts(scheduler),
            0xF5 => InstructionDecoder::push_reg_pair_to_stack(scheduler, opcode),
            0xF6 => InstructionDecoder::or_immediate_with_reg_a_and_write_to_reg_a(scheduler),
            0xF7 => InstructionDecoder::restart(scheduler, opcode),
            0xF8 => InstructionDecoder::reg_sp_plus_signed_immediate_to_hl_ld(scheduler),
            0xF9 => InstructionDecoder::reg_hl_to_reg_sp_ld(scheduler),
            0xFA => InstructionDecoder::immediate_indirect_to_reg_a_ld(scheduler),
            0xFB => InstructionDecoder::enable_interrupts(scheduler),
            0xFE => InstructionDecoder::compare_immediate_with_reg_a(scheduler),
            0xFF => InstructionDecoder::restart(scheduler, opcode),
            _ => panic!("Unknown opcode"),
        };
    }

    pub fn decode_cb(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        match opcode.value() {
            0x00..=0x05 => InstructionDecoder::rotate_reg_left(scheduler, opcode),
            0x06 => InstructionDecoder::rotate_indirect_hl_left(scheduler),
            0x07 => InstructionDecoder::rotate_reg_left(scheduler, opcode),
            0x08..=0x0D => InstructionDecoder::rotate_reg_right(scheduler, opcode),
            0x0E => InstructionDecoder::rotate_indirect_hl_right(scheduler),
            0x0F => InstructionDecoder::rotate_reg_right(scheduler, opcode),
            0x10..=0x15 => InstructionDecoder::rotate_reg_left_through_carry(scheduler, opcode),
            0x16 => InstructionDecoder::rotate_indirect_hl_left_through_carry(scheduler),
            0x17 => InstructionDecoder::rotate_reg_left_through_carry(scheduler, opcode),
            0x18..=0x1D => InstructionDecoder::rotate_reg_right_through_carry(scheduler, opcode),
            0x1E => InstructionDecoder::rotate_indirect_hl_right_through_carry(scheduler),
            0x1F => InstructionDecoder::rotate_reg_right_through_carry(scheduler, opcode),
            0x20..=0x25 => InstructionDecoder::shift_reg_left(scheduler, opcode),
            0x26 => InstructionDecoder::shift_indirect_hl_left(scheduler),
            0x27 => InstructionDecoder::shift_reg_left(scheduler, opcode),
            0x28..=0x2D => InstructionDecoder::shift_reg_right_arithmetic(scheduler, opcode),
            0x2E => InstructionDecoder::shift_indirect_hl_right_arithmetic(scheduler),
            0x2F => InstructionDecoder::shift_reg_right_arithmetic(scheduler, opcode),
            0x30..=0x35 => InstructionDecoder::swap_reg(scheduler, opcode),
            0x36 => InstructionDecoder::swap_indirect_hl(scheduler),
            0x37 => InstructionDecoder::swap_reg(scheduler, opcode),
            0x38..=0x3D => InstructionDecoder::shift_reg_right(scheduler, opcode),
            0x3E => InstructionDecoder::shift_indirect_hl_right(scheduler),
            0x3F => InstructionDecoder::shift_reg_right(scheduler, opcode),
            0x40..=0x45 => InstructionDecoder::get_reg_bit(scheduler, opcode),
            0x46 => InstructionDecoder::get_indirect_hl_bit(scheduler, opcode),
            0x47..=0x4D => InstructionDecoder::get_reg_bit(scheduler, opcode),
            0x4E => InstructionDecoder::get_indirect_hl_bit(scheduler, opcode),
            0x4F..=0x55 => InstructionDecoder::get_reg_bit(scheduler, opcode),
            0x56 => InstructionDecoder::get_indirect_hl_bit(scheduler, opcode),
            0x57..=0x5D => InstructionDecoder::get_reg_bit(scheduler, opcode),
            0x5E => InstructionDecoder::get_indirect_hl_bit(scheduler, opcode),
            0x5F..=0x65 => InstructionDecoder::get_reg_bit(scheduler, opcode),
            0x66 => InstructionDecoder::get_indirect_hl_bit(scheduler, opcode),
            0x67..=0x6D => InstructionDecoder::get_reg_bit(scheduler, opcode),
            0x6E => InstructionDecoder::get_indirect_hl_bit(scheduler, opcode),
            0x6F..=0x75 => InstructionDecoder::get_reg_bit(scheduler, opcode),
            0x76 => InstructionDecoder::get_indirect_hl_bit(scheduler, opcode),
            0x77..=0x7D => InstructionDecoder::get_reg_bit(scheduler, opcode),
            0x7E => InstructionDecoder::get_indirect_hl_bit(scheduler, opcode),
            0x7F => InstructionDecoder::get_reg_bit(scheduler, opcode),
            0x80..=0x85 => InstructionDecoder::reset_reg_bit(scheduler, opcode),
            0x86 => InstructionDecoder::reset_indirect_hl_bit(scheduler, opcode),
            0x87..=0x8D => InstructionDecoder::reset_reg_bit(scheduler, opcode),
            0x8E => InstructionDecoder::reset_indirect_hl_bit(scheduler, opcode),
            0x8F..=0x95 => InstructionDecoder::reset_reg_bit(scheduler, opcode),
            0x96 => InstructionDecoder::reset_indirect_hl_bit(scheduler, opcode),
            0x97..=0x9D => InstructionDecoder::reset_reg_bit(scheduler, opcode),
            0x9E => InstructionDecoder::reset_indirect_hl_bit(scheduler, opcode),
            0x9F..=0xA5 => InstructionDecoder::reset_reg_bit(scheduler, opcode),
            0xA6 => InstructionDecoder::reset_indirect_hl_bit(scheduler, opcode),
            0xA7..=0xAD => InstructionDecoder::reset_reg_bit(scheduler, opcode),
            0xAE => InstructionDecoder::reset_indirect_hl_bit(scheduler, opcode),
            0xAF..=0xB5 => InstructionDecoder::reset_reg_bit(scheduler, opcode),
            0xB6 => InstructionDecoder::reset_indirect_hl_bit(scheduler, opcode),
            0xB7..=0xBD => InstructionDecoder::reset_reg_bit(scheduler, opcode),
            0xBE => InstructionDecoder::reset_indirect_hl_bit(scheduler, opcode),
            0xBF => InstructionDecoder::reset_reg_bit(scheduler, opcode),
            0xC0..=0xC5 => InstructionDecoder::set_reg_bit(scheduler, opcode),
            0xC6 => InstructionDecoder::set_indirect_hl_bit(scheduler, opcode),
            0xC7..=0xCD => InstructionDecoder::set_reg_bit(scheduler, opcode),
            0xCE => InstructionDecoder::set_indirect_hl_bit(scheduler, opcode),
            0xCF..=0xD5 => InstructionDecoder::set_reg_bit(scheduler, opcode),
            0xD6 => InstructionDecoder::set_indirect_hl_bit(scheduler, opcode),
            0xD7..=0xDD => InstructionDecoder::set_reg_bit(scheduler, opcode),
            0xDE => InstructionDecoder::set_indirect_hl_bit(scheduler, opcode),
            0xDF..=0xE5 => InstructionDecoder::set_reg_bit(scheduler, opcode),
            0xE6 => InstructionDecoder::set_indirect_hl_bit(scheduler, opcode),
            0xE7..=0xED => InstructionDecoder::set_reg_bit(scheduler, opcode),
            0xEE => InstructionDecoder::set_indirect_hl_bit(scheduler, opcode),
            0xEF..=0xF5 => InstructionDecoder::set_reg_bit(scheduler, opcode),
            0xF6 => InstructionDecoder::set_indirect_hl_bit(scheduler, opcode),
            0xF7..=0xFD => InstructionDecoder::set_reg_bit(scheduler, opcode),
            0xFE => InstructionDecoder::set_indirect_hl_bit(scheduler, opcode),
            0xFF => InstructionDecoder::set_reg_bit(scheduler, opcode),
        };
    }

    fn reg_to_reg_ld(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::from_r_bits(opcode.z_bits())),
                destination: ByteLocation::Register(ByteRegister::from_r_bits(opcode.y_bits())),
            })
        );
    }

    fn immediate_to_reg_ld(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::NextMemoryByte,
                destination: ByteLocation::Register(ByteRegister::from_r_bits(opcode.y_bits())),
            })
        )
    }

    fn immediate_to_indirect_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::NextMemoryByte,
                destination: ByteLocation::ByteBuffer,
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::ByteBuffer,
                destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            })
        );
    }

    fn indirect_to_reg_ld(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
                destination: ByteLocation::Register(ByteRegister::from_r_bits(opcode.y_bits())),
            })
        );
    }

    fn reg_to_indirect_ld(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::from_r_bits(opcode.z_bits())),
                destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            })
        );
    }

    fn indirect_bc_to_reg_a_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::MemoryReferencedByRegister(WordRegister::BC),
                destination: ByteLocation::Register(ByteRegister::A),
            })
        );
    }

    fn indirect_de_to_reg_a_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::MemoryReferencedByRegister(WordRegister::DE),
                destination: ByteLocation::Register(ByteRegister::A),
            })
        );
    }

    fn indirect_c_with_offset_to_reg_a_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Value(0xFF),
                destination: ByteLocation::UpperAddressBuffer,
            })
        );
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::C),
                destination: ByteLocation::LowerAddressBuffer,
            }),
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::MemoryReferencedByAddressBuffer,
                destination: ByteLocation::Register(ByteRegister::A),
            })
        );
    }

    fn reg_a_to_indirect_c_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Value(0xFF),
                destination: ByteLocation::UpperAddressBuffer,
            })
        );
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::C),
                destination: ByteLocation::LowerAddressBuffer,
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::A),
                destination: ByteLocation::MemoryReferencedByAddressBuffer,
            })
        );
    }

    fn immediate_indirect_with_offset_to_reg_a_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Value(0xFF),
                destination: ByteLocation::UpperAddressBuffer,
            })
        );
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::NextMemoryByte,
                destination: ByteLocation::LowerAddressBuffer,
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::MemoryReferencedByAddressBuffer,
                destination: ByteLocation::Register(ByteRegister::A),
            })
        );
    }

    fn reg_a_to_immediate_indirect_with_offset_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Value(0xFF),
                destination: ByteLocation::UpperAddressBuffer,
            })
        );
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::NextMemoryByte,
                destination: ByteLocation::LowerAddressBuffer,
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::A),
                destination: ByteLocation::MemoryReferencedByAddressBuffer,
            })
        );
    }

    fn immediate_indirect_to_reg_a_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::NextMemoryByte,
                destination: ByteLocation::LowerAddressBuffer,
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::NextMemoryByte,
                destination: ByteLocation::UpperAddressBuffer,
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::MemoryReferencedByAddressBuffer,
                destination: ByteLocation::Register(ByteRegister::A),
            })
        );
    }

    fn reg_a_to_immediate_indirect_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::NextMemoryByte,
                destination: ByteLocation::LowerAddressBuffer,
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::NextMemoryByte,
                destination: ByteLocation::UpperAddressBuffer,
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::A),
                destination: ByteLocation::MemoryReferencedByAddressBuffer,
            })
        );
    }

    fn indirect_hl_to_reg_a_ld_and_increment(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
                destination: ByteLocation::Register(ByteRegister::A),
            })
        );
        scheduler.schedule(
            IncrementWord(WordLocation::Register(WordRegister::HL))
        );
    }

    fn indirect_hl_to_reg_a_ld_and_decrement(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
                destination: ByteLocation::Register(ByteRegister::A),
            })
        );
        scheduler.schedule(
            DecrementWord(WordLocation::Register(WordRegister::HL))
        );
    }

    fn reg_a_to_indirect_bc_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::A),
                destination: ByteLocation::MemoryReferencedByRegister(WordRegister::BC),
            })
        );
    }

    fn reg_a_to_indirect_de_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::A),
                destination: ByteLocation::MemoryReferencedByRegister(WordRegister::DE),
            })
        );
    }

    fn reg_a_to_indirect_hl_ld_and_increment(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::A),
                destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            })
        );
        scheduler.schedule(
            IncrementWord(WordLocation::Register(WordRegister::HL))
        );
    }

    fn reg_a_to_indirect_hl_ld_and_decrement(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::A),
                destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            })
        );
        scheduler.schedule(
            DecrementWord(WordLocation::Register(WordRegister::HL)),
        );
    }

    fn immediate_to_reg_pair_ld(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = WordRegister::from_dd_bits(opcode.dd_bits());
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::NextMemoryByte,
                destination: ByteLocation::Register(register.get_lower_byte_register()),
            })
        );
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::NextMemoryByte,
                destination: ByteLocation::Register(register.get_upper_byte_register()),
            })
        );
    }

    fn reg_hl_to_reg_sp_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::LowerHL),
                destination: ByteLocation::Register(ByteRegister::LowerSP),
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(ByteRegister::UpperHL),
                destination: ByteLocation::Register(ByteRegister::UpperSP),
            })
        );
    }

    fn push_reg_pair_to_stack(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = WordRegister::from_qq_bits(opcode.qq_bits());
        scheduler.schedule(Defer);
        scheduler.schedule(
            DecrementWord(WordLocation::Register(WordRegister::SP))
        );
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(register.get_upper_byte_register()),
                destination: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            DecrementWord(WordLocation::Register(WordRegister::SP))
        );
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Register(register.get_lower_byte_register()),
                destination: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(Noop); // Normally we'd decrement the SP by 2 here, but we've already done this in the previous steps
    }

    fn pop_stack_to_reg_pair(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = WordRegister::from_qq_bits(opcode.qq_bits());
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
                destination: ByteLocation::Register(register.get_lower_byte_register()),
            })
        );
        scheduler.schedule(
            IncrementWord(WordLocation::Register(WordRegister::SP))
        );
        scheduler.schedule(Defer);
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
                destination: ByteLocation::Register(register.get_upper_byte_register()),
            })
        );
        scheduler.schedule(
            IncrementWord(WordLocation::Register(WordRegister::SP))
        );
    }

    // TODO: Do a more thorough check to see if this is correct. There seems to be a lot of confusion surrounding the (half) carry bits
    fn reg_sp_plus_signed_immediate_to_hl_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(
            MoveByte(ByteOperationParams {
                source: ByteLocation::Value(0x00),
                destination: ByteLocation::Register(ByteRegister::F),
            })
        );
        scheduler.schedule(Defer);
        scheduler.schedule(CastByteToSignedWord(ByteCastingParams {
            source: ByteLocation::NextMemoryByte,
            destination: WordLocation::WordBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(AddWords(WordArithmeticParams {
            first: WordLocation::Register(WordRegister::SP),
            second: WordLocation::WordBuffer,
            destination: WordLocation::Register(WordRegister::HL),
            set_flag: true,
            reset_zero_flag: true,
        }));
    }

    fn reg_sp_to_immediate_indirect_ld(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::LowerAddressBuffer,
        }));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::UpperAddressBuffer,
        }));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::LowerSP),
            destination: ByteLocation::MemoryReferencedByAddressBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(IncrementWord(WordLocation::AddressBuffer));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::UpperSP),
            destination: ByteLocation::MemoryReferencedByAddressBuffer,
        }));
    }

    fn add_reg_to_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler, opcode: Opcode, use_carry: bool) {
        scheduler.schedule(AddBytes(ByteArithmeticParams {
            first: ByteLocation::Register(ByteRegister::A),
            second: ByteLocation::Register(ByteRegister::from_r_bits(opcode.z_bits())),
            destination: ByteLocation::Register(ByteRegister::A),
            use_carry,
            flag_mask: 0xF0,
        }));
    }

    fn add_immediate_to_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler, use_carry: bool) {
        scheduler.schedule(Defer);
        scheduler.schedule(AddBytes(ByteArithmeticParams {
            first: ByteLocation::Register(ByteRegister::A),
            second: ByteLocation::NextMemoryByte,
            destination: ByteLocation::Register(ByteRegister::A),
            use_carry,
            flag_mask: 0xF0,
        }));
    }

    fn add_indirect_hl_to_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler, use_carry: bool) {
        scheduler.schedule(Defer);
        scheduler.schedule(AddBytes(ByteArithmeticParams {
            first: ByteLocation::Register(ByteRegister::A),
            second: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::Register(ByteRegister::A),
            use_carry,
            flag_mask: 0xF0,
        }));
    }

    fn subtract_reg_from_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler, opcode: Opcode, use_carry: bool) {
        scheduler.schedule(SubtractBytes(ByteArithmeticParams {
            first: ByteLocation::Register(ByteRegister::A),
            second: ByteLocation::Register(ByteRegister::from_r_bits(opcode.z_bits())),
            destination: ByteLocation::Register(ByteRegister::A),
            use_carry,
            flag_mask: 0xF0,
        }));
    }

    fn subtract_immediate_from_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler, use_carry: bool) {
        scheduler.schedule(Defer);
        scheduler.schedule(SubtractBytes(ByteArithmeticParams {
            first: ByteLocation::Register(ByteRegister::A),
            second: ByteLocation::NextMemoryByte,
            destination: ByteLocation::Register(ByteRegister::A),
            use_carry,
            flag_mask: 0xF0,
        }));
    }

    fn subtract_indirect_hl_from_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler, use_carry: bool) {
        scheduler.schedule(Defer);
        scheduler.schedule(SubtractBytes(ByteArithmeticParams {
            first: ByteLocation::Register(ByteRegister::A),
            second: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::Register(ByteRegister::A),
            use_carry,
            flag_mask: 0xF0,
        }));
    }

    fn and_reg_with_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(AndBytes(ByteLogicParams {
            first: ByteLocation::Register(ByteRegister::from_r_bits(opcode.z_bits())),
            second: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
        }));
    }

    fn and_immediate_with_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(AndBytes(ByteLogicParams {
            first: ByteLocation::NextMemoryByte,
            second: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
        }));
    }

    fn and_indirect_hl_with_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(AndBytes(ByteLogicParams {
            first: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            second: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
        }));
    }

    fn or_reg_with_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(OrBytes(ByteLogicParams {
            first: ByteLocation::Register(ByteRegister::from_r_bits(opcode.z_bits())),
            second: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
        }));
    }

    fn or_immediate_with_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(OrBytes(ByteLogicParams {
            first: ByteLocation::NextMemoryByte,
            second: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
        }));
    }

    fn or_indirect_hl_with_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(OrBytes(ByteLogicParams {
            first: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            second: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
        }));
    }

    fn xor_reg_with_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(XorBytes(ByteLogicParams {
            first: ByteLocation::Register(ByteRegister::from_r_bits(opcode.z_bits())),
            second: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
        }));
    }

    fn xor_immediate_with_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(XorBytes(ByteLogicParams {
            first: ByteLocation::NextMemoryByte,
            second: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
        }));
    }

    fn xor_indirect_hl_with_reg_a_and_write_to_reg_a(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(XorBytes(ByteLogicParams {
            first: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            second: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
        }));
    }

    fn compare_reg_with_reg_a(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(SubtractBytes(ByteArithmeticParams {
            first: ByteLocation::Register(ByteRegister::A),
            second: ByteLocation::Register(ByteRegister::from_r_bits(opcode.z_bits())),
            destination: ByteLocation::ByteBuffer,
            use_carry: false,
            flag_mask: 0xF0,
        }));
    }

    fn compare_immediate_with_reg_a(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(SubtractBytes(ByteArithmeticParams {
            first: ByteLocation::Register(ByteRegister::A),
            second: ByteLocation::NextMemoryByte,
            destination: ByteLocation::ByteBuffer,
            use_carry: false,
            flag_mask: 0xF0,
        }));
    }

    fn compare_indirect_hl_with_reg_a(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(SubtractBytes(ByteArithmeticParams {
            first: ByteLocation::Register(ByteRegister::A),
            second: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
            use_carry: false,
            flag_mask: 0xF0,
        }));
    }

    fn increment_reg(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = ByteRegister::from_r_bits(opcode.y_bits());
        scheduler.schedule(AddBytes(ByteArithmeticParams {
            first: ByteLocation::Register(register),
            second: ByteLocation::Value(1),
            destination: ByteLocation::Register(register),
            use_carry: false,
            flag_mask: 0xE0,
        }));
    }

    fn increment_indirect_hl(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(AddBytes(ByteArithmeticParams {
            first: ByteLocation::ByteBuffer,
            second: ByteLocation::Value(1),
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            use_carry: false,
            flag_mask: 0xE0,
        }));
    }

    fn decrement_reg(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = ByteRegister::from_r_bits(opcode.y_bits());
        scheduler.schedule(SubtractBytes(ByteArithmeticParams {
            first: ByteLocation::Register(register),
            second: ByteLocation::Value(1),
            destination: ByteLocation::Register(register),
            use_carry: false,
            flag_mask: 0xE0,
        }));
    }

    fn decrement_indirect_hl(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(SubtractBytes(ByteArithmeticParams {
            first: ByteLocation::ByteBuffer,
            second: ByteLocation::Value(1),
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            use_carry: false,
            flag_mask: 0xE0,
        }));
    }

    fn add_reg_pair_to_reg_hl(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = WordRegister::from_dd_bits(opcode.dd_bits());
        scheduler.schedule(AddWords(WordArithmeticParams {
            first: WordLocation::Register(register),
            second: WordLocation::Register(WordRegister::HL),
            destination: WordLocation::WordBuffer,
            set_flag: true,
            reset_zero_flag: false,
        }));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::LowerWordBuffer,
            destination: ByteLocation::Register(ByteRegister::LowerHL),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::UpperWordBuffer,
            destination: ByteLocation::Register(ByteRegister::UpperHL),
        }));
    }

    //TODO: Check whether the flags are set correctly
    fn add_immediate_to_reg_sp(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(CastByteToSignedWord(ByteCastingParams {
            source: ByteLocation::NextMemoryByte,
            destination: WordLocation::WordBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(AddWords(WordArithmeticParams {
            first: WordLocation::Register(WordRegister::SP),
            second: WordLocation::WordBuffer,
            destination: WordLocation::WordBuffer,
            set_flag: true,
            reset_zero_flag: true,
        }));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::LowerWordBuffer,
            destination: ByteLocation::Register(ByteRegister::LowerSP),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::UpperWordBuffer,
            destination: ByteLocation::Register(ByteRegister::UpperSP),
        }));
    }

    fn increment_reg_pair(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = WordRegister::from_dd_bits(opcode.dd_bits());
        scheduler.schedule(MoveWord(WordOperationParams {
            source: WordLocation::Register(register),
            destination: WordLocation::WordBuffer,
        }));
        scheduler.schedule(IncrementWord(WordLocation::WordBuffer));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::LowerWordBuffer,
            destination: ByteLocation::Register(register.get_lower_byte_register()),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::UpperWordBuffer,
            destination: ByteLocation::Register(register.get_upper_byte_register()),
        }));
    }

    fn decrement_reg_pair(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = WordRegister::from_dd_bits(opcode.dd_bits());
        scheduler.schedule(MoveWord(WordOperationParams {
            source: WordLocation::Register(register),
            destination: WordLocation::WordBuffer,
        }));
        scheduler.schedule(DecrementWord(WordLocation::WordBuffer));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::LowerWordBuffer,
            destination: ByteLocation::Register(register.get_lower_byte_register()),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::UpperWordBuffer,
            destination: ByteLocation::Register(register.get_upper_byte_register()),
        }));
    }

    fn rotate_reg_a_left(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(RotateByteLeft(ByteRotationParams {
            source: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
            unset_zero: true,
        }));
    }

    fn rotate_reg_left(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = ByteRegister::from_r_bits(opcode.z_bits());
        scheduler.schedule(RotateByteLeft(ByteRotationParams {
            source: ByteLocation::Register(register),
            destination: ByteLocation::Register(register),
            unset_zero: false,
        }));
    }

    fn rotate_indirect_hl_left(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(RotateByteLeft(ByteRotationParams {
            source: ByteLocation::ByteBuffer,
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            unset_zero: false,
        }));
    }

    fn rotate_reg_a_left_through_carry(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(RotateByteLeftThroughCarry(ByteRotationParams {
            source: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
            unset_zero: true,
        }));
    }

    fn rotate_reg_left_through_carry(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = ByteRegister::from_r_bits(opcode.z_bits());
        scheduler.schedule(RotateByteLeftThroughCarry(ByteRotationParams {
            source: ByteLocation::Register(register),
            destination: ByteLocation::Register(register),
            unset_zero: false,
        }));
    }

    fn rotate_indirect_hl_left_through_carry(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(RotateByteLeftThroughCarry(ByteRotationParams {
            source: ByteLocation::ByteBuffer,
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            unset_zero: false,
        }));
    }

    fn rotate_reg_a_right(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(RotateByteRight(ByteRotationParams {
            source: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
            unset_zero: true,
        }));
    }

    fn rotate_reg_right(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = ByteRegister::from_r_bits(opcode.z_bits());
        scheduler.schedule(RotateByteRight(ByteRotationParams {
            source: ByteLocation::Register(register),
            destination: ByteLocation::Register(register),
            unset_zero: false,
        }));
    }

    fn rotate_indirect_hl_right(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(RotateByteRight(ByteRotationParams {
            source: ByteLocation::ByteBuffer,
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            unset_zero: false,
        }));
    }

    fn rotate_reg_a_right_through_carry(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(RotateByteRightThroughCarry(ByteRotationParams {
            source: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
            unset_zero: true,
        }));
    }

    fn rotate_reg_right_through_carry(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = ByteRegister::from_r_bits(opcode.z_bits());
        scheduler.schedule(RotateByteRightThroughCarry(ByteRotationParams {
            source: ByteLocation::Register(register),
            destination: ByteLocation::Register(register),
            unset_zero: false,
        }));
    }

    fn rotate_indirect_hl_right_through_carry(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(RotateByteRightThroughCarry(ByteRotationParams {
            source: ByteLocation::ByteBuffer,
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            unset_zero: false,
        }));
    }

    fn shift_reg_left(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = ByteRegister::from_r_bits(opcode.z_bits());
        scheduler.schedule(ShiftByteLeft(ByteShiftParams {
            source: ByteLocation::Register(register),
            destination: ByteLocation::Register(register),
            arithmetic: false,
        }));
    }

    fn shift_reg_right(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = ByteRegister::from_r_bits(opcode.z_bits());
        scheduler.schedule(ShiftByteRight(ByteShiftParams {
            source: ByteLocation::Register(register),
            destination: ByteLocation::Register(register),
            arithmetic: false,
        }));
    }

    fn shift_reg_right_arithmetic(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = ByteRegister::from_r_bits(opcode.z_bits());
        scheduler.schedule(ShiftByteRight(ByteShiftParams {
            source: ByteLocation::Register(register),
            destination: ByteLocation::Register(register),
            arithmetic: true,
        }));
    }

    fn shift_indirect_hl_left(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(ShiftByteLeft(ByteShiftParams {
            source: ByteLocation::ByteBuffer,
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            arithmetic: false,
        }));
    }

    fn shift_indirect_hl_right(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(ShiftByteRight(ByteShiftParams {
            source: ByteLocation::ByteBuffer,
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            arithmetic: false,
        }));
    }

    fn shift_indirect_hl_right_arithmetic(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(ShiftByteRight(ByteShiftParams {
            source: ByteLocation::ByteBuffer,
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            arithmetic: true,
        }));
    }

    fn swap_reg(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let register = ByteRegister::from_r_bits(opcode.z_bits());
        scheduler.schedule(SwapByte(ByteOperationParams {
            source: ByteLocation::Register(register),
            destination: ByteLocation::Register(register),
        }));
    }

    fn swap_indirect_hl(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(SwapByte(ByteOperationParams {
            source: ByteLocation::ByteBuffer,
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
        }));
    }

    fn get_reg_bit(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let bit_number = opcode.y_bits();
        let register = ByteLocation::Register(ByteRegister::from_r_bits(opcode.z_bits()));
        scheduler.schedule(GetBitFromByte(register, bit_number));
    }

    fn get_indirect_hl_bit(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(Defer);
        let bit_number = opcode.y_bits();
        scheduler.schedule(GetBitFromByte(
            ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            bit_number,
        ));
    }

    fn set_reg_bit(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let bit_number = opcode.y_bits();
        let register = ByteRegister::from_r_bits(opcode.z_bits());
        scheduler.schedule(SetBitOnByte(ByteOperationParams {
            source: ByteLocation::Register(register),
            destination: ByteLocation::Register(register),
        }, bit_number));
    }

    fn set_indirect_hl_bit(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let bit_number = opcode.y_bits();
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(SetBitOnByte(ByteOperationParams {
            source: ByteLocation::ByteBuffer,
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
        }, bit_number));
    }

    fn reset_reg_bit(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let bit_number = opcode.y_bits();
        let register = ByteRegister::from_r_bits(opcode.z_bits());
        scheduler.schedule(ResetBitOnByte(ByteOperationParams {
            source: ByteLocation::Register(register),
            destination: ByteLocation::Register(register),
        }, bit_number));
    }

    fn reset_indirect_hl_bit(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        let bit_number = opcode.y_bits();
        scheduler.schedule(ResetBitOnByte(ByteOperationParams {
            source: ByteLocation::ByteBuffer,
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::HL),
        }, bit_number));
    }

    fn jump(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::LowerAddressBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::UpperAddressBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveWord(WordOperationParams {
            source: WordLocation::AddressBuffer,
            destination: WordLocation::Register(WordRegister::PC),
        }));
    }

    fn get_branching_instruction(opcode: Opcode) -> Instruction {
        let condition = opcode.cc_bits();
        match condition {
            0x00 => BranchIfNotZero,
            0x01 => BranchIfZero,
            0x02 => BranchIfNotCarry,
            0x03 => BranchIfCarry,
            _ => panic!("{} doesn't map to a condition value", condition)
        }
    }

    fn jump_conditional(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::LowerAddressBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::UpperAddressBuffer,
        }));
        scheduler.schedule(InstructionDecoder::get_branching_instruction(opcode));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveWord(WordOperationParams {
            source: WordLocation::AddressBuffer,
            destination: WordLocation::Register(WordRegister::PC),
        }));
        scheduler.schedule(EndBranch);
    }

    fn jump_relative(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(CastByteToSignedWord(ByteCastingParams {
            source: ByteLocation::ByteBuffer,
            destination: WordLocation::WordBuffer,
        }));
        scheduler.schedule(AddWords(WordArithmeticParams {
            first: WordLocation::Register(WordRegister::PC),
            second: WordLocation::WordBuffer,
            destination: WordLocation::Register(WordRegister::PC),
            set_flag: false,
            reset_zero_flag: false
        }));
    }

    fn jump_conditional_relative(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::ByteBuffer,
        }));
        scheduler.schedule(InstructionDecoder::get_branching_instruction(opcode));
        scheduler.schedule(Defer);
        scheduler.schedule(CastByteToSignedWord(ByteCastingParams {
            source: ByteLocation::ByteBuffer,
            destination: WordLocation::WordBuffer,
        }));
        scheduler.schedule(AddWords(WordArithmeticParams {
            first: WordLocation::Register(WordRegister::PC),
            second: WordLocation::WordBuffer,
            destination: WordLocation::Register(WordRegister::PC),
            set_flag: false,
            reset_zero_flag: false,
        }));
        scheduler.schedule(EndBranch);
    }

    fn jump_to_indirect_hl(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(MoveWord(WordOperationParams {
            source: WordLocation::Register(WordRegister::HL),
            destination: WordLocation::Register(WordRegister::PC),
        }));
    }

    pub fn schedule_call_interrupt_routine(scheduler: &mut dyn InstructionScheduler, interrupt: Interrupt) {
        scheduler.schedule(ClearInterrupt(interrupt));
        scheduler.schedule(DisableInterrupts);
        scheduler.schedule(Defer);
        scheduler.schedule(Noop);
        scheduler.schedule(DecrementWord(WordLocation::Register(WordRegister::SP)));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::UpperPC),
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(DecrementWord(WordLocation::Register(WordRegister::SP)));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::LowerPC),
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveWord(WordOperationParams {
            source: WordLocation::Value(interrupt.get_routine_address()),
            destination: WordLocation::Register(WordRegister::PC),
        }));
    }

    fn call(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::LowerAddressBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::UpperAddressBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(DecrementWord(WordLocation::Register(WordRegister::SP)));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::UpperPC),
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(DecrementWord(WordLocation::Register(WordRegister::SP)));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::LowerPC),
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveWord(WordOperationParams {
            source: WordLocation::AddressBuffer,
            destination: WordLocation::Register(WordRegister::PC),
        }));
    }

    fn call_conditional(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::LowerAddressBuffer,
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::NextMemoryByte,
            destination: ByteLocation::UpperAddressBuffer,
        }));
        scheduler.schedule(InstructionDecoder::get_branching_instruction(opcode));
        scheduler.schedule(Defer);
        scheduler.schedule(DecrementWord(WordLocation::Register(WordRegister::SP)));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::UpperPC),
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(DecrementWord(WordLocation::Register(WordRegister::SP)));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::LowerPC),
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveWord(WordOperationParams {
            source: WordLocation::AddressBuffer,
            destination: WordLocation::Register(WordRegister::PC),
        }));
        scheduler.schedule(EndBranch);
    }

    fn return_from_call(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
            destination: ByteLocation::LowerWordBuffer,
        }));
        scheduler.schedule(IncrementWord(WordLocation::Register(WordRegister::SP)));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
            destination: ByteLocation::UpperWordBuffer,
        }));
        scheduler.schedule(IncrementWord(WordLocation::Register(WordRegister::SP)));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveWord(WordOperationParams {
            source: WordLocation::WordBuffer,
            destination: WordLocation::Register(WordRegister::PC),
        }));
    }

    fn return_from_interrupt(scheduler: &mut dyn InstructionScheduler) {
        InstructionDecoder::return_from_call(scheduler);
        scheduler.schedule(EnableInterrupts);
    }

    fn return_conditionally(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        scheduler.schedule(Defer);
        scheduler.schedule(InstructionDecoder::get_branching_instruction(opcode));
        InstructionDecoder::return_from_call(scheduler);
        scheduler.schedule(EndBranch);
    }

    fn restart(scheduler: &mut dyn InstructionScheduler, opcode: Opcode) {
        let address = match opcode.y_bits() {
            0 => 0x0000u16,
            1 => 0x0008u16,
            2 => 0x0010u16,
            3 => 0x0018u16,
            4 => 0x0020u16,
            5 => 0x0028u16,
            6 => 0x0030u16,
            7 => 0x0038u16,
            _ => panic!("{} is not a valid restart code", opcode.y_bits())
        };
        scheduler.schedule(Defer);
        scheduler.schedule(DecrementWord(WordLocation::Register(WordRegister::SP)));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::UpperPC),
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(DecrementWord(WordLocation::Register(WordRegister::SP)));
        scheduler.schedule(MoveByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::LowerPC),
            destination: ByteLocation::MemoryReferencedByRegister(WordRegister::SP),
        }));
        scheduler.schedule(Defer);
        scheduler.schedule(MoveWord(WordOperationParams {
            source: WordLocation::Value(address),
            destination: WordLocation::Register(WordRegister::PC),
        }));
    }

    fn decimal_adjust_reg_a(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(DecimalAdjust);
    }

    fn ones_complement_reg_a(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(OnesComplementByte(ByteOperationParams {
            source: ByteLocation::Register(ByteRegister::A),
            destination: ByteLocation::Register(ByteRegister::A),
        }));
    }

    fn flip_carry_flag(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(FlipCarry);
    }

    fn set_carry_flag(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(SetCarry);
    }

    fn disable_interrupts(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(DisableInterrupts);
    }

    fn enable_interrupts(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(EnableInterrupts);
    }

    fn halt(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Halt);
    }

    fn stop(scheduler: &mut dyn InstructionScheduler) {
        scheduler.schedule(Stop)
    }
}