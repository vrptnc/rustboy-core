use serde::{Deserialize, Serialize};

use crate::internal::cpu::interrupts::Interrupt;
use crate::internal::cpu::register::{ByteRegister, WordRegister};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ByteLocation {
  Value(u8),
  Register(ByteRegister),
  ByteBuffer,
  LowerAddressBuffer,
  UpperAddressBuffer,
  LowerWordBuffer,
  UpperWordBuffer,
  NextMemoryByte,
  MemoryReferencedByAddressBuffer,
  MemoryReferencedByRegister(WordRegister),
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum WordLocation {
  Value(u16),
  Register(WordRegister),
  WordBuffer,
  AddressBuffer,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ByteArithmeticParams {
  pub first: ByteLocation,
  pub second: ByteLocation,
  pub destination: ByteLocation,
  pub use_carry: bool,
  pub flag_mask: u8,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ByteOperationParams {
  pub source: ByteLocation,
  pub destination: ByteLocation,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WordOperationParams {
  pub source: WordLocation,
  pub destination: WordLocation,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ByteCastingParams {
  pub source: ByteLocation,
  pub destination: WordLocation
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ByteRotationParams {
  pub source: ByteLocation,
  pub destination: ByteLocation,
  pub unset_zero: bool,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ByteShiftParams {
  pub source: ByteLocation,
  pub destination: ByteLocation,
  pub arithmetic: bool,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ByteLogicParams {
  pub first: ByteLocation,
  pub second: ByteLocation,
  pub destination: ByteLocation,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WordArithmeticParams {
  pub first: WordLocation,
  pub second: WordLocation,
  pub destination: WordLocation,
  pub set_flag: bool,
  pub reset_zero_flag: bool
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Instruction {
  Noop,
  Defer,
  DecodeCBInstruction,
  BranchIfZero,
  BranchIfNotZero,
  BranchIfCarry,
  BranchIfNotCarry,
  EndBranch,
  MoveByte(ByteOperationParams),
  CastByteToSignedWord(ByteCastingParams),
  MoveWord(WordOperationParams),
  IncrementWord(WordLocation),
  DecrementWord(WordLocation),
  AddBytes(ByteArithmeticParams),
  SubtractBytes(ByteArithmeticParams),
  AndBytes(ByteLogicParams),
  OrBytes(ByteLogicParams),
  XorBytes(ByteLogicParams),
  OnesComplementByte(ByteOperationParams),
  RotateByteLeft(ByteRotationParams),
  RotateByteLeftThroughCarry(ByteRotationParams),
  ShiftByteLeft(ByteShiftParams),
  RotateByteRight(ByteRotationParams),
  RotateByteRightThroughCarry(ByteRotationParams),
  ShiftByteRight(ByteShiftParams),
  SwapByte(ByteOperationParams),
  AddWords(WordArithmeticParams),
  DecimalAdjust,
  GetBitFromByte(ByteLocation, u8),
  SetBitOnByte(ByteOperationParams, u8),
  ResetBitOnByte(ByteOperationParams, u8),
  ClearInterrupt(Interrupt),
  EnableInterrupts,
  DisableInterrupts,
  FlipCarry,
  SetCarry,
  Halt,
  Stop
}