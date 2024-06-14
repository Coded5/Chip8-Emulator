mod chip8;
mod device;

///CHIP-8 Spec
///16x 8bit register -> index by V0 (0 is HEX)
///4096 B of Memory
/// Sections:
///     0x000-0x1FF => CHIP-8 Interpreter
///     0x050-0x0A0 => Storage for 0-F characters
///     0x200-0xFFF => Instruction from ROM, free to use 
///
///16bit Index Register: store memory address for operations
///16bit Program Counter: keeping tracking of execution index
///16-level Stack: keep track of order of execution E.X. CALL and RET
///8bit Stack Pointer
///8bit Delay Timer
///8bit Sound Timer
///16 Input keys
///64x32 Monochrome display memory
///

use std::{env, time::SystemTime};
use device::Device;
use chip8::Chip8;
use piston::{Button, EventSettings, Events, Key, PressEvent, ReleaseEvent, RenderEvent};

const CYCLE_DELAY: u128 = 1;

#[allow(unreachable_code)]
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let mut chip8 = Chip8::create();
    chip8.load_rom("./rom/Pong.ch8");
    //chip8.temp(); 
    //

    //return;

    let mut device = Device::start();   

    let mut events = Events::new(EventSettings::new());

    let mut last_time = SystemTime::now();

    while let Some(e) = events.next(&mut device.window) {
        let current_time = SystemTime::now();
        let dt = current_time.duration_since(last_time).unwrap().as_millis();
        
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::D1 => chip8.keypad[0x1] = true,
                Key::D2 => chip8.keypad[0x2] = true,
                Key::D3 => chip8.keypad[0x3] = true,
                Key::D4 => chip8.keypad[0xC] = true,
                Key::Q  => chip8.keypad[0x4] = true,
                Key::W  => chip8.keypad[0x5] = true,
                Key::E  => chip8.keypad[0x6] = true,
                Key::R  => chip8.keypad[0xD] = true,
                Key::A  => chip8.keypad[0x7] = true,
                Key::S  => chip8.keypad[0x8] = true,
                Key::D  => chip8.keypad[0x9] = true,
                Key::F  => chip8.keypad[0xE] = true,
                Key::Z  => chip8.keypad[0xA] = true,
                Key::X  => chip8.keypad[0x9] = true,
                Key::C  => chip8.keypad[0xB] = true,
                Key::V  => chip8.keypad[0xF] = true,
                Key::K => {
                    for y in 0..32 {
                        for x in 0..64 {
                            print!("{} ", chip8.video[x + y * 64]);
                        }
                        println!();
                    }
                },
                _ => ()
            }
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            match key {
                Key::D1 => chip8.keypad[0x1] = false,
                Key::D2 => chip8.keypad[0x2] = false,
                Key::D3 => chip8.keypad[0x3] = false,
                Key::D4 => chip8.keypad[0xC] = false,
                Key::Q  => chip8.keypad[0x4] = false,
                Key::W  => chip8.keypad[0x5] = false,
                Key::E  => chip8.keypad[0x6] = false,
                Key::R  => chip8.keypad[0xD] = false,
                Key::A  => chip8.keypad[0x7] = false,
                Key::S  => chip8.keypad[0x8] = false,
                Key::D  => chip8.keypad[0x9] = false,
                Key::F  => chip8.keypad[0xE] = false,
                Key::Z  => chip8.keypad[0xA] = false,
                Key::X  => chip8.keypad[0x9] = false,
                Key::C  => chip8.keypad[0xB] = false,
                Key::V  => chip8.keypad[0xF] = false,
                _ => ()
            }
        } 

        if dt > CYCLE_DELAY {
            last_time = current_time;

            chip8.run();
            
        }

        if let Some(args) = e.render_args() {
            device.render(&args, chip8.video);
        }
    }
}
