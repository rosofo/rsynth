use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

use crate::{
    chain::Voice,
    config::{Config, ConfigReceiver, HasConfig, ValidatedConfig},
};

#[derive(Debug, Clone, Copy)]
pub struct TwoChannelConfig {
    pub a_mix: f32,
    pub b_mix: f32,
}

fn validate_two_channel_config(config: TwoChannelConfig) -> bool {
    let within_range = |vol| vol >= 0.0 && vol <= 1.0;
    within_range(config.a_mix) && within_range(config.b_mix)
}

pub struct TwoChannel<S, Va: Voice<S>, Vb: Voice<S>> {
    pub a: Va,
    pub b: Vb,
    pub config: ValidatedConfig<TwoChannelConfig, fn(TwoChannelConfig) -> bool>,
    _phantom: PhantomData<S>,
}

impl<S, Va: Voice<S>, Vb: Voice<S>> TwoChannel<S, Va, Vb> {
    pub fn new(a: Va, b: Vb) -> Self {
        Self {
            a,
            b,
            config: ValidatedConfig::new(
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
