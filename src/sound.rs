//! Keystroke/completion sound effects via `rodio`. Tones are synthesized
//! sine-wave beeps rather than sample files, so there's nothing to bundle.

use rodio::buffer::SamplesBuffer;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use std::f32::consts::PI;
use std::num::NonZero;

cycle_enum! {
    #[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
    pub enum SoundPack {
        Off = "off",
        Click = "click",
        Pop = "pop",
        Beep = "beep",
        Typewriter = "typewriter",
    }
}

pub struct SoundPlayer {
    stream: MixerDeviceSink,
    pub pack: SoundPack,
    pub volume_pct: u8,
}

const SAMPLE_RATE: u32 = 44100;

/// Raised-cosine attack/decay envelope (zero slope at both ends) instead of
/// an instant onset + linear fade — the sudden onset was what made the
/// original beeps sound like harsh clicks rather than soft tones.
fn beep_buf(hz: f32, ms: u64, volume: f32) -> SamplesBuffer {
    let n = ((SAMPLE_RATE as f64 * ms as f64 / 1000.0) as usize).max(1);
    let attack_n = (n / 6).max(1).min(n);
    let samples: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let envelope = if i < attack_n {
                0.5 * (1.0 - (PI * i as f32 / attack_n as f32).cos())
            } else {
                let j = i - attack_n;
                let decay_n = (n - attack_n).max(1);
                0.5 * (1.0 + (PI * j as f32 / decay_n as f32).cos())
            };
            (2.0 * PI * hz * t).sin() * envelope * volume
        })
        .collect();
    SamplesBuffer::new(
        NonZero::new(1).unwrap(),
        NonZero::new(SAMPLE_RATE).unwrap(),
        samples,
    )
}

impl SoundPlayer {
    pub fn new() -> Option<Self> {
        let stream = DeviceSinkBuilder::open_default_sink().ok()?;
        Some(Self {
            stream,
            pack: SoundPack::Click,
            volume_pct: crate::config::DEFAULT_VOLUME_PCT,
        })
    }

    pub fn set_volume_pct(&mut self, pct: u8) {
        self.volume_pct = pct.clamp(1, 100);
    }

    fn volume(&self) -> f32 {
        self.volume_pct as f32 / 100.0
    }

    pub fn play_correct(&self) {
        let (hz, ms) = match self.pack {
            SoundPack::Off => return,
            SoundPack::Click => (850.0_f32, 20u64),
            SoundPack::Pop => (500.0_f32, 30u64),
            SoundPack::Beep => (350.0_f32, 45u64),
            SoundPack::Typewriter => (180.0_f32, 14u64),
        };
        self.play(beep_buf(hz, ms, self.volume()));
    }

    pub fn play_error(&self) {
        if self.pack == SoundPack::Off {
            return;
        }
        self.play(beep_buf(200.0, 80, self.volume()));
    }

    pub fn play_complete(&self) {
        if self.pack == SoundPack::Off {
            return;
        }
        let player = Player::connect_new(self.stream.mixer());
        player.append(beep_buf(523.0, 180, self.volume()));
        player.append(beep_buf(659.0, 180, self.volume()));
        player.detach();
    }

    fn play(&self, buf: SamplesBuffer) {
        let player = Player::connect_new(self.stream.mixer());
        player.append(buf);
        player.detach();
    }
}
