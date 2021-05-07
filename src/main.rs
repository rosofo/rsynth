use std::sync::{Arc, Mutex};

use chain::Chain;
use combinators::{TwoChannel, TwoChannelConfig};
use config::Config;
use cpal::{traits::StreamTrait, Sample};
use effects::{Gate, LowPassFilter, FM};
use synth::Synth;
use voices::{Chained, Sine};

use crate::voices::SineConfig;

mod audio;
mod chain;
mod combinators;
mod config;
mod effects;
mod synth;
mod voices;

fn main() {
    let voice = Sine::new(400.0);
    let modulator = Sine::new(20.0);
    let mod_hz_sender = modulator.config.sender.clone();
    let mut mod_hz = 20.0;
    let fm = FM::new(modulator, voice);
    let mut chain = Chain::new();

    let cutoff_config = Config::new(2.0);
    let gate_cutoff_sender = cutoff_config.sender.clone();
    chain.add(Box::new(Gate { cutoff_config }));
    let chained = Chained::new(fm, chain);

    let sine_2 = Sine::new(440.0);
    let mix = TwoChannel::new(chained, sine_2);
    let mix_config_sender = mix.config.sender.clone();
    let mut mix_config = TwoChannelConfig {
        a_mix: 1.0,
        b_mix: 0.0,
    };

    let mut synth = Synth::new();
    synth.play(Arc::new(Mutex::new(mix)));

    loop {
        let mut input = String::new();
        println!("{:?}, press any key", mix_config);
        std::io::stdin().read_line(&mut input).unwrap();

        mix_config.a_mix -= 0.1;
        mix_config.b_mix += 0.1;
        mix_config_sender.send(mix_config).unwrap();
    }
}
