extern crate sdl2;

use chip8_rs::*;

use clap::Parser;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;
use sdl2::{event::Event, render::Canvas};
use std::collections::HashMap;
use std::{
    fs::File,
    io::{BufReader, Read},
};

const SCALE: u32 = 16;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH_C8 as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT_C8 as u32) * SCALE;

const TICK_PER_FRAME: usize = 10;

#[derive(Clone)]
struct Userkey {
	keys: HashMap<Keycode, usize>,
}

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

fn draw_screen(chip8: &Chip8, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    let screen_buf = chip8.get_screen();

    canvas.set_draw_color(Color::WHITE);
    for (i, &pixel_on) in screen_buf.iter().enumerate() {
        if pixel_on {
            let x = (i % SCREEN_WIDTH_C8) as u32;
            let y = (i / SCREEN_WIDTH_C8) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }

    canvas.present();
}

fn userkey_to_chip8(userkeys: &Userkey, key: Keycode) -> Option<usize> {
	match userkeys.keys.get(&key) {
		Some(&c8key) => Some(c8key),
		None => None,
	}
	
}

fn main() {
    let args = Cli::parse();

    let file = match File::open(args.path.clone()) {
        Err(_) => panic!("[Error] Couldn't open ROM '{}'", args.path.display()),
        Ok(file) => file,
    };

    let mut reader = BufReader::new(file);
    let mut data: Vec<u8> = Vec::new();

    if reader.read_to_end(&mut data).is_err() {
        panic!("[Error] Couldn't read from file '{}'", args.path.display())
    }

    let mut chip8 = Chip8::new();
	let userkeys = Userkey {
		keys: HashMap::from([
			(Keycode::Num1, 1), 	(Keycode::Num2, 2), 	(Keycode::Num3, 3), 	(Keycode::Num4, C),
			(Keycode::A, 4), 		(Keycode::Z, 5),		(Keycode::E, 6),		(Keycode::R, D),
			(Keycode::Q, 7),		(Keycode::S, 8),		(Keycode::D, 9),		(Keycode::F, E),
			(Keycode::W, A),		(Keycode::X, 0),		(Keycode::C, B),		(Keycode::V, F),
		]),
	};

    chip8.load_rom(&data);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CXIP8 - A Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'gameloop: loop {
        for _tick in 0..TICK_PER_FRAME {
            chip8.cpu_tick();
        }

		chip8.timers_tick();

		draw_screen(&chip8, &mut canvas);

        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. } | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'gameloop;
                },
				Event::KeyDown {keycode: Some(key), ..} => {
					if let Some(c8key) = userkey_to_chip8(&userkeys, key) {
						chip8.keypress(c8key, true);
					}
				}
				Event::KeyUp {keycode: Some(key), ..} => {
					if let Some(c8key) = userkey_to_chip8(&userkeys, key) {
						chip8.keypress(c8key, false);
					}
				}
                _ => (),
            }
        }
    }
}
