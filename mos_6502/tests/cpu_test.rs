use mos_6502::{cpu::CPU, memory::Bus16, memory::FlatMemory};

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

    let mut last_pc = None;
    loop {
        cpu.execute_instruction(&mut memory);
        let current_pc = Some(cpu.pc);

        if last_pc == current_pc {
            break;
        }
        last_pc = Some(cpu.pc);
    }

    match last_pc {
        Some(0x336D) => (),
        Some(pc) => panic!(
            "CPU trapped at PC={:X} in test={}",
            pc,
            memory.read_byte(0x200)
        ),
        None => panic!("The CPU didn't run."),
    };

    // NOTE: This cycle count may or may not be correct. This assertion mainly exists to guard against accidentally
    //       regressing the emulator's timings. It may need to be updated if accuracy is improved.
    let expected_cycles = 84_030_434;
    assert_eq!(
        cpu.total_cycles, expected_cycles,
        "CPU completed test in unexpected number of cycles."
    );
}
