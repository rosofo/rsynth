use crate::config::{Config, ConfigReceiver};

pub trait Effect<Signal>: ConfigReceiver {
    fn process(&mut self, signal: Signal) -> Signal;
}

pub trait Voice<Signal>: ConfigReceiver {
    fn generate(&mut self) -> Signal;
}

pub struct Chain<Signal> {
    pub chain: Vec<Box<dyn Effect<Signal> + Send + 'static>>,
}

impl<Signal> Chain<Signal> {
    pub fn new() -> Self {
        Chain { chain: Vec::new() }
    }

    pub fn add(&mut self, effect: Box<dyn Effect<Signal> + Send + 'static>) {
        self.chain.push(effect);
    }
}

impl<S> ConfigReceiver for Chain<S> {
    fn try_update_configs(&mut self) {
        for effect in self.chain.iter_mut() {
            effect.try_update_configs();
        }
    }
}

impl<Signal> Effect<Signal> for Chain<Signal> {
    fn process(&mut self, input: Signal) -> Signal {
        let mut output = input;
        for effect in self.chain.iter_mut() {
            output = effect.process(output);
        }
        output
    }
}
