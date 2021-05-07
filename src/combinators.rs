use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

use crate::{
    chain::Voice,
    config::{Config, ConfigReceiver},
};

#[derive(Debug, Clone, Copy)]
pub struct TwoChannelConfig {
    pub a_mix: f32,
    pub b_mix: f32,
}

pub struct TwoChannel<S, Va: Voice<S>, Vb: Voice<S>> {
    pub a: Va,
    pub b: Vb,
    pub config: Config<TwoChannelConfig>,
    _phantom: PhantomData<S>,
}

impl<S, Va: Voice<S>, Vb: Voice<S>> TwoChannel<S, Va, Vb> {
    pub fn new(a: Va, b: Vb) -> Self {
        Self {
            a,
            b,
            config: Config::new(TwoChannelConfig {
                a_mix: 0.5,
                b_mix: 0.5,
            }),
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
        signal_a * self.config.config.a_mix.into() + signal_b * self.config.config.b_mix.into()
    }
}
