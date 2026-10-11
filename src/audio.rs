use std::io::Cursor;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoundEffect {
    TileClick,
    TileReveal(u8),
    FlagPlace,
    FlagRemove,
    Chord,
    Explosion,
    Victory,
    Defeat,
    UiClick,
}

pub struct SoundManager {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub muted: bool,
    pub pitch_variation: bool,
}

impl SoundManager {
    pub fn new() -> Self {
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!("[Audio] Warning: Could not initialize audio output: {}. Running in silent mode.", e);
                (None, None)
            }
        };

        Self {
            _stream: stream,
            stream_handle,
            master_volume: 0.8,
            sfx_volume: 0.9,
            muted: false,
            pitch_variation: true,
        }
    }

    pub fn play(&self, effect: SoundEffect) {
        if self.muted || self.master_volume <= 0.001 || self.sfx_volume <= 0.001 {
            return;
        }

        let handle = match &self.stream_handle {
            Some(h) => h,
            None => return,
        };

        let wav_data = generate_procedural_wav(effect, self.pitch_variation);
        let cursor = Cursor::new(wav_data);

        if let Ok(source) = Decoder::new(cursor) {
            if let Ok(sink) = Sink::try_new(handle) {
                let volume = (self.master_volume * self.sfx_volume).clamp(0.0, 1.0);
                sink.set_volume(volume);
                sink.append(source);
                sink.detach();
            }
        }
    }
}

/// Generates an in-memory 16-bit mono 44.1kHz PCM WAV buffer for a given sound effect.
fn generate_procedural_wav(effect: SoundEffect, pitch_var: bool) -> Vec<u8> {
    let sample_rate = 44100u32;
    let mut rng = rand::thread_rng();
    let pitch_factor = if pitch_var {
        rng.gen_range(0.96..=1.04)
    } else {
        1.0
    };

    let samples: Vec<i16> = match effect {
        SoundEffect::TileClick | SoundEffect::UiClick => {
            // Tactile crisp micro-click (12ms)
            let duration = 0.015;
            let total_samples = (sample_rate as f32 * duration) as usize;
            (0..total_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    let env = (1.0 - t / duration).powf(3.0);
                    let freq = 1800.0 * pitch_factor;
                    let tone = (t * freq * std::f32::consts::TAU).sin();
                    let click = if i < 15 { rng.gen_range(-1.0..1.0) } else { 0.0 };
                    let sample = (tone * 0.7 + click * 0.3) * env;
                    (sample * 16000.0) as i16
                })
                .collect()
        }
        SoundEffect::TileReveal(num) => {
            // Melodic cascade pop with pitch scaled by number
            let base_freq = match num {
                0 => 440.0,
                1 => 523.25, // C5
                2 => 587.33, // D5
                3 => 659.25, // E5
                4 => 698.46, // F5
                5 => 783.99, // G5
                6 => 880.00, // A5
                7 => 987.77, // B5
                _ => 1046.50, // C6
            } * pitch_factor;

            let duration = 0.08;
            let total_samples = (sample_rate as f32 * duration) as usize;
            (0..total_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    let env = (1.0 - t / duration).powf(2.0);
                    let freq = base_freq * (1.0 + 0.1 * (1.0 - t / duration));
                    let tone = (t * freq * std::f32::consts::TAU).sin();
                    let harmonic = (t * freq * 2.0 * std::f32::consts::TAU).sin() * 0.3;
                    let sample = (tone + harmonic) * env * 0.7;
                    (sample * 18000.0) as i16
                })
                .collect()
        }
        SoundEffect::FlagPlace => {
            // Snappy mechanical pin lock (two-stage click + metallic ping)
            let duration = 0.06;
            let total_samples = (sample_rate as f32 * duration) as usize;
            (0..total_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    let env = (1.0 - t / duration).powf(2.5);
                    let freq = 1200.0 * pitch_factor;
                    let tone = (t * freq * std::f32::consts::TAU).sin();
                    let ping = (t * 2800.0 * std::f32::consts::TAU).sin() * 0.4;
                    let noise = if i < 30 { rng.gen_range(-0.75..0.75) } else { 0.0 };
                    let sample = (tone + ping + noise) * env * 0.7;
                    (sample * 20000.0) as i16
                })
                .collect()
        }
        SoundEffect::FlagRemove => {
            // Lighter reverse unlatch click
            let duration = 0.05;
            let total_samples = (sample_rate as f32 * duration) as usize;
            (0..total_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    let env = (1.0 - t / duration).powf(3.0);
                    let freq = 850.0 * pitch_factor;
                    let tone = (t * freq * std::f32::consts::TAU).sin();
                    let noise = if i < 20 { rng.gen_range(-0.5..0.5) } else { 0.0 };
                    let sample = (tone + noise) * env * 0.6;
                    (sample * 17000.0) as i16
                })
                .collect()
        }
        SoundEffect::Chord => {
            // Energetic dual harmonic sweep chime
            let duration = 0.16;
            let total_samples = (sample_rate as f32 * duration) as usize;
            (0..total_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    let env = (1.0 - t / duration).powf(1.8);
                    let f1 = (600.0 + t * 400.0) * pitch_factor;
                    let f2 = (900.0 + t * 600.0) * pitch_factor;
                    let tone1 = (t * f1 * std::f32::consts::TAU).sin();
                    let tone2 = (t * f2 * std::f32::consts::TAU).sin() * 0.6;
                    let sample = (tone1 + tone2) * env * 0.6;
                    (sample * 18000.0) as i16
                })
                .collect()
        }
        SoundEffect::Explosion => {
            // Deep bass rumble + filtered noise shockwave burst
            let duration = 0.45;
            let total_samples = (sample_rate as f32 * duration) as usize;
            let mut filter_state = 0.0f32;
            (0..total_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    let env = (1.0 - t / duration).powf(1.5);
                    let raw_noise = rng.gen_range(-1.0..1.0);
                    // One-pole low-pass filter
                    let cutoff = (1.0 - t / duration).clamp(0.05, 0.85);
                    filter_state += cutoff * (raw_noise - filter_state);
                    let rumble_freq = 75.0 * (1.0 - t * 0.8);
                    let rumble = (t * rumble_freq * std::f32::consts::TAU).sin() * 0.8;
                    let sample = (filter_state * 0.6 + rumble * 0.6) * env;
                    (sample.clamp(-1.0, 1.0) * 26000.0) as i16
                })
                .collect()
        }
        SoundEffect::Victory => {
            // 4-note ascending triumphant arpeggio (C5 - E5 - G5 - C6)
            let note_dur = 0.09;
            let notes = [523.25, 659.25, 783.99, 1046.50];
            let total_duration = note_dur * 4.0 + 0.25;
            let total_samples = (sample_rate as f32 * total_duration) as usize;
            (0..total_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    let mut sum = 0.0f32;
                    for (idx, &freq) in notes.iter().enumerate() {
                        let note_start = idx as f32 * note_dur;
                        if t >= note_start {
                            let note_t = t - note_start;
                            let dur = if idx == 3 { 0.35 } else { 0.2 };
                            if note_t < dur {
                                let env = (1.0 - note_t / dur).powf(1.5);
                                let tone = (note_t * freq * std::f32::consts::TAU).sin();
                                let harm = (note_t * freq * 2.0 * std::f32::consts::TAU).sin() * 0.3;
                                sum += (tone + harm) * env * 0.4;
                            }
                        }
                    }
                    (sum.clamp(-1.0, 1.0) * 20000.0) as i16
                })
                .collect()
        }
        SoundEffect::Defeat => {
            // Descending minor warning drone
            let duration = 0.35;
            let total_samples = (sample_rate as f32 * duration) as usize;
            (0..total_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    let env = (1.0 - t / duration).powf(1.4);
                    let f1 = (330.0 - t * 150.0).max(60.0);
                    let f2 = (311.13 - t * 140.0).max(55.0); // Minor tone dissonance
                    let tone = (t * f1 * std::f32::consts::TAU).sin() * 0.5
                        + (t * f2 * std::f32::consts::TAU).sin() * 0.5;
                    let sample = tone * env * 0.7;
                    (sample * 21000.0) as i16
                })
                .collect()
        }
    };

    write_wav_buffer(sample_rate, &samples)
}

fn write_wav_buffer(sample_rate: u32, samples: &[i16]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(44 + samples.len() * 2);
    // RIFF chunk descriptor
    buf.extend_from_slice(b"RIFF");
    let file_size: u32 = 36 + (samples.len() * 2) as u32;
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // "fmt " sub-chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // subchunk1 size (16 for PCM)
    buf.extend_from_slice(&1u16.to_le_bytes());  // Audio format 1 = PCM
    buf.extend_from_slice(&1u16.to_le_bytes());  // NumChannels 1 = Mono
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * 2; // sample_rate * num_channels * bits_per_sample / 8
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());  // BlockAlign = num_channels * bits_per_sample / 8
    buf.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample = 16

    // "data" sub-chunk
    buf.extend_from_slice(b"data");
    let data_bytes = (samples.len() * 2) as u32;
    buf.extend_from_slice(&data_bytes.to_le_bytes());
    for &sample in samples {
        buf.extend_from_slice(&sample.to_le_bytes());
    }

    buf
}
