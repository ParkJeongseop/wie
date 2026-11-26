#![no_std]
extern crate alloc;

use alloc::{boxed::Box, format, sync::Arc, vec, vec::Vec};
use core::ops::Range;

use spin::Mutex;

use wie_core_arm_aot::{
    CompileRequest, CompiledArtifact, CompiledExecutor, CompiledExit, CompiledHandle, CompiledRegion, ExecutionAccess, ManifestRegion, MemoryPage,
    PreparationFuture, RunFrame,
    ir::{AluOp, BranchTarget, Condition, Instruction, MemoryOperand, Operand, Operation, Reg, RegionIr, Shift, ShiftAmount, Value, Width},
};
use wie_util::{Result, WieError};

// CPSR bits a region is specialized for: Jazelle, Thumb, and the processor mode.
const MODE_MASK: u32 = 0x0100_003f;
const THUMB: u32 = 0x20;

type Pages = [MemoryPage; 0x10000];

/// Bit `NZCV` of entry `condition` tells whether the condition holds for those flags.
const CONDITIONS: [u16; 16] = {
    let mut table = [u16::MAX; 16];
    let mut flags = 0;
    while flags < 16 {
        let (n, z, c, v) = (flags & 8 != 0, flags & 4 != 0, flags & 2 != 0, flags & 1 != 0);
        let holds = [z, !z, c, !c, n, !n, v, !v, c && !z, !c || z, n == v, n != v, !z && n == v, z || n != v];
        let mut condition = 0;
        while condition < holds.len() {
            if !holds[condition] {
                table[condition] &= !(1 << flags);
            }
            condition += 1;
        }
        flags += 1;
    }
    table
};

/// The condition flags, kept apart from the CPSR so that setting them costs no packing.
#[derive(Clone, Copy)]
struct Flags {
    /// Bit 31 is N.
    negative: u32,
    /// Zero when Z is set.
    nonzero: u32,
    carry: u32,
    overflow: u32,
}

impl Flags {
    #[inline(always)]
    fn unpack(cpsr: u32) -> Self {
        Self {
            negative: cpsr,
            nonzero: !cpsr & 0x4000_0000,
            carry: (cpsr >> 29) & 1,
            overflow: (cpsr >> 28) & 1,
        }
    }

    #[inline(always)]
    fn pack(self, cpsr: u32) -> u32 {
        (cpsr & 0x0fff_ffff) | (self.negative & 0x8000_0000) | (u32::from(self.nonzero == 0) << 30) | (self.carry << 29) | (self.overflow << 28)
    }

    #[inline(always)]
    fn holds(self, condition: u8) -> bool {
        let flags = ((self.negative >> 31) << 3) | (u32::from(self.nonzero == 0) << 2) | (self.carry << 1) | self.overflow;
        (CONDITIONS[usize::from(condition) & 15] >> flags) & 1 != 0
    }

    #[inline(always)]
    fn result(&mut self, result: u32) {
        self.negative = result;
        self.nonzero = result;
    }

    #[inline(always)]
    fn add(&mut self, left: u32, right: u32, result: u32) {
        self.result(result);
        self.carry = u32::from(result < left);
        self.overflow = (!(left ^ right) & (left ^ result)) >> 31;
    }

    #[inline(always)]
    fn subtract(&mut self, left: u32, right: u32, result: u32) {
        self.result(result);
        self.carry = u32::from(left >= right);
        self.overflow = ((left ^ right) & (left ^ result)) >> 31;
    }
}

#[derive(Clone, Copy)]
enum Source {
    Register(u8),
    Immediate(u32),
}

enum Shifted {
    /// A folded immediate; an absent carry preserves the current one.
    Constant(u32, Option<bool>),
    Register(u8),
    /// A shift by a nonzero amount.
    Immediate(Source, Shift, u8),
    Variable(Source, Shift, Source),
}

struct Address {
    base: Source,
    offset: Shifted,
    subtract: bool,
    pre_index: bool,
    write_back: Option<u8>,
}

/// Every instruction form, executed by `Region::general`.
enum Op {
    Alu {
        op: AluOp,
        destination: Option<u8>,
        left: Source,
        right: Shifted,
        set_flags: bool,
    },
    /// A fixed transfer to an instruction of this region in the same instruction set.
    Jump {
        index: u16,
        link: Option<u32>,
    },
    /// A fixed transfer out of this region.
    Leave {
        pc: u32,
        thumb: bool,
        link: Option<u32>,
    },
    Branch {
        target: Source,
        exchange: bool,
        link: Option<u32>,
    },
    Load {
        destination: u8,
        address: Address,
        width: Width,
        signed: bool,
    },
    Store {
        value: Source,
        address: Address,
        width: Width,
    },
    MultiplyAccumulate {
        destination: u8,
        left: Source,
        right: Source,
        accumulate: Source,
        set_flags: bool,
    },
    MultiplyLong {
        low: u8,
        high: u8,
        left: Source,
        right: Source,
        signed: bool,
        accumulate: bool,
        set_flags: bool,
    },
    ReadCpsr {
        destination: u8,
    },
    WriteCpsr {
        value: Source,
        mask: u32,
        /// Control-field writes leave so the dispatcher rechecks the mode.
        leave: Option<u32>,
    },
    MultipleTransfer {
        base: u8,
        address: Source,
        offset: u32,
        write_back: Option<u32>,
        registers: u16,
        load: bool,
    },
    DoubleTransfer {
        registers: u16,
        address: Address,
        load: bool,
    },
    Swap {
        destination: u8,
        address: Source,
        value: Source,
        width: Width,
    },
    Nop,
}

/// One step of a region's program. Frequent instruction forms have a step of their own, the rest run as `General`.
/// Registers are never the PC, and memory steps address `rn` plus an offset without write-back.
#[derive(Clone, Copy)]
enum Code {
    /// An index into the region's general operations.
    General(u32),
    /// Skips the next step unless the condition holds.
    Skip(u8),
    Jump(u16),
    /// A fixed transfer out of this region.
    Exit(u32),
    /// The end of a run of consecutive instructions whose successor is not compiled here; not an instruction itself.
    End(u32),
    /// A fixed transfer out of this region that links to the following instruction.
    Call(u32),
    /// An interworking transfer to the address in a register.
    Exchange(u8),
    Move {
        rd: u8,
        rm: u8,
        set_flags: bool,
    },
    MoveImmediate {
        rd: u8,
        set_flags: bool,
        value: u32,
    },
    ShiftLeft {
        rd: u8,
        rm: u8,
        amount: u8,
        set_flags: bool,
    },
    ShiftRight {
        rd: u8,
        rm: u8,
        amount: u8,
        set_flags: bool,
    },
    ShiftRightSigned {
        rd: u8,
        rm: u8,
        amount: u8,
        set_flags: bool,
    },
    ShiftLeftBy {
        rd: u8,
        rm: u8,
        rs: u8,
        set_flags: bool,
    },
    ShiftRightBy {
        rd: u8,
        rm: u8,
        rs: u8,
        set_flags: bool,
    },
    ShiftRightSignedBy {
        rd: u8,
        rm: u8,
        rs: u8,
        set_flags: bool,
    },
    Add {
        rd: u8,
        rn: u8,
        rm: u8,
        set_flags: bool,
    },
    AddImmediate {
        rd: u8,
        rn: u8,
        set_flags: bool,
        value: u32,
    },
    Subtract {
        rd: u8,
        rn: u8,
        rm: u8,
        set_flags: bool,
    },
    SubtractImmediate {
        rd: u8,
        rn: u8,
        set_flags: bool,
        value: u32,
    },
    Compare {
        rn: u8,
        rm: u8,
    },
    CompareImmediate {
        rn: u8,
        value: u32,
    },
    And {
        rd: u8,
        rn: u8,
        rm: u8,
        set_flags: bool,
    },
    Or {
        rd: u8,
        rn: u8,
        rm: u8,
        set_flags: bool,
    },
    Xor {
        rd: u8,
        rn: u8,
        rm: u8,
        set_flags: bool,
    },
    BitClear {
        rd: u8,
        rn: u8,
        rm: u8,
        set_flags: bool,
    },
    Multiply {
        rd: u8,
        rn: u8,
        rm: u8,
        set_flags: bool,
    },
    Test {
        rn: u8,
        rm: u8,
    },
    LoadWord {
        rd: u8,
        rn: u8,
        offset: u32,
    },
    LoadHalf {
        rd: u8,
        rn: u8,
        offset: u32,
    },
    LoadByte {
        rd: u8,
        rn: u8,
        offset: u32,
    },
    LoadWordIndexed {
        rd: u8,
        rn: u8,
        rm: u8,
    },
    LoadHalfIndexed {
        rd: u8,
        rn: u8,
        rm: u8,
    },
    LoadByteIndexed {
        rd: u8,
        rn: u8,
        rm: u8,
    },
    LoadHalfSignedIndexed {
        rd: u8,
        rn: u8,
        rm: u8,
    },
    LoadByteSignedIndexed {
        rd: u8,
        rn: u8,
        rm: u8,
    },
    LoadWordAt {
        rd: u8,
        address: u32,
    },
    StoreWord {
        rs: u8,
        rn: u8,
        offset: u32,
    },
    StoreHalf {
        rs: u8,
        rn: u8,
        offset: u32,
    },
    StoreByte {
        rs: u8,
        rn: u8,
        offset: u32,
    },
    StoreWordIndexed {
        rs: u8,
        rn: u8,
        rm: u8,
    },
    StoreHalfIndexed {
        rs: u8,
        rn: u8,
        rm: u8,
    },
    StoreByteIndexed {
        rs: u8,
        rn: u8,
        rm: u8,
    },
}

/// What follows a general operation.
enum Step {
    Next,
    Jump(u16),
    /// A computed address, which may still be inside the region.
    Branch(u32),
    Leave(u32),
    Fault(u32),
}

struct Region {
    mode: u32,
    base: u32,
    shift: u32,
    /// Code index of every instruction address from `base`, or `u16::MAX`.
    entries: Vec<u16>,
    code: Vec<Code>,
    /// The instruction address of every step.
    pcs: Vec<u32>,
    general: Vec<Op>,
}

#[inline(always)]
fn read(regs: &[u32; 16], source: Source) -> u32 {
    match source {
        Source::Register(register) => regs[usize::from(register) & 15],
        Source::Immediate(value) => value,
    }
}

/// Shifts by a nonzero amount, returning the value and the shifter carry-out.
#[inline(always)]
fn shift_by(value: u32, shift: &Shift, amount: u32, carry: u32) -> (u32, u32) {
    match shift {
        Shift::Lsl | Shift::Lsr if amount > 32 => (0, 0),
        Shift::Lsl => (value.checked_shl(amount).unwrap_or(0), (value >> (32 - amount)) & 1),
        Shift::Lsr => (value.checked_shr(amount).unwrap_or(0), (value >> (amount - 1)) & 1),
        Shift::Asr => (((value as i32) >> amount.min(31)) as u32, (value >> (amount - 1).min(31)) & 1),
        Shift::Ror => {
            let result = value.rotate_right(amount);
            (result, result >> 31)
        }
        Shift::Rrx => ((carry << 31) | (value >> 1), value & 1),
    }
}

/// Returns the operand value and the shifter carry-out.
#[inline(always)]
fn shift(regs: &[u32; 16], carry: u32, operand: &Shifted) -> (u32, u32) {
    match operand {
        Shifted::Constant(value, constant_carry) => (*value, constant_carry.map_or(carry, u32::from)),
        Shifted::Register(register) => (regs[usize::from(*register) & 15], carry),
        Shifted::Immediate(source, shift, amount) => shift_by(read(regs, *source), shift, u32::from(*amount), carry),
        Shifted::Variable(source, shift, amount) => {
            let value = read(regs, *source);
            match read(regs, *amount) & 255 {
                0 => (value, carry),
                amount => shift_by(value, shift, amount, carry),
            }
        }
    }
}

/// Adds with carry-in; subtraction passes the complemented subtrahend.
#[inline(always)]
fn add_carry(flags: &mut Flags, left: u32, right: u32, set_flags: bool) -> u32 {
    let (sum, first_carry) = left.overflowing_add(right);
    let (result, last_carry) = sum.overflowing_add(flags.carry);
    if set_flags {
        flags.result(result);
        flags.carry = u32::from(first_carry | last_carry);
        flags.overflow = (!(left ^ right) & (left ^ result)) >> 31;
    }
    result
}

/// An interworking PC write: bit 0 selects the instruction set.
#[inline(always)]
fn exchange(cpsr: &mut u32, value: u32) -> u32 {
    if value & 1 != 0 {
        *cpsr |= THUMB;
        value & !1
    } else {
        *cpsr &= !THUMB;
        value & !3
    }
}

/// Reads an aligned little-endian value, or nothing from an unmapped page.
#[inline(always)]
fn load(pages: &Pages, address: u32, width: &Width) -> Option<u32> {
    let page = pages[(address >> 16) as usize].bytes.as_deref()?;
    Some(match width {
        Width::Byte => u32::from(page[(address & 0xffff) as usize]),
        Width::Half => {
            let offset = (address & 0xfffe) as usize;
            u32::from(u16::from_le_bytes([page[offset], page[offset + 1]]))
        }
        Width::Word => {
            let offset = (address & 0xfffc) as usize;
            u32::from_le_bytes([page[offset], page[offset + 1], page[offset + 2], page[offset + 3]])
        }
    })
}

#[inline(always)]
fn store(pages: &mut Pages, address: u32, width: &Width, value: u32) -> Option<()> {
    let page = pages[(address >> 16) as usize].bytes.as_deref_mut()?;
    match width {
        Width::Byte => page[(address & 0xffff) as usize] = value as u8,
        Width::Half => {
            let offset = (address & 0xfffe) as usize;
            page[offset..offset + 2].copy_from_slice(&(value as u16).to_le_bytes());
        }
        Width::Word => {
            let offset = (address & 0xfffc) as usize;
            page[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
    Some(())
}

impl Address {
    /// Returns the accessed address and the write-back value.
    #[inline(always)]
    fn resolve(&self, regs: &[u32; 16], carry: u32) -> (u32, u32) {
        let base = read(regs, self.base);
        let (offset, _) = shift(regs, carry, &self.offset);
        let indexed = if self.subtract {
            base.wrapping_sub(offset)
        } else {
            base.wrapping_add(offset)
        };
        (if self.pre_index { indexed } else { base }, indexed)
    }
}

/// Transfers aligned words between ascending registers and memory, admitting every page before the first access.
/// Returns a loaded PC, or the faulting address.
fn transfer(
    regs: &mut [u32; 16],
    pages: &mut Pages,
    address: u32,
    registers: u16,
    load_registers: bool,
    stored_pc: u32,
) -> core::result::Result<Option<u32>, u32> {
    let count = registers.count_ones();
    if count == 0 {
        return Ok(None);
    }
    let last = address.wrapping_add((count - 1) * 4);
    for address in [address, last] {
        if pages[(address >> 16) as usize].bytes.is_none() {
            return Err(address);
        }
    }
    let mut cursor = address;
    let mut remaining = registers;
    let mut next = None;
    while remaining != 0 {
        let register = remaining.trailing_zeros() as usize;
        remaining &= remaining - 1;
        if load_registers {
            let value = load(pages, cursor, &Width::Word).ok_or(cursor)?;
            if register == 15 {
                next = Some(value);
            } else {
                regs[register] = value;
            }
        } else {
            let value = if register == 15 { stored_pc } else { regs[register] };
            store(pages, cursor, &Width::Word, value).ok_or(cursor)?;
        }
        cursor = (cursor & !3).wrapping_add(4);
    }
    Ok(next)
}

fn lookup(entries: &[u16], base: u32, shift: u32, pc: u32) -> Option<usize> {
    let offset = pc.wrapping_sub(base);
    if offset & ((1 << shift) - 1) != 0 {
        return None;
    }
    entries
        .get((offset >> shift) as usize)
        .filter(|&&index| index != u16::MAX)
        .map(|&index| usize::from(index))
}

impl Region {
    #[inline(always)]
    fn entry(&self, pc: u32, cpsr: u32) -> Option<usize> {
        if cpsr & MODE_MASK != self.mode {
            return None;
        }
        lookup(&self.entries, self.base, self.shift, pc)
    }

    /// Runs from the frame's PC until control leaves this region or the frame's budget runs out.
    /// Fails when the region has no entry there.
    fn run(&self, frame: &mut RunFrame, pages: &mut Pages) -> Option<CompiledExit> {
        let mut index = self.entry(frame.regs[15], frame.cpsr)?;
        let end = frame.end;
        // The flag bits of `cpsr` are stale while `flags` is live.
        let mut cpsr = frame.cpsr;
        let mut flags = Flags::unpack(cpsr);
        let mut budget = frame.budget;
        let regs = &mut frame.regs;
        macro_rules! reg {
            ($register:expr) => {
                regs[usize::from($register) & 15]
            };
        }
        macro_rules! leave {
            ($pc:expr) => {{
                regs[15] = $pc;
                frame.cpsr = flags.pack(cpsr);
                frame.budget = budget;
                return Some(CompiledExit::Dispatch);
            }};
        }
        macro_rules! fault {
            ($address:expr) => {{
                regs[15] = *self.pcs.get(index - 1)?;
                frame.cpsr = flags.pack(cpsr);
                frame.budget = budget;
                frame.fault_address = $address;
                return Some(CompiledExit::GuestFault);
            }};
        }
        macro_rules! branch {
            ($target:expr) => {{
                let target = $target;
                match self.entry(target, cpsr) {
                    Some(next) if target != end => index = next,
                    _ => leave!(target),
                }
            }};
        }
        macro_rules! shift_variable {
            ($rd:expr, $rm:expr, $rs:expr, $set_flags:expr, $shift:expr) => {{
                let value = reg!($rm);
                let (result, carry) = match reg!($rs) & 255 {
                    0 => (value, flags.carry),
                    amount => shift_by(value, &$shift, amount, flags.carry),
                };
                reg!($rd) = result;
                if $set_flags {
                    flags.result(result);
                    flags.carry = carry;
                }
            }};
        }
        macro_rules! load {
            ($rd:expr, $address:expr, $width:expr) => {
                load!($rd, $address, $width, u32)
            };
            ($rd:expr, $address:expr, $width:expr, $extend:ty) => {{
                let address = $address;
                let Some(value) = load(pages, address, &$width) else {
                    fault!(address);
                };
                reg!($rd) = value as $extend as u32;
            }};
        }
        macro_rules! store {
            ($rs:expr, $address:expr, $width:expr) => {{
                let address = $address;
                if store(pages, address, &$width, reg!($rs)).is_none() {
                    fault!(address);
                }
            }};
        }
        loop {
            if budget == 0 {
                leave!(*self.pcs.get(index)?);
            }
            // Every step but a passing `Skip` and an `End` completes one instruction.
            budget -= 1;
            let code = *self.code.get(index)?;
            index += 1;
            match code {
                Code::General(general) => {
                    let pc = *self.pcs.get(index - 1)?;
                    match self.general(self.general.get(general as usize)?, pc, regs, pages, &mut cpsr, &mut flags) {
                        Step::Next => {}
                        Step::Jump(target) => index = usize::from(target),
                        Step::Branch(target) => branch!(target),
                        Step::Leave(pc) => leave!(pc),
                        Step::Fault(address) => fault!(address),
                    }
                }
                Code::Skip(condition) => {
                    if flags.holds(condition) {
                        budget += 1;
                    } else {
                        index += 1;
                    }
                }
                Code::Jump(target) => index = usize::from(target),
                Code::Exit(pc) => leave!(pc),
                Code::End(pc) => {
                    budget += 1;
                    leave!(pc);
                }
                Code::Call(pc) => {
                    regs[14] = self.pcs.get(index - 1)?.wrapping_add(4) | (self.mode & THUMB) >> 5;
                    leave!(pc);
                }
                Code::Exchange(rm) => branch!(exchange(&mut cpsr, reg!(rm))),
                Code::Move { rd, rm, set_flags } => {
                    let result = reg!(rm);
                    reg!(rd) = result;
                    if set_flags {
                        flags.result(result);
                    }
                }
                Code::MoveImmediate { rd, set_flags, value } => {
                    reg!(rd) = value;
                    if set_flags {
                        flags.result(value);
                    }
                }
                Code::ShiftLeft { rd, rm, amount, set_flags } => {
                    let value = reg!(rm);
                    let result = value << (amount & 31);
                    reg!(rd) = result;
                    if set_flags {
                        flags.result(result);
                        flags.carry = (value >> ((32 - u32::from(amount)) & 31)) & 1;
                    }
                }
                Code::ShiftRight { rd, rm, amount, set_flags } => {
                    let value = reg!(rm);
                    let result = value >> (amount & 31);
                    reg!(rd) = result;
                    if set_flags {
                        flags.result(result);
                        flags.carry = (value >> (u32::from(amount).wrapping_sub(1) & 31)) & 1;
                    }
                }
                Code::ShiftRightSigned { rd, rm, amount, set_flags } => {
                    let value = reg!(rm);
                    let result = ((value as i32) >> (amount & 31)) as u32;
                    reg!(rd) = result;
                    if set_flags {
                        flags.result(result);
                        flags.carry = (value >> (u32::from(amount).wrapping_sub(1) & 31)) & 1;
                    }
                }
                Code::ShiftLeftBy { rd, rm, rs, set_flags } => shift_variable!(rd, rm, rs, set_flags, Shift::Lsl),
                Code::ShiftRightBy { rd, rm, rs, set_flags } => shift_variable!(rd, rm, rs, set_flags, Shift::Lsr),
                Code::ShiftRightSignedBy { rd, rm, rs, set_flags } => shift_variable!(rd, rm, rs, set_flags, Shift::Asr),
                Code::Add { rd, rn, rm, set_flags } => {
                    let (left, right) = (reg!(rn), reg!(rm));
                    let result = left.wrapping_add(right);
                    reg!(rd) = result;
                    if set_flags {
                        flags.add(left, right, result);
                    }
                }
                Code::AddImmediate { rd, rn, set_flags, value } => {
                    let left = reg!(rn);
                    let result = left.wrapping_add(value);
                    reg!(rd) = result;
                    if set_flags {
                        flags.add(left, value, result);
                    }
                }
                Code::Subtract { rd, rn, rm, set_flags } => {
                    let (left, right) = (reg!(rn), reg!(rm));
                    let result = left.wrapping_sub(right);
                    reg!(rd) = result;
                    if set_flags {
                        flags.subtract(left, right, result);
                    }
                }
                Code::SubtractImmediate { rd, rn, set_flags, value } => {
                    let left = reg!(rn);
                    let result = left.wrapping_sub(value);
                    reg!(rd) = result;
                    if set_flags {
                        flags.subtract(left, value, result);
                    }
                }
                Code::Compare { rn, rm } => {
                    let (left, right) = (reg!(rn), reg!(rm));
                    flags.subtract(left, right, left.wrapping_sub(right));
                }
                Code::CompareImmediate { rn, value } => {
                    let left = reg!(rn);
                    flags.subtract(left, value, left.wrapping_sub(value));
                }
                Code::And { rd, rn, rm, set_flags } => {
                    let result = reg!(rn) & reg!(rm);
                    reg!(rd) = result;
                    if set_flags {
                        flags.result(result);
                    }
                }
                Code::Or { rd, rn, rm, set_flags } => {
                    let result = reg!(rn) | reg!(rm);
                    reg!(rd) = result;
                    if set_flags {
                        flags.result(result);
                    }
                }
                Code::Xor { rd, rn, rm, set_flags } => {
                    let result = reg!(rn) ^ reg!(rm);
                    reg!(rd) = result;
                    if set_flags {
                        flags.result(result);
                    }
                }
                Code::BitClear { rd, rn, rm, set_flags } => {
                    let result = reg!(rn) & !reg!(rm);
                    reg!(rd) = result;
                    if set_flags {
                        flags.result(result);
                    }
                }
                Code::Multiply { rd, rn, rm, set_flags } => {
                    let result = reg!(rn).wrapping_mul(reg!(rm));
                    reg!(rd) = result;
                    if set_flags {
                        flags.result(result);
                    }
                }
                Code::Test { rn, rm } => flags.result(reg!(rn) & reg!(rm)),
                Code::LoadWord { rd, rn, offset } => load!(rd, reg!(rn).wrapping_add(offset), Width::Word),
                Code::LoadHalf { rd, rn, offset } => load!(rd, reg!(rn).wrapping_add(offset), Width::Half),
                Code::LoadByte { rd, rn, offset } => load!(rd, reg!(rn).wrapping_add(offset), Width::Byte),
                Code::LoadWordIndexed { rd, rn, rm } => load!(rd, reg!(rn).wrapping_add(reg!(rm)), Width::Word),
                Code::LoadHalfIndexed { rd, rn, rm } => load!(rd, reg!(rn).wrapping_add(reg!(rm)), Width::Half),
                Code::LoadByteIndexed { rd, rn, rm } => load!(rd, reg!(rn).wrapping_add(reg!(rm)), Width::Byte),
                Code::LoadHalfSignedIndexed { rd, rn, rm } => load!(rd, reg!(rn).wrapping_add(reg!(rm)), Width::Half, i16),
                Code::LoadByteSignedIndexed { rd, rn, rm } => load!(rd, reg!(rn).wrapping_add(reg!(rm)), Width::Byte, i8),
                Code::LoadWordAt { rd, address } => load!(rd, address, Width::Word),
                Code::StoreWord { rs, rn, offset } => store!(rs, reg!(rn).wrapping_add(offset), Width::Word),
                Code::StoreHalf { rs, rn, offset } => store!(rs, reg!(rn).wrapping_add(offset), Width::Half),
                Code::StoreByte { rs, rn, offset } => store!(rs, reg!(rn).wrapping_add(offset), Width::Byte),
                Code::StoreWordIndexed { rs, rn, rm } => store!(rs, reg!(rn).wrapping_add(reg!(rm)), Width::Word),
                Code::StoreHalfIndexed { rs, rn, rm } => store!(rs, reg!(rn).wrapping_add(reg!(rm)), Width::Half),
                Code::StoreByteIndexed { rs, rn, rm } => store!(rs, reg!(rn).wrapping_add(reg!(rm)), Width::Byte),
            }
        }
    }

    /// Executes the instruction at `pc` in its general form.
    fn general(&self, op: &Op, pc: u32, regs: &mut [u32; 16], pages: &mut Pages, cpsr: &mut u32, flags: &mut Flags) -> Step {
        let thumb = self.mode & THUMB != 0;
        let pc_mask: u32 = if thumb { !1 } else { !3 };
        // A stored PC reads one instruction further ahead in ARM state.
        let stored_pc = pc.wrapping_add(if thumb { 4 } else { 12 });
        match op {
            Op::Alu {
                op,
                destination,
                left,
                right,
                set_flags,
            } => {
                let left = read(regs, *left);
                let (right, carry) = shift(regs, flags.carry, right);
                let set_flags = *set_flags;
                let result = match op {
                    AluOp::And | AluOp::Xor | AluOp::Or | AluOp::Move | AluOp::BitClear | AluOp::Not => {
                        let result = match op {
                            AluOp::And => left & right,
                            AluOp::Xor => left ^ right,
                            AluOp::Or => left | right,
                            AluOp::Move => right,
                            AluOp::BitClear => left & !right,
                            _ => !right,
                        };
                        if set_flags {
                            flags.result(result);
                            flags.carry = carry;
                        }
                        result
                    }
                    AluOp::Add => {
                        let result = left.wrapping_add(right);
                        if set_flags {
                            flags.add(left, right, result);
                        }
                        result
                    }
                    AluOp::Sub => {
                        let result = left.wrapping_sub(right);
                        if set_flags {
                            flags.subtract(left, right, result);
                        }
                        result
                    }
                    AluOp::ReverseSub => {
                        let result = right.wrapping_sub(left);
                        if set_flags {
                            flags.subtract(right, left, result);
                        }
                        result
                    }
                    AluOp::AddCarry => add_carry(flags, left, right, set_flags),
                    AluOp::SubCarry => add_carry(flags, left, !right, set_flags),
                    AluOp::ReverseSubCarry => add_carry(flags, right, !left, set_flags),
                    AluOp::Multiply | AluOp::CountLeadingZeros => {
                        let result = if *op == AluOp::Multiply {
                            left.wrapping_mul(right)
                        } else {
                            right.leading_zeros()
                        };
                        if set_flags {
                            flags.result(result);
                        }
                        result
                    }
                };
                match destination {
                    Some(15) => return Step::Branch(result & pc_mask),
                    Some(destination) => regs[usize::from(*destination) & 15] = result,
                    None => {}
                }
            }
            Op::Jump { index, link } => {
                if let Some(link) = link {
                    regs[14] = *link;
                }
                return Step::Jump(*index);
            }
            Op::Leave { pc, thumb, link } => {
                if let Some(link) = link {
                    regs[14] = *link;
                }
                *cpsr = if *thumb { *cpsr | THUMB } else { *cpsr & !THUMB };
                return Step::Leave(*pc);
            }
            Op::Branch {
                target,
                exchange: interwork,
                link,
            } => {
                let value = read(regs, *target);
                if let Some(link) = link {
                    regs[14] = *link;
                }
                return Step::Branch(if *interwork { exchange(cpsr, value) } else { value & pc_mask });
            }
            Op::Load {
                destination,
                address,
                width,
                signed,
            } => {
                let (effective, indexed) = address.resolve(regs, flags.carry);
                let Some(value) = load(pages, effective, width) else {
                    return Step::Fault(effective);
                };
                let value = match width {
                    Width::Byte if *signed => value as i8 as u32,
                    Width::Half if *signed => value as i16 as u32,
                    _ => value,
                };
                if *destination != 15 {
                    regs[usize::from(*destination) & 15] = value;
                }
                if let Some(register) = address.write_back {
                    regs[usize::from(register) & 15] = indexed;
                }
                if *destination == 15 {
                    return Step::Branch(exchange(cpsr, value));
                }
            }
            Op::Store { value, address, width } => {
                let (effective, indexed) = address.resolve(regs, flags.carry);
                if store(pages, effective, width, read(regs, *value)).is_none() {
                    return Step::Fault(effective);
                }
                if let Some(register) = address.write_back {
                    regs[usize::from(register) & 15] = indexed;
                }
            }
            Op::MultiplyAccumulate {
                destination,
                left,
                right,
                accumulate,
                set_flags,
            } => {
                let result = read(regs, *left).wrapping_mul(read(regs, *right)).wrapping_add(read(regs, *accumulate));
                regs[usize::from(*destination) & 15] = result;
                if *set_flags {
                    flags.result(result);
                }
            }
            Op::MultiplyLong {
                low,
                high,
                left,
                right,
                signed,
                accumulate,
                set_flags,
            } => {
                let (left, right) = (read(regs, *left), read(regs, *right));
                let mut result = if *signed {
                    (i64::from(left as i32) * i64::from(right as i32)) as u64
                } else {
                    u64::from(left) * u64::from(right)
                };
                let (low, high) = (usize::from(*low) & 15, usize::from(*high) & 15);
                if *accumulate {
                    result = result.wrapping_add(u64::from(regs[low]) | (u64::from(regs[high]) << 32));
                }
                regs[low] = result as u32;
                regs[high] = (result >> 32) as u32;
                if *set_flags {
                    flags.negative = (result >> 32) as u32;
                    flags.nonzero = u32::from(result != 0);
                }
            }
            Op::ReadCpsr { destination } => regs[usize::from(*destination) & 15] = flags.pack(*cpsr),
            Op::WriteCpsr { value, mask, leave } => {
                *cpsr = (flags.pack(*cpsr) & !mask) | (read(regs, *value) & mask);
                *flags = Flags::unpack(*cpsr);
                if let Some(pc) = leave {
                    return Step::Leave(*pc);
                }
            }
            Op::MultipleTransfer {
                base,
                address,
                offset,
                write_back,
                registers,
                load,
            } => {
                let base_value = read(regs, *address);
                let loaded = match transfer(regs, pages, base_value.wrapping_add(*offset), *registers, *load, stored_pc) {
                    Ok(loaded) => loaded,
                    Err(address) => return Step::Fault(address),
                };
                if let Some(adjustment) = write_back {
                    regs[usize::from(*base) & 15] = base_value.wrapping_add(*adjustment);
                }
                if let Some(value) = loaded {
                    return Step::Branch(exchange(cpsr, value));
                }
            }
            Op::DoubleTransfer { registers, address, load } => {
                let (effective, indexed) = address.resolve(regs, flags.carry);
                let loaded = match transfer(regs, pages, effective, *registers, *load, stored_pc) {
                    Ok(loaded) => loaded,
                    Err(address) => return Step::Fault(address),
                };
                if let Some(register) = address.write_back {
                    regs[usize::from(register) & 15] = indexed;
                }
                if let Some(value) = loaded {
                    return Step::Branch(exchange(cpsr, value));
                }
            }
            Op::Swap {
                destination,
                address,
                value,
                width,
            } => {
                let address = read(regs, *address);
                let value = read(regs, *value);
                let Some(loaded) = load(pages, address, width) else {
                    return Step::Fault(address);
                };
                if store(pages, address, width, value).is_none() {
                    return Step::Fault(address);
                }
                regs[usize::from(*destination) & 15] = loaded;
            }
            Op::Nop => {}
        }
        Step::Next
    }
}

/// Picks the dedicated step of a frequent instruction form at `pc`.
fn specialize(op: &Op, pc: u32, thumb: bool) -> Option<Code> {
    Some(match op {
        Op::Jump { index, link: None } => Code::Jump(*index),
        Op::Leave {
            pc: target,
            thumb: target_thumb,
            link,
        } if *target_thumb == thumb => match link {
            None => Code::Exit(*target),
            Some(link) if *link == pc.wrapping_add(4) | u32::from(thumb) => Code::Call(*target),
            Some(_) => return None,
        },
        Op::Branch {
            target: Source::Register(rm),
            exchange: true,
            link: None,
        } => Code::Exchange(*rm),
        Op::Alu {
            op: AluOp::Move,
            destination: Some(rd @ 0..15),
            right: Shifted::Variable(Source::Register(rm), shift, Source::Register(rs)),
            set_flags,
            ..
        } => {
            let (rd, rm, rs, set_flags) = (*rd, *rm, *rs, *set_flags);
            match shift {
                Shift::Lsl => Code::ShiftLeftBy { rd, rm, rs, set_flags },
                Shift::Lsr => Code::ShiftRightBy { rd, rm, rs, set_flags },
                Shift::Asr => Code::ShiftRightSignedBy { rd, rm, rs, set_flags },
                _ => return None,
            }
        }
        Op::Alu {
            op: AluOp::Move,
            destination: Some(rd @ 0..15),
            right: Shifted::Immediate(Source::Register(rm), shift, amount @ 1..32),
            set_flags,
            ..
        } => {
            let (rd, rm, amount, set_flags) = (*rd, *rm, *amount, *set_flags);
            match shift {
                Shift::Lsl => Code::ShiftLeft { rd, rm, amount, set_flags },
                Shift::Lsr => Code::ShiftRight { rd, rm, amount, set_flags },
                Shift::Asr => Code::ShiftRightSigned { rd, rm, amount, set_flags },
                _ => return None,
            }
        }
        Op::Alu {
            op,
            destination,
            left,
            right,
            set_flags,
        } => {
            let set_flags = *set_flags;
            // Arithmetic ignores the shifter carry; the logical forms here must preserve it.
            let arithmetic = matches!(op, AluOp::Add | AluOp::Sub);
            let right = match right {
                Shifted::Register(rm) => Source::Register(*rm),
                Shifted::Constant(value, carry) if arithmetic || carry.is_none() => Source::Immediate(*value),
                _ => return None,
            };
            match (op, destination, left, right) {
                (AluOp::Move, Some(rd @ 0..15), _, Source::Register(rm)) => Code::Move { rd: *rd, rm, set_flags },
                (AluOp::Move, Some(rd @ 0..15), _, Source::Immediate(value)) => Code::MoveImmediate { rd: *rd, set_flags, value },
                (AluOp::Add, Some(rd @ 0..15), Source::Register(rn), Source::Register(rm)) => Code::Add {
                    rd: *rd,
                    rn: *rn,
                    rm,
                    set_flags,
                },
                (AluOp::Add, Some(rd @ 0..15), Source::Register(rn), Source::Immediate(value)) => Code::AddImmediate {
                    rd: *rd,
                    rn: *rn,
                    set_flags,
                    value,
                },
                (AluOp::Sub, Some(rd @ 0..15), Source::Register(rn), Source::Register(rm)) => Code::Subtract {
                    rd: *rd,
                    rn: *rn,
                    rm,
                    set_flags,
                },
                (AluOp::Sub, Some(rd @ 0..15), Source::Register(rn), Source::Immediate(value)) => Code::SubtractImmediate {
                    rd: *rd,
                    rn: *rn,
                    set_flags,
                    value,
                },
                (AluOp::Sub, None, Source::Register(rn), Source::Register(rm)) if set_flags => Code::Compare { rn: *rn, rm },
                (AluOp::Sub, None, Source::Register(rn), Source::Immediate(value)) if set_flags => Code::CompareImmediate { rn: *rn, value },
                (AluOp::And, Some(rd @ 0..15), Source::Register(rn), Source::Register(rm)) => Code::And {
                    rd: *rd,
                    rn: *rn,
                    rm,
                    set_flags,
                },
                (AluOp::Or, Some(rd @ 0..15), Source::Register(rn), Source::Register(rm)) => Code::Or {
                    rd: *rd,
                    rn: *rn,
                    rm,
                    set_flags,
                },
                (AluOp::Xor, Some(rd @ 0..15), Source::Register(rn), Source::Register(rm)) => Code::Xor {
                    rd: *rd,
                    rn: *rn,
                    rm,
                    set_flags,
                },
                (AluOp::BitClear, Some(rd @ 0..15), Source::Register(rn), Source::Register(rm)) => Code::BitClear {
                    rd: *rd,
                    rn: *rn,
                    rm,
                    set_flags,
                },
                (AluOp::Multiply, Some(rd @ 0..15), Source::Register(rn), Source::Register(rm)) => Code::Multiply {
                    rd: *rd,
                    rn: *rn,
                    rm,
                    set_flags,
                },
                (AluOp::And, None, Source::Register(rn), Source::Register(rm)) if set_flags => Code::Test { rn: *rn, rm },
                _ => return None,
            }
        }
        Op::Load {
            destination: rd @ 0..15,
            address:
                Address {
                    base,
                    offset,
                    subtract,
                    pre_index: true,
                    write_back: None,
                },
            width,
            signed: false,
        } => {
            let rd = *rd;
            match (base, offset, width) {
                (Source::Immediate(base), Shifted::Constant(offset, _), Width::Word) => Code::LoadWordAt {
                    rd,
                    address: if *subtract {
                        base.wrapping_sub(*offset)
                    } else {
                        base.wrapping_add(*offset)
                    },
                },
                (Source::Register(rn), Shifted::Constant(offset, _), _) => {
                    let (rn, offset) = (*rn, if *subtract { offset.wrapping_neg() } else { *offset });
                    match width {
                        Width::Word => Code::LoadWord { rd, rn, offset },
                        Width::Half => Code::LoadHalf { rd, rn, offset },
                        Width::Byte => Code::LoadByte { rd, rn, offset },
                    }
                }
                (Source::Register(rn), Shifted::Register(rm), _) if !subtract => {
                    let (rn, rm) = (*rn, *rm);
                    match width {
                        Width::Word => Code::LoadWordIndexed { rd, rn, rm },
                        Width::Half => Code::LoadHalfIndexed { rd, rn, rm },
                        Width::Byte => Code::LoadByteIndexed { rd, rn, rm },
                    }
                }
                _ => return None,
            }
        }
        Op::Load {
            destination: rd @ 0..15,
            address:
                Address {
                    base: Source::Register(rn),
                    offset: Shifted::Register(rm),
                    subtract: false,
                    pre_index: true,
                    write_back: None,
                },
            width,
            signed: true,
        } => {
            let (rd, rn, rm) = (*rd, *rn, *rm);
            match width {
                Width::Half => Code::LoadHalfSignedIndexed { rd, rn, rm },
                Width::Byte => Code::LoadByteSignedIndexed { rd, rn, rm },
                Width::Word => return None,
            }
        }
        Op::Store {
            value: Source::Register(rs),
            address:
                Address {
                    base: Source::Register(rn),
                    offset,
                    subtract,
                    pre_index: true,
                    write_back: None,
                },
            width,
        } => {
            let (rs, rn) = (*rs, *rn);
            match offset {
                Shifted::Constant(offset, _) => {
                    let offset = if *subtract { offset.wrapping_neg() } else { *offset };
                    match width {
                        Width::Word => Code::StoreWord { rs, rn, offset },
                        Width::Half => Code::StoreHalf { rs, rn, offset },
                        Width::Byte => Code::StoreByte { rs, rn, offset },
                    }
                }
                Shifted::Register(rm) if !subtract => {
                    let rm = *rm;
                    match width {
                        Width::Word => Code::StoreWordIndexed { rs, rn, rm },
                        Width::Half => Code::StoreHalfIndexed { rs, rn, rm },
                        Width::Byte => Code::StoreByteIndexed { rs, rn, rm },
                    }
                }
                _ => return None,
            }
        }
        _ => return None,
    })
}

struct Translator {
    thumb: bool,
    pc: u32,
}

impl Translator {
    fn register(register: &Reg) -> Result<u8> {
        if register.index() > 15 {
            return Err(WieError::FatalError(format!(
                "Portable ARM register {} is out of range",
                register.index()
            )));
        }
        Ok(register.index())
    }

    /// Instructions read their own address two instructions ahead.
    fn register_source(&self, register: &Reg) -> Result<Source> {
        Ok(if *register == Reg::PC {
            Source::Immediate(self.pc.wrapping_add(if self.thumb { 4 } else { 8 }))
        } else {
            Source::Register(Self::register(register)?)
        })
    }

    fn source(&self, value: &Value) -> Result<Source> {
        match value {
            Value::Register(register) => self.register_source(register),
            Value::Immediate(value) => Ok(Source::Immediate(*value)),
        }
    }

    fn shifted(&self, operand: Operand) -> Result<Shifted> {
        if let Some((value, carry)) = operand.constant_value() {
            return Ok(Shifted::Constant(value, carry.map(|carry| carry != 0)));
        }
        let value = self.source(&operand.value)?;
        Ok(match (operand.shift, operand.amount) {
            (Shift::Rrx, _) => Shifted::Immediate(value, Shift::Rrx, 1),
            (_, ShiftAmount::Immediate(0)) => match value {
                Source::Register(register) => Shifted::Register(register),
                Source::Immediate(value) => Shifted::Constant(value, None),
            },
            (shift, ShiftAmount::Immediate(amount)) => Shifted::Immediate(value, shift, amount),
            (shift, ShiftAmount::Register(amount)) => Shifted::Variable(value, shift, self.register_source(&amount)?),
        })
    }

    fn address(&self, address: MemoryOperand) -> Result<Address> {
        Ok(Address {
            base: self.source(&address.base)?,
            offset: self.shifted(address.offset)?,
            subtract: address.subtract,
            pre_index: address.pre_index,
            write_back: address.write_back.as_ref().map(Self::register).transpose()?,
        })
    }

    fn operation(&self, operation: Operation, fallthrough: u32, locate: impl Fn(u32) -> Option<usize>) -> Result<Op> {
        Ok(match operation {
            Operation::Alu {
                op,
                destination,
                left,
                right,
                set_flags,
            } => Op::Alu {
                op,
                destination: destination.as_ref().map(Self::register).transpose()?,
                left: self.source(&left)?,
                right: self.shifted(right)?,
                set_flags,
            },
            Operation::Branch {
                target: BranchTarget::Address(address),
                link,
                exchange,
            } => {
                let link = link.map(|link| link.get());
                let thumb = if exchange { address.get() & 1 != 0 } else { self.thumb };
                let pc = address.get() & if thumb { !1 } else { !3 };
                match locate(pc).filter(|_| thumb == self.thumb) {
                    Some(index) => Op::Jump { index: index as u16, link },
                    None => Op::Leave { pc, thumb, link },
                }
            }
            Operation::Branch {
                target: BranchTarget::Register(register),
                link,
                exchange,
            } => Op::Branch {
                target: self.register_source(&register)?,
                exchange,
                link: link.map(|link| link.get()),
            },
            Operation::Load {
                destination,
                address,
                width,
                signed,
            } => Op::Load {
                destination: Self::register(&destination)?,
                address: self.address(address)?,
                width,
                signed,
            },
            Operation::Store { value, address, width } => Op::Store {
                // A stored PC reads one instruction further ahead in ARM state.
                value: if value == Value::Register(Reg::PC) {
                    Source::Immediate(self.pc.wrapping_add(if self.thumb { 4 } else { 12 }))
                } else {
                    self.source(&value)?
                },
                address: self.address(address)?,
                width,
            },
            Operation::MultiplyAccumulate {
                destination,
                left,
                right,
                accumulate,
                set_flags,
            } => Op::MultiplyAccumulate {
                destination: Self::register(&destination)?,
                left: self.register_source(&left)?,
                right: self.register_source(&right)?,
                accumulate: self.register_source(&accumulate)?,
                set_flags,
            },
            Operation::MultiplyLong {
                low,
                high,
                left,
                right,
                signed,
                accumulate,
                set_flags,
            } => Op::MultiplyLong {
                low: Self::register(&low)?,
                high: Self::register(&high)?,
                left: self.register_source(&left)?,
                right: self.register_source(&right)?,
                signed,
                accumulate,
                set_flags,
            },
            Operation::ReadCpsr { destination } => Op::ReadCpsr {
                destination: Self::register(&destination)?,
            },
            Operation::WriteCpsr { value, mask } => Op::WriteCpsr {
                value: self.source(&value)?,
                mask,
                leave: (mask & MODE_MASK != 0).then_some(fallthrough),
            },
            Operation::MultipleTransfer {
                base,
                registers,
                increment,
                before,
                write_back,
                load,
            } => {
                let bytes = registers.count_ones() * 4;
                Op::MultipleTransfer {
                    base: Self::register(&base)?,
                    address: self.register_source(&base)?,
                    offset: if increment {
                        u32::from(before) * 4
                    } else {
                        (u32::from(!before) * 4).wrapping_sub(bytes)
                    },
                    write_back: write_back.then_some(if increment { bytes } else { bytes.wrapping_neg() }),
                    registers,
                    load,
                }
            }
            Operation::DoubleTransfer { register, address, load } => {
                let register = Self::register(&register)?;
                if register == 15 {
                    return Err(WieError::FatalError("Portable ARM register pair is out of range".into()));
                }
                Op::DoubleTransfer {
                    registers: 3 << register,
                    address: self.address(address)?,
                    load,
                }
            }
            Operation::Swap {
                destination,
                address,
                value,
                width,
            } => Op::Swap {
                destination: Self::register(&destination)?,
                address: self.register_source(&address)?,
                value: self.register_source(&value)?,
                width,
            },
            Operation::Nop => Op::Nop,
        })
    }
}

fn compile(ir: RegionIr) -> Result<Option<(Region, ManifestRegion)>> {
    let thumb = ir.entry.thumb;
    let shift = if thumb { 1 } else { 2 };
    // Addresses below the first page always return to the dispatcher.
    let mut instructions: Vec<Instruction> = ir
        .blocks
        .into_iter()
        .flat_map(|block| block.instructions)
        .filter(|instruction| instruction.pc.get() >= 0x1000)
        .collect();
    instructions.sort_unstable_by_key(|instruction| instruction.pc.get());
    instructions.dedup_by_key(|instruction| instruction.pc.get());
    let (Some(first), Some(last)) = (instructions.first(), instructions.last()) else {
        return Ok(None);
    };
    let base = first.pc.get();
    let mut entries = vec![u16::MAX; ((last.pc.get() - base) >> shift) as usize + 1];
    let mut length = 0;
    let mut instruction_pcs = Vec::with_capacity(instructions.len());
    let mut code_ranges: Vec<Range<u64>> = Vec::new();
    for (index, instruction) in instructions.iter().enumerate() {
        let pc = instruction.pc.get();
        let fallthrough = pc.wrapping_add(u32::from(instruction.size));
        if (pc - base) & ((1 << shift) - 1) != 0 || length >= usize::from(u16::MAX) - 2 {
            return Err(WieError::FatalError(format!("Portable ARM region cannot index {pc:#x}")));
        }
        entries[((pc - base) >> shift) as usize] = length as u16;
        // A condition step, the instruction, and an exit unless the next instruction follows.
        length += usize::from(instruction.condition != Condition::Always)
            + 1
            + usize::from(instructions.get(index + 1).is_none_or(|next| next.pc.get() != fallthrough));
        instruction_pcs.push(pc);
        let (start, end) = (u64::from(pc), u64::from(pc) + u64::from(instruction.size));
        if let Some(previous) = code_ranges.last_mut()
            && start <= previous.end
        {
            previous.end = previous.end.max(end);
        } else {
            code_ranges.push(start..end);
        }
    }
    let mut code = Vec::with_capacity(length);
    let mut pcs = Vec::with_capacity(length);
    let mut general = Vec::new();
    let mut instructions = instructions.into_iter().peekable();
    while let Some(instruction) = instructions.next() {
        let pc = instruction.pc.get();
        let fallthrough = pc.wrapping_add(u32::from(instruction.size));
        if instruction.condition != Condition::Always {
            code.push(Code::Skip(instruction.condition as u8));
            pcs.push(pc);
        }
        let op = Translator { thumb, pc }.operation(instruction.operation, fallthrough, |pc| lookup(&entries, base, shift, pc))?;
        code.push(specialize(&op, pc, thumb).unwrap_or_else(|| {
            general.push(op);
            Code::General(general.len() as u32 - 1)
        }));
        pcs.push(pc);
        if instructions.peek().is_none_or(|next| next.pc.get() != fallthrough) {
            code.push(Code::End(fallthrough));
            pcs.push(fallthrough);
        }
    }
    Ok(Some((
        Region {
            mode: u32::from(ir.entry.cpu_mode) | if thumb { THUMB } else { 0 },
            base,
            shift,
            entries,
            code,
            pcs,
            general,
        },
        ManifestRegion {
            entry: ir.entry,
            instruction_pcs,
            code_ranges,
        },
    )))
}

#[derive(Default)]
struct Slots {
    regions: Vec<Option<Region>>,
    released: Vec<u32>,
}

/// Executes prepared regions by interpreting their decoded form, for hosts that cannot generate code.
#[derive(Default)]
pub struct PortableExecutor {
    slots: Arc<Mutex<Slots>>,
}

impl PortableExecutor {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CompiledExecutor for PortableExecutor {
    fn prepare(&mut self, request: CompileRequest) -> PreparationFuture {
        let slots = Arc::downgrade(&self.slots);
        Box::pin(async move {
            let mut compiled = Vec::new();
            for region in request.coalesced_regions().flatten() {
                compiled.extend(compile(region.ir)?);
            }
            let slots = slots
                .upgrade()
                .ok_or_else(|| WieError::FatalError("Portable ARM executor was dropped".into()))?;
            let mut slots = slots.lock();
            tracing::debug!(
                regions = compiled.len(),
                steps = compiled.iter().map(|(region, _)| region.code.len()).sum::<usize>(),
                general = compiled.iter().map(|(region, _)| region.general.len()).sum::<usize>(),
                "Portable ARM regions prepared"
            );
            Ok(CompiledArtifact {
                regions: compiled
                    .into_iter()
                    .map(|(region, manifest)| {
                        let module = slots.released.pop().unwrap_or_else(|| {
                            slots.regions.push(None);
                            slots.regions.len() as u32 - 1
                        });
                        slots.regions[module as usize] = Some(region);
                        CompiledRegion {
                            manifest,
                            handle: CompiledHandle { module, slot: 0 },
                        }
                    })
                    .collect(),
            })
        })
    }

    fn release(&mut self, handle: CompiledHandle) {
        let mut slots = self.slots.lock();
        if let Some(region) = slots.regions.get_mut(handle.module as usize) {
            *region = None;
            slots.released.push(handle.module);
        }
    }

    fn execute(&mut self, mut handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit> {
        let slots = self.slots.lock();
        loop {
            // A handle resolved for the frame's PC and mode always has an entry there.
            let exit = slots
                .regions
                .get(handle.module as usize)
                .and_then(Option::as_ref)
                .and_then(|region| region.run(frame, access.pages()))
                .ok_or_else(|| WieError::FatalError("Portable ARM region is unavailable".into()))?;
            if exit == CompiledExit::GuestFault {
                return Ok(exit);
            }
            let pc = frame.regs[15];
            if pc < 0x1000 || pc == frame.end || frame.budget == 0 {
                return Ok(CompiledExit::Dispatch);
            }
            let Some(next) = access.resolve(pc, frame.cpsr) else {
                return Ok(CompiledExit::Dispatch);
            };
            handle = next;
        }
    }
}
