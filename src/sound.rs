use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq)]
pub enum SoundPack {
    Off,
    Click,
    Pop,
}

impl SoundPack {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Click,
            Self::Click => Self::Pop,
            Self::Pop => Self::Off,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Self::Off => Self::Pop,
            Self::Click => Self::Off,
            Self::Pop => Self::Click,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Click => "click",
            Self::Pop => "pop",
        }
    }
}

pub struct SoundPlayer {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    pub pack: SoundPack,
    pub volume_pct: u8,
}

const SAMPLE_RATE: u32 = 44100;

fn beep_buf(hz: f32, ms: u64, volume: f32) -> SamplesBuffer<f32> {
    let n = (SAMPLE_RATE as f64 * ms as f64 / 1000.0) as usize;
    let samples: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let fade = 1.0 - (i as f32 / n as f32); // linear fade-out
            (2.0 * PI * hz * t).sin() * fade * volume
        })
        .collect();
    SamplesBuffer::new(1, SAMPLE_RATE, samples)
}

impl SoundPlayer {
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        Some(Self {
            _stream: stream,
            handle,
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
        if let Ok(sink) = Sink::try_new(&self.handle) {
            sink.append(beep_buf(523.0, 180, self.volume()));
            sink.append(beep_buf(659.0, 180, self.volume()));
            sink.detach();
        }
    }

    fn play(&self, buf: SamplesBuffer<f32>) {
        if let Ok(sink) = Sink::try_new(&self.handle) {
            sink.append(buf);
            sink.detach();
        }
    }
}
