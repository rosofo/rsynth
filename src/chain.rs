pub trait Effect<Signal> {
    fn process(&mut self, signal: Signal) -> Signal;
}

pub trait Voice<Signal> {
    fn generate(&mut self) -> Signal;
}

pub enum ChainItem<Signal> {
    Effect(Box<dyn Effect<Signal>>),
    Voice(Box<dyn Voice<Signal>>),
}

pub struct Chain<Signal> {
    pub chain: Vec<Box<dyn Effect<Signal> + Send + Sync + 'static>>,
}

impl<Signal> Chain<Signal> {
    pub fn new() -> Self {
        Chain { chain: Vec::new() }
    }

    pub fn add(&mut self, effect: Box<dyn Effect<Signal> + Send + Sync + 'static>) {
        self.chain.push(effect);
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
