use hound::{SampleFormat, WavSpec, WavWriter};
use macroquad::audio::{Sound, play_sound_once};
use std::{f32::consts::PI, io::Cursor};

pub struct Audio {
    pub beep: Option<Sound>,
    pub wav_bytes: Option<Vec<u8>>,
    pub sound_timer: u8,
}

impl Audio {
    pub fn start(frequency: f32, duration_seconds: f32, sample_rate: u32) -> Self {
        let wavspec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let samples_per_cycle = sample_rate as f32 / frequency;
        let cycles = (sample_rate as f32 * duration_seconds / samples_per_cycle)
            .round()
            .max(1.0);
        let n_samples = (cycles * samples_per_cycle).round() as usize;

        let mut buffer = Vec::new();
        let mut writer = WavWriter::new(Cursor::new(&mut buffer), wavspec).unwrap();

        let amplitude = i16::MAX as f32;
        for i in 0..n_samples {
            let time = i as f32 / sample_rate as f32;
            let sample = (2.0 * PI * frequency * time).sin();
            writer.write_sample((sample * amplitude) as i16).unwrap();
        }

        writer.finalize().unwrap();

        Self {
            beep: None,
            wav_bytes: Some(buffer),
            sound_timer: 0,
        }
    }

    pub fn play(&self) {
        play_sound_once(self.beep.as_ref().unwrap());
    }
}
