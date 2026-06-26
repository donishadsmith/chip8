use crate::{
    Variant,
    components::{
        audio::Audio,
        bus::Bus,
        cartridge::Cartridge,
        cpu::{CPU, STARTING_ROM_ADDRESS},
        display::Display,
        ram::RAM,
    },
    fontset::FONTSET,
};

pub struct VirtualMachine {
    pub cpu: CPU,
    pub bus: Bus,
    pub delay_timer: u8,
    pub audio: Audio,
    pub variant: Variant,
}

impl VirtualMachine {
    pub fn boot(cartridge: Cartridge, variant: Variant, audio: Audio) -> Self {
        Self {
            cpu: CPU::start(),
            bus: Bus {
                ram: Self::controller(cartridge),
                display: Display::on(),
            },
            delay_timer: 0,
            audio: audio,
            variant: variant,
        }
    }

    pub fn controller(cartridge: Cartridge) -> RAM {
        let mut ram = RAM::start();
        for index in 0..FONTSET.len() {
            ram.memory[index] = FONTSET[index];
        }

        for (index, byte) in cartridge.buffer.iter().enumerate() {
            let address = STARTING_ROM_ADDRESS + index as u16;
            if address >= 4096 {
                break;
            }

            ram.memory[address as usize] = *byte;
        }

        ram
    }

    //https://www.reddit.com/r/EmuDev/comments/1ev3ool/chip8_instructions_per_second/
    const CYCLES_PER_FRAME: u8 = 10;
    pub fn process(&mut self) {
        for _ in 0..Self::CYCLES_PER_FRAME {
            self.cpu.control_unit.cycle(
                &self.variant,
                &mut self.bus,
                &mut self.cpu.index_register,
                &mut self.cpu.registers,
                &mut self.delay_timer,
                &mut self.audio.sound_timer,
            );
        }

        self.bus.display.draw();
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.audio.sound_timer > 0 {
            if self.audio.sound_timer == 1 {
                self.audio.play();
            }

            self.audio.sound_timer -= 1;
        }
    }
}
