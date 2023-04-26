# Cathode

Cathode is a WIP NES emulator. The project currently includes the following components:

- `/mos_6502`: A MOS-6502 CPU emulator. Supports all legal and illegal opcodes but lacks decimal mode arithmetic (like the CPU core in the actual NES). Includes a comprehensive test suite.
- `/nes`: An extremely WIP NES emulator.
- `/nes_sdl`: An SDL2 binary target.

## TODO

- [x] CPU emulation
- [x] INES file format support
- [x] NROM mapper support
- [ ] PPU emulation (WIP)
- [ ] Joypad support (WIP)
- [ ] More mappers
- [ ] WASM target
