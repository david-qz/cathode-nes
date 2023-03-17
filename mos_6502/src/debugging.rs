use crate::{cpu::CPU, disassembly::Instruction, memory::Bus16};
use std::collections::VecDeque;

pub struct Debugger {
    states: VecDeque<ExecutionState>,
    backtrace_limit: usize,
}

impl Debugger {
    const DEFAULT_BACKTRACE_LIMIT: usize = 20;

    pub fn new(backtrace_limit: Option<usize>) -> Self {
        let limit = backtrace_limit.unwrap_or(Self::DEFAULT_BACKTRACE_LIMIT);

        Self {
            states: VecDeque::with_capacity(limit),
            backtrace_limit: limit,
        }
    }

    pub fn record_state(&mut self, cpu: &CPU, bus: &dyn Bus16) {
        if self.states.len() == self.backtrace_limit {
            self.states.pop_front();
        }
        self.states.push_back(ExecutionState::new(cpu, bus));
    }

    pub fn dump_backtrace(&self) {
        for state in &self.states {
            println!("{}", state);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutionState {
    pub next_instruction: Instruction,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub s: u8,
    pub pc: u16,
    pub cycle_number: u64,
}

impl ExecutionState {
    pub fn new(cpu: &CPU, bus: &dyn Bus16) -> Self {
        let opcode = bus.read_byte(cpu.pc);
        let operand1 = bus.read_byte(cpu.pc + 1);
        let operand2 = bus.read_byte(cpu.pc + 2);
        let next_instruction = Instruction::new(opcode, operand1, operand2);

        Self {
            next_instruction,
            a: cpu.a,
            x: cpu.x,
            y: cpu.y,
            p: cpu.status_register(),
            s: cpu.s,
            pc: cpu.pc,
            cycle_number: cpu.total_cycles,
        }
    }
}

impl std::fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04X}  {:<40}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            self.pc,
            self.next_instruction,
            self.a,
            self.x,
            self.y,
            self.p,
            self.s,
            self.cycle_number
        )
    }
}
