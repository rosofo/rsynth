use std::{
    marker::PhantomData,
    sync::Arc,
    time::{Duration, Instant},
};

use crossbeam::atomic::AtomicCell;
use num::{Num, NumCast, Signed};

use crate::{
    chain::{Effect, Voice},
    config::{Config, ConfigReceiver},
    voices::{HasFreq, Waveform},
};

pub struct LowPassFilter<Signal> {
    pub hz: f32,
    pub velocity: Arc<AtomicCell<f32>>,
    output: Signal,
}

impl LowPassFilter<f32> {
    pub fn new(hz: f32) -> Self {
        Self {
            hz,
            velocity: Arc::new(AtomicCell::new(1.0)),
            output: 0.0,
        }
    }

    pub fn delta(&mut self) -> Duration {
        Duration::from_secs_f32(1.0 / 44100.0)
    }
}

//impl Effect<f32> for LowPassFilter<f32> {
//fn process(&mut self, signal: f32) -> f32 {
//let delta = self.delta();
//let movement = self.velocity.load() * delta.as_secs_f32();
//if signal < self.output {
//self.output -= movement;
//} else if signal > self.output {
//self.output += movement;
//}

//self.output
//}
//}

pub struct Gate<Signal> {
    pub cutoff_config: Config<Signal>,
}

impl<S: Copy> ConfigReceiver for Gate<S> {
    fn try_update_configs(&mut self) {
        self.cutoff_config.try_update();
    }
}

impl<Signal: PartialOrd + Signed + Default + Copy> Effect<Signal> for Gate<Signal> {
    fn process(&mut self, signal: Signal) -> Signal {
        if signal.abs() < self.cutoff_config.config {
            signal
        } else {
            Signal::default()
        }
    }
}

pub struct FM<S, M: Waveform<S>, V: Waveform<S>> {
    voice: V,
    modulator: M,
    _phantom: PhantomData<S>,
}

impl<S, M: Waveform<S>, V: Waveform<S>> FM<S, M, V> {
    pub fn new(modulator: M, voice: V) -> Self {
        Self {
            voice,
            modulator,
            _phantom: PhantomData,
        }
    }
}

impl<S, M: Waveform<S>, V: Waveform<S>> HasFreq for FM<S, M, V> {
    fn freq(&mut self, hz: f32) {
        self.modulator.freq(hz);
    }
}

impl<S, M: Waveform<S>, V: Waveform<S>> ConfigReceiver for FM<S, M, V> {
    fn try_update_configs(&mut self) {
        self.modulator.try_update_configs();
        self.voice.try_update_configs();
    }
}

impl<S: NumCast, M: Waveform<S>, V: Waveform<S>> Voice<S> for FM<S, M, V> {
    fn generate(&mut self) -> S {
        self.voice.freq(self.modulator.generate().to_f32().unwrap());
        self.voice.generate()
    }
}
