extern crate sdl2;

use nes::cartridge::Cartridge;
use nes::frame::Frame;
use nes::input::StandardController;
use nes::nes::NES;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Texture, TextureAccess, UpdateTextureError};
use sdl2::render::{TextureCreator, TextureValueError};
use sdl2::video::{Window, WindowBuildError, WindowContext};
use sdl2::VideoSubsystem;
use std::env;
use std::error;
use std::time::{Duration, Instant};

pub fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = match args.get(1) {
        Some(x) => x,
        None => panic!("Path to rom not provided"),
    };

    let rom_file_bytes = std::fs::read(file_path)?;
    let cartridge = <dyn Cartridge>::load(rom_file_bytes).unwrap();

    let mut nes = NES::new();
    nes.insert_cartridge(cartridge);

    let mut controller: StandardController = Default::default();

    let sdl_ctx = sdl2::init()?;

    let mut canvas = {
        let video_subsystem = sdl_ctx.video()?;
        let window = create_window(&video_subsystem)?;
        window.into_canvas().build()?
    };

    let texture_creator = canvas.texture_creator();
    let mut texture = create_texture(&texture_creator)?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_ctx.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => update_controller(&mut controller, keycode, true),
                Event::KeyUp {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => update_controller(&mut controller, keycode, false),
                _ => {}
            }
        }

        if !nes.jammed() {
            let start = Instant::now();

            nes.advance_to_next_frame();
            nes.update_controller_port_a(&controller);
            let frame = nes.borrow_frame();

            copy_frame_to_texture(&mut texture, frame)?;
            canvas.copy(&texture, None, None)?;
            canvas.present();

            println!("frame time: {:?}", start.elapsed());
        }

        // TODO: Create a more precise timing mechanism. This doesn't take into account time spent executing.
        std::thread::sleep(Duration::from_millis(16));
    }

    return Ok(());
}

fn create_window(video_subsystem: &VideoSubsystem) -> Result<Window, WindowBuildError> {
    let width = 2 * Frame::WIDTH as u32;
    let height = 2 * Frame::HEIGHT as u32;

    video_subsystem
        .window("cathode", width, height)
        .position_centered()
        .build()
}

fn create_texture(creator: &TextureCreator<WindowContext>) -> Result<Texture, TextureValueError> {
    let width = Frame::WIDTH as u32;
    let height = Frame::HEIGHT as u32;

    creator.create_texture(
        PixelFormatEnum::RGB24,
        TextureAccess::Streaming,
        width,
        height,
    )
}

fn copy_frame_to_texture(texture: &mut Texture, frame: &Frame) -> Result<(), UpdateTextureError> {
    let pitch = Frame::WIDTH * Frame::BYTES_PER_PIXEL;
    texture.update(None, frame.data_rgb8(), pitch)
}

fn update_controller(controller: &mut StandardController, keycode: Keycode, pressed: bool) {
    match keycode {
        Keycode::Up => controller.up = pressed,
        Keycode::Down => controller.down = pressed,
        Keycode::Left => controller.left = pressed,
        Keycode::Right => controller.right = pressed,
        Keycode::Q => controller.select = pressed,
        Keycode::W => controller.start = pressed,
        Keycode::A => controller.a = pressed,
        Keycode::S => controller.b = pressed,
        _ => {}
    }
}
