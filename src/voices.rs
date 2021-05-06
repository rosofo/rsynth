use std::{marker::PhantomData, time::Instant};

use crate::chain::{Chain, Effect, Voice};

pub struct Sine<Signal> {
    hz: f32,
    clock: f32,
    _phantom: PhantomData<Signal>,
}

impl<Signal> Sine<Signal> {
    pub fn new(hz: f32) -> Self {
        Sine {
            hz,
            clock: 0.0,
            _phantom: PhantomData {},
        }
    }
}

impl Voice<f32> for Sine<f32> {
    fn generate(&mut self) -> f32 {
        self.clock += 1.0 / 44100.0;
        (self.clock * self.hz).sin()
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
