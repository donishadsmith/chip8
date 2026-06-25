/*Great references:
 -https://github.com/ablakey/chip8/blob/master/src/chip8.rs
 -https://github.com/starrhorne/chip8-rust/
 -https://github.com/tendstofortytwo/chip8-rust/
*/

mod components;
mod fontset;
mod utils;
mod vm;

use macroquad::{audio::load_sound_from_bytes, prelude::*};
use rfd::FileDialog;
use std::{
    path::PathBuf,
    process,
    time::{Duration, Instant},
};

use crate::{
    components::{audio::Audio, cartridge::Cartridge},
    utils::get_key,
    vm::VirtualMachine,
};

#[derive(PartialEq)]
enum EmulatorState {
    Start,
    Quit,
    Active,
}

fn file_dialog() -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select a CHIP-8 ROM file")
        .set_directory("/")
        .pick_file()
}

fn change_emulator_state(emulator_state: &mut EmulatorState, state: EmulatorState) {
    *emulator_state = state;
}

fn quit_emulator(emulator_state: &mut EmulatorState, default_state: EmulatorState) {
    if let Some(key) = get_key()
        && key == KeyCode::Escape
    {
        change_emulator_state(emulator_state, EmulatorState::Quit);
    } else {
        change_emulator_state(emulator_state, default_state);
    }
}

fn draw_highlight(choices: [&str; 2], cursor: &mut usize) {
    if is_key_pressed(KeyCode::Down) | is_key_pressed(KeyCode::S) {
        *cursor += 1;
        *cursor = cursor.rem_euclid(choices.len());
    }

    if is_key_pressed(KeyCode::Up) | is_key_pressed(KeyCode::W) {
        *cursor = (*cursor + choices.len() - 1) % choices.len();
    }

    for (index, text) in choices.iter().enumerate() {
        draw_text(
            text,
            screen_width() / 2.0 * 0.85,
            screen_height() / 2.0 * 0.80 + index as f32 * 50.0,
            40.0,
            if index == *cursor { YELLOW } else { WHITE },
        );
    }

    draw_text(
        "Up (W/Up Arrow) | Down (S/Down Arrow) | Quit (Escape)",
        screen_width() / 2.0 * 0.30,
        screen_height() / 2.0 + choices.len() as f32 * 50.0,
        24.0,
        WHITE,
    );
}

pub fn error_message(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}

// https://github.com/not-fl3/macroquad/issues/749
#[macroquad::main("CHIP-8")]
async fn main() -> Result<(), std::io::Error> {
    let variants = ["CHIP-8", "CHIP-48"];
    let mut cursor = 0usize;
    let mut emulator_state = EmulatorState::Start;

    let frame_dur = Duration::from_secs_f64(1.0 / 60.0);
    let mut next_tick = Instant::now();

    let mut vm: Option<VirtualMachine> = None;

    loop {
        clear_background(BLACK);

        match emulator_state {
            EmulatorState::Start => {
                draw_highlight(variants, &mut cursor);

                if is_key_pressed(KeyCode::Enter) {
                    match Cartridge::load(file_dialog()) {
                        Ok(cartridge) => {
                            let mut audio = Audio::start(441.0, 0.1, 44100);
                            audio.beep = Some(
                                load_sound_from_bytes(audio.wav_bytes.as_ref().unwrap())
                                    .await
                                    .unwrap(),
                            );
                            vm = Some(VirtualMachine::boot(cartridge, variants[cursor], audio));
                            emulator_state = EmulatorState::Active;
                        }
                        Err(e) => {
                            eprintln!("{e}");
                            process::exit(1);
                        }
                    }
                } else {
                    quit_emulator(&mut emulator_state, EmulatorState::Start);
                }
            }
            EmulatorState::Active => {
                if let Some(vm) = vm.as_mut() {
                    vm.process();
                    vm.update_timers();
                    vm.display.draw();
                }

                quit_emulator(&mut emulator_state, EmulatorState::Active);
            }
            EmulatorState::Quit => break,
        }

        next_tick += frame_dur;
        let now = Instant::now();
        if next_tick > now {
            spin_sleep::sleep(next_tick - now);
        } else {
            next_tick = now;
        }

        next_frame().await;
    }

    Ok(())
}
