use mos_6502::{cpu::CPU, debugging::Debugger, memory::Bus16, memory::FlatMemory};
use std::{cell::RefCell, rc::Rc};

#[test]
fn two_plus_two() {
    let program = vec![
        0xA9, 0x02, // LDA #2
        0x69, 0x02, // ADC #2
        0x8D, 0x00, 0x02, // STA $0200
    ];

    let mut memory = FlatMemory::new();
    memory.load_code(&program, 0, Some(0));

    let mut cpu = CPU::new();
    cpu.reset(&mut memory);
    for _ in 0..3 {
        cpu.execute_instruction(&mut memory);
    }

    assert_eq!(memory.read_byte(0x0200), 4);
}

#[test]
fn klaus_functional_test_no_decimal() {
    let bin = std::fs::read("test_programs/bin/6502_functional_test_no_decimal.bin")
        .expect("Failed to load test code.");

    let mut memory = FlatMemory::new();
    memory.load_code(&bin, 0, Some(0x400));

    let mut cpu = CPU::new();
    cpu.reset(&mut memory);

    let debugger = Rc::new(RefCell::new(Debugger::new()));
    cpu.attach_debugger(Rc::clone(&debugger));

    let mut last_pc = cpu.pc;
    loop {
        cpu.execute_instruction(&mut memory);
        let current_pc = cpu.pc;

        if last_pc == current_pc {
            break;
        }
        last_pc = cpu.pc;
    }

    if last_pc != 0x336D {
        debugger.borrow().dump_backtrace();
        panic!(
            "CPU trapped at PC={:X} in test={}",
            last_pc,
            memory.read_byte(0x200)
        );
    }

    // NOTE: This cycle count may or may not be correct. This assertion mainly exists to guard against accidentally
    //       regressing the emulator's timings. It may need to be updated if accuracy is improved.
    let expected_cycles = 84_030_458;
    assert_eq!(
        cpu.total_cycles, expected_cycles,
        "CPU completed test in unexpected number of cycles."
    );
}
