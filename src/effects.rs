use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crossbeam::atomic::AtomicCell;
use num::{Num, Signed};

use crate::chain::Effect;

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

impl Effect<f32> for LowPassFilter<f32> {
    fn process(&mut self, signal: f32) -> f32 {
        let delta = self.delta();
        let movement = self.velocity.load() * delta.as_secs_f32();
        if signal < self.output {
            self.output -= movement;
        } else if signal > self.output {
            self.output += movement;
        }

        self.output
    }
}

pub struct Gate<Signal> {
    pub cutoff: Signal,
}

impl<Signal: PartialOrd + Signed + Default> Effect<Signal> for Gate<Signal> {
    fn process(&mut self, signal: Signal) -> Signal {
        if signal.abs() < self.cutoff {
            signal
        } else {
            Signal::default()
        }
    }
}
