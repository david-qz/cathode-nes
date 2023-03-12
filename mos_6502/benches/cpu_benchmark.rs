use criterion::{criterion_group, criterion_main, Criterion};
use mos_6502::{cpu::CPU, memory::Bus16, memory::FlatMemory};

fn klaus_functional_test_no_decimal_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu");
    group.sample_size(20);

    let bin = std::fs::read("test_programs/bin/6502_functional_test_no_decimal.bin")
        .expect("Failed to load test code.");

    let mut memory = FlatMemory::new();

    group.bench_function("klaus_function_test_no_decimal", |b| {
        b.iter(|| {
            memory.load_code(&bin, 0, Some(0x400));
            let mut cpu = CPU::new();
            while cpu.pc != 0x336D {
                cpu.execute_instruction(&mut memory);
            }
        })
    });
    group.finish();
}

criterion_group!(benches, klaus_functional_test_no_decimal_bench);
criterion_main!(benches);
