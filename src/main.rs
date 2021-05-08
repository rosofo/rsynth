use std::sync::{Arc, Mutex};

use chain::Chain;
use combinators::{TwoChannel, TwoChannelConfig};
use config::Config;
use cpal::{traits::StreamTrait, Sample};
use effects::{Gate, LowPassFilter, FM};
use synth::Synth;
use ui::{draw_synth, get_terminal};
use voices::{Chained, Sine};

use crate::voices::SineConfig;

mod audio;
mod chain;
mod combinators;
mod config;
mod effects;
mod synth;
mod ui;
mod voices;

#[tokio::main]
async fn main() {
    let voice = Sine::new(400.0);
    let mut modulator = Sine::new(20.0);
    let fm = FM::new(modulator, voice);
    let mut chain = Chain::new();

    let cutoff_config = Config::new(2.0);
    chain.add(Box::new(Gate { cutoff_config }));
    let chained = Chained::new(fm, chain);

    let sine_2 = Sine::new(440.0);
    let mut mix = TwoChannel::new(chained, sine_2);
    let mut mix_config_client = mix.config.get_client().unwrap();

    let mut synth = Synth::new();
    synth.play(Arc::new(Mutex::new(mix)));

    let mut terminal = get_terminal().unwrap();

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let mut config = mix_config_client.get();
        config.a_mix -= 0.1;
        config.b_mix += 0.1;
        mix_config_client.update(config);

        draw_synth(&mut terminal, &mix_config_client).unwrap();
    }
}
