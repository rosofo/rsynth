use std::{marker::PhantomData, ops::Add, sync::Arc, time::Instant};

use crossbeam::atomic::AtomicCell;

use crate::{
    chain::{Chain, Effect, Voice},
    combinators::Mixer,
    config::{ComposeConfig, Config, ConfigReceiver, HasConfig},
};

pub trait HasFreq {
    fn set_freq(&mut self, hz: f32);
    fn get_freq(&self) -> f32;
}

pub trait Waveform<Signal>: HasFreq + Voice<Signal> {}
impl<S, T: HasFreq + Voice<S>> Waveform<S> for T {}

#[derive(Clone, Copy)]
pub struct SineConfig {
    pub hz: f32,
}

impl HasFreq for SineConfig {
    fn set_freq(&mut self, hz: f32) {
        self.hz = hz;
    }

    fn get_freq(&self) -> f32 {
        self.hz
    }
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
    fn set_freq(&mut self, hz: f32) {
        self.hz = hz;
    }

    fn get_freq(&self) -> f32 {
        self.hz
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

pub struct Additive<V: Waveform<f32>> {
    pub config: ComposeConfig<
        AdditiveConfig,
        AdditiveAction,
        fn(AdditiveConfig, AdditiveAction) -> AdditiveConfig,
    >,
    pub mixer: Mixer<V>,
}

impl Additive<Sine<f32>> {
    pub fn new(fundamental: f32, overtones: Vec<f32>) -> Self {
        let voices = (0..overtones.len() + 1)
            .map(|_i| Sine::new(fundamental))
            .collect();

        Self {
            config: ComposeConfig::new(
                AdditiveConfig {
                    fundamental,
                    overtones,
                },
                reduce_additive_action,
            ),
            mixer: Mixer::new(voices),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AdditiveConfig {
    fundamental: f32,
    overtones: Vec<f32>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AdditiveAction {}

fn reduce_additive_action(config: AdditiveConfig, action: AdditiveAction) -> AdditiveConfig {
    config
}

impl<V: Waveform<f32>> ConfigReceiver for Additive<V> {
    fn try_update_configs(&mut self) {
        self.mixer.try_update_configs()
    }
}

impl<V: Waveform<f32>> HasFreq for Additive<V> {
    fn get_freq(&self) -> f32 {
        self.config.get().fundamental
    }

    fn set_freq(&mut self, hz: f32) {
        self.config.config.config.fundamental = hz;
    }
}

impl<V: Waveform<f32>> Voice<f32> for Additive<V> {
    fn generate(&mut self) -> f32 {
        for (sine, multiple) in self
            .mixer
            .voices
            .iter_mut()
            .zip(std::iter::once(&1.0).chain(self.config.get().overtones.iter()))
        {
            sine.set_freq(self.config.get().fundamental * (*multiple))
        }

        self.mixer.generate()
    }
}
