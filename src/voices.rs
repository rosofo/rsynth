use std::{marker::PhantomData, sync::Arc, time::Instant};

use crossbeam::atomic::AtomicCell;

use crate::{
    chain::{Chain, Effect, Voice},
    config::{Config, ConfigReceiver},
};

pub trait HasFreq {
    fn freq(&mut self, hz: f32);
}

pub trait Waveform<Signal>: HasFreq + Voice<Signal> {}
impl<S, T: HasFreq + Voice<S>> Waveform<S> for T {}

#[derive(Clone, Copy)]
pub struct SineConfig {
    pub hz: f32,
}

pub struct Sine<Signal> {
    pub config: Config<SineConfig>,
    hz: f32,
    clock: f32,
    _phantom: PhantomData<Signal>,
}

impl<Signal> Sine<Signal> {
    pub fn new(hz: f32) -> Self {
        Sine {
            config: Config::new(SineConfig { hz }),
            hz,
            clock: 0.0,
            _phantom: PhantomData {},
        }
    }

    fn try_update_hz(&mut self) {
        let new_hz = self.config.config.hz;

        if new_hz != self.hz {
            let signal = self.get_output();
            let limit = 0.0001;
            if signal.abs() < limit {
                self.hz = new_hz;
            }
        }
    }

    fn get_output(&self) -> f32 {
        ((self.clock * self.hz) - 1.0).sin()
    }
}

impl<S> ConfigReceiver for Sine<S> {
    fn try_update_configs(&mut self) {
        self.config.try_update();
    }
}

impl Voice<f32> for Sine<f32> {
    fn generate(&mut self) -> f32 {
        self.try_update_hz();
        self.clock += 1.0 / 44100.0;
        self.get_output()
    }
}

impl<S> HasFreq for Sine<S> {
    fn freq(&mut self, hz: f32) {
        self.hz = hz;
    }
}

pub struct Chained<Signal, V: Voice<Signal>, E: Effect<Signal>> {
    pub voice: V,
    pub effect: E,
    _phantom: PhantomData<Signal>,
}

impl<S, V: Voice<S>, E: Effect<S>> Chained<S, V, E> {
    pub fn new(voice: V, effect: E) -> Self {
        Self {
            _phantom: PhantomData,
            voice,
            effect,
        }
    }
}

impl<V: Voice<Signal>, E: Effect<Signal>, Signal> Voice<Signal> for Chained<Signal, V, E> {
    fn generate(&mut self) -> Signal {
        self.effect.process(self.voice.generate())
    }
}

impl<S, V: Voice<S> + ConfigReceiver, E: Effect<S> + ConfigReceiver> ConfigReceiver
    for Chained<S, V, E>
{
    fn try_update_configs(&mut self) {
        self.voice.try_update_configs();
        self.effect.try_update_configs();
    }
}
