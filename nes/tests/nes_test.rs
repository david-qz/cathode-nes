use mos_6502::{debugging::ExecutionState, disassembly::Instruction};
use nes::{cartridge::Cartridge, nes::NES};

#[test]
fn nes_test() {
    let golden_path = load_golden_log();
    let mut ticks = 0;

    let bytes = std::fs::read("test-roms/nestest/nestest.nes").unwrap();
    let cartridge = <dyn Cartridge>::load(bytes).unwrap();
    let mut nes = NES::new();
    nes.insert_cartridge(cartridge);
    nes.set_pc(0xC000);
    nes.enable_debugger();

    while !nes.jammed() {
        if ticks < golden_path.len() && golden_path[ticks] != nes.current_state() {
            nes.dump_backtrace();
            assert_eq!(golden_path[ticks], nes.current_state())
        }

        nes.tick();
        ticks += 1;
    }
}

fn load_golden_log() -> Vec<ExecutionState> {
    let log = std::fs::read_to_string("test-roms/nestest/golden_log.txt").unwrap();

    let mut expected_states = Vec::new();
    for line in log.lines() {
        expected_states.push(parse_log_line(line))
    }

    expected_states
}

fn parse_log_line(line: &str) -> ExecutionState {
    let pc = u16::from_str_radix(&line[0..4], 16).unwrap();
    let opcode = u8::from_str_radix(&line[6..8], 16).unwrap();
    let operand1 = u8::from_str_radix(&line[9..11], 16).unwrap_or(0);
    let operand2 = u8::from_str_radix(&line[12..14], 16).unwrap_or(0);

    let a = u8::from_str_radix(&line[50..52], 16).unwrap();
    let x = u8::from_str_radix(&line[55..57], 16).unwrap();
    let y = u8::from_str_radix(&line[60..62], 16).unwrap();
    let p = u8::from_str_radix(&line[65..67], 16).unwrap();
    let s = u8::from_str_radix(&line[71..73], 16).unwrap();
    let cycle_number = u64::from_str_radix(&line[90..], 10).unwrap();

    ExecutionState {
        next_instruction: Instruction::new(opcode, operand1, operand2),
        a,
        x,
        y,
        p,
        s,
        pc,
        cycle_number,
    }
}
