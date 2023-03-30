mod chip8;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::env;
use std::time::{Duration, Instant};

const WIDTH: u32 = 64 * 10;
const HEIGHT: u32 = 32 * 10;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Chip-8 Emulator", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = chip8::State::new();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: chip8_emulator <path_to_rom>");
        return;
    }
    if let Err(e) = chip8.load_rom(&args[1]) {
        println!("Failed to load ROM: {}", e);
        return;
    }

    let mut running: bool = true;

    while running {
        let start_time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    println!("Escape pressed...");
                    running = false;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(chip8_key) = map_keycode_to_chip8_key(keycode) {
                        println!("Key DOWN: {}", chip8_key);
                        chip8.key_down(chip8_key);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(chip8_key) = map_keycode_to_chip8_key(keycode) {
                        println!("Key UP: {}", chip8_key);
                        chip8.key_up(chip8_key);
                    }
                }
                _ => {}
            }
        }

        chip8.emulate_cycle();
        chip8.update_timers();
        draw_display(&chip8, &mut canvas);

        let delay_per_instruction = 500;

        let elapsed = start_time.elapsed();
        if elapsed < Duration::from_micros(delay_per_instruction) {
            std::thread::sleep(Duration::from_micros(delay_per_instruction) - elapsed);
        }
    }
}

fn map_keycode_to_chip8_key(keycode: Keycode) -> Option<u8> {
    match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

fn draw_display(chip8: &chip8::State, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));

    for y in 0..32 {
        for x in 0..64 {
            let index = y * 64 + x;
            if chip8.get_display()[index] {
                let _ = canvas.fill_rect(Rect::new(
                    (x as u32 * 10) as i32,
                    (y as u32 * 10) as i32,
                    10,
                    10,
                ));
            }
        }
    }

    canvas.present();
}
