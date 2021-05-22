use std::{
    rc::Rc,
    sync::{Arc, LockResult, Mutex, MutexGuard, RwLock},
    time::Duration,
};

use chain::Chain;
use combinators::{TwoChannel, TwoChannelConfig};
use config::Config;
use cpal::{traits::StreamTrait, Sample};
use crossterm::{
    event::{self, read, EventStream},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use effects::{Gate, LowPassFilter, FM};
use synth::Synth;

use ui::{
    components::{ChainComponent, TwoChannelComponent, UIComponent},
    draw_synth, get_terminal, input,
    model::UIModel,
};
use voices::{Chained, Sine};

use crate::{
    config::HasConfig,
    controllers::{KBCConfig, KeyboardController},
    ui::{
        components::{KeyboardInputComponent, VoiceComponent},
        input::parse_input_event,
    },
};

mod audio;
mod chain;
mod combinators;
mod config;
mod controllers;
mod effects;
mod synth;
mod ui;
mod voices;

fn main() {
    let mut voice_a = Sine::new(440.0);
    let voice_a_client = voice_a.config.get_client().unwrap();
    let mut ctrl = KeyboardController::new(voice_a);
    let ctrl_client = ctrl.config.get_client().unwrap();

    let mut voice_b = Sine::new(440.0);
    let voice_b_client = voice_b.config.get_client().unwrap();
    let chain_a = Chain::new();
    let chain_b = Chain::new();
    let mut mixer = TwoChannel::new(Chained::new(ctrl, chain_a), Chained::new(voice_b, chain_b));
    let mixer_client = mixer.config.get_client().unwrap();

    let mut synth = Synth::new();
    synth.play(Arc::new(Mutex::new(mixer)));

    let ui_model = UIModel::new(
        KeyboardInputComponent {
            controller_clients: vec![ctrl_client],
        },
        TwoChannelComponent {
            client: Arc::new(Mutex::new(mixer_client)),
            selected_channel: 0,
        },
        ChainComponent {
            components: vec![Box::new(VoiceComponent::new(voice_a_client))],
        },
        ChainComponent {
            components: vec![Box::new(VoiceComponent::new(voice_b_client))],
        },
    );
    start(ui_model);
}

fn start(ui_model: UIModel) {
    let ui_model = Arc::new(Mutex::new(ui_model));
    let ui_model_arc_copy = Arc::clone(&ui_model);
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || loop {
        let event = read();
        event
            .ok()
            .and_then(parse_input_event)
            .and_then(|e| sender.send(e).ok());
    });

    std::thread::spawn(move || {
        for event in receiver.iter() {
            let mut model = ui_model.lock().unwrap();
            model.dispatch(event);
        }
    });

    std::thread::spawn(move || {
        let mut terminal = get_terminal().unwrap();
        loop {
            draw_synth(&mut terminal, &ui_model_arc_copy).unwrap();
            std::thread::sleep(Duration::from_millis(17));
        }
    });

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
