use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

use crate::{
    chain::Voice,
    config::{
        ComposeConfig, ComposeConfigClient, Config, ConfigReceiver, HasConfig, ValidatedConfig,
        ValidatedConfigClient,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct TwoChannelConfig {
    pub a_mix: f32,
    pub b_mix: f32,
}

fn validate_two_channel_config(config: &TwoChannelConfig) -> bool {
    let within_range = |vol| vol >= 0.0 && vol <= 1.0;
    within_range(config.a_mix) && within_range(config.b_mix)
}

pub struct TwoChannel<S, Va: Voice<S>, Vb: Voice<S>> {
    pub a: Va,
    pub b: Vb,
    pub config: ValidatedConfig<TwoChannelConfig>,
    _phantom: PhantomData<S>,
}

pub type TwoChannelClient = ValidatedConfigClient<TwoChannelConfig>;

impl<S, Va: Voice<S>, Vb: Voice<S>> TwoChannel<S, Va, Vb> {
    pub fn new(a: Va, b: Vb) -> Self {
        Self {
            a,
            b,
            config: ValidatedConfig::new_validated(
                TwoChannelConfig {
                    a_mix: 0.5,
                    b_mix: 0.5,
                },
                validate_two_channel_config,
            ),
            _phantom: PhantomData,
        }
    }
}

impl<S, Va: Voice<S>, Vb: Voice<S>> ConfigReceiver for TwoChannel<S, Va, Vb> {
    fn try_update_configs(&mut self) {
        self.config.try_update();
        self.a.try_update_configs();
        self.b.try_update_configs();
    }
}

impl<S: Add<Output = S> + Mul<Output = S> + From<f32>, Va: Voice<S>, Vb: Voice<S>> Voice<S>
    for TwoChannel<S, Va, Vb>
{
    fn generate(&mut self) -> S {
        let signal_a = self.a.generate();
        let signal_b = self.b.generate();
        signal_a * self.config.get().a_mix.into() + signal_b * self.config.get().b_mix.into()
    }
}

pub struct Mixer<V: Voice<f32>> {
    pub config:
        ComposeConfig<MixerConfig, MixerAction, fn(MixerConfig, MixerAction) -> MixerConfig>,
    pub voices: Vec<V>,
}

pub type MixerClient =
    ComposeConfigClient<MixerConfig, MixerAction, fn(MixerConfig, MixerAction) -> MixerConfig>;

fn reduce_mixer_action(mut config: MixerConfig, action: MixerAction) -> MixerConfig {
    match action {
        MixerAction::Change {
            channel,
            volume_change,
        } => {
            config.channels[channel] += volume_change;
        }
    }

    config
}

impl<V: Voice<f32>> Mixer<V> {
    pub fn new(voices: Vec<V>) -> Self {
        Self {
            config: ComposeConfig::new(
                MixerConfig {
                    channels: vec![0.5; voices.len()],
                },
                reduce_mixer_action,
            ),
            voices,
        }
    }
}

impl<V: Voice<f32>> ConfigReceiver for Mixer<V> {
    fn try_update_configs(&mut self) {
        self.config.try_update();
    }
}

impl<V: Voice<f32>> Voice<f32> for Mixer<V> {
    fn generate(&mut self) -> f32 {
        let channels = &self.config.get().channels;
        let output = self
            .voices
            .iter_mut()
            .enumerate()
            .map(|(channel, voice)| voice.generate() * channels[channel])
            .sum();
        output
    }
}

#[derive(Clone)]
pub struct MixerConfig {
    pub channels: Vec<f32>,
}

#[derive(Clone, Copy)]
pub enum MixerAction {
    Change { channel: usize, volume_change: f32 },
}
