use std::sync::{Arc, Mutex};

use chain::Chain;
use config::Config;
use cpal::{traits::StreamTrait, Sample};
use effects::{Gate, LowPassFilter, FM};
use synth::Synth;
use voices::{Chained, Sine};

use crate::voices::SineConfig;

mod audio;
mod chain;
mod config;
mod effects;
mod synth;
mod voices;

fn main() {
    let voice = Sine::new(20.0);
    let modulator = Sine::new(20.0);
    let mod_hz_sender = modulator.config.sender.clone();
    let mut mod_hz = 20.0;
    let fm = FM::new(modulator, voice);
    let mut chain = Chain::new();

    let cutoff_config = Config::new(2.0);
    let gate_cutoff_sender = cutoff_config.sender.clone();
    chain.add(Box::new(Gate { cutoff_config }));
    let chained = Chained::new(fm, chain);

    let mut synth = Synth::new();
    synth.play(Arc::new(Mutex::new(chained)));

    loop {
        let mut input = String::new();
        println!("{}, press any key", mod_hz);
        std::io::stdin().read_line(&mut input).unwrap();

        mod_hz += 1.0;
        mod_hz_sender.send(SineConfig { hz: mod_hz }).unwrap();
    }
}
