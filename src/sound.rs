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
    }
}

pub struct SoundPlayer {
    stream: MixerDeviceSink,
    pub pack: SoundPack,
    pub volume_pct: u8,
}

const SAMPLE_RATE: u32 = 44100;

fn beep_buf(hz: f32, ms: u64, volume: f32) -> SamplesBuffer {
    let n = (SAMPLE_RATE as f64 * ms as f64 / 1000.0) as usize;
    let samples: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let fade = 1.0 - (i as f32 / n as f32); // linear fade-out
            (2.0 * PI * hz * t).sin() * fade * volume
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
            volume_pct: 25,
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
            SoundPack::Click => (1100.0_f32, 18u64),
            SoundPack::Pop => (650.0_f32, 25u64),
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
