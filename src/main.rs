use std::sync::Arc;

use chain::Chain;
use cpal::{traits::StreamTrait, Sample};
use effects::{Gate, LowPassFilter};
use synth::Synth;
use voices::{Chained, Sine};

mod audio;
mod chain;
mod effects;
mod synth;
mod voices;

fn main() {
    let mut chain = Chain::new();
    let lpf = Box::new(LowPassFilter::new(440.0));
    let velocity = Arc::clone(&lpf.velocity);
    chain.add(Box::new(Gate { cutoff: 0.1 }));
    let chained = Chained::new(Sine::new(300.0), chain);

    let mut synth = Synth::new(chained);
    synth.play();

    loop {
        let mut input = String::new();
        println!("{}, press any key", velocity.load());
        std::io::stdin().read_line(&mut input).unwrap();

        velocity.store(velocity.load() + 50.0);
    }
}
