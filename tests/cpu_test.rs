use untitled_emulator::{bus::Bus16, cpu::CPU, flat_memory::FlatMemory};

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
