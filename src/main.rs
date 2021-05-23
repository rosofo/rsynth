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

use tui::layout::Direction;
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
        components::{
            AdditiveComponent, KeyboardInputComponent, MixerComponent, NavigationContainer,
        },
        input::parse_input_event,
    },
    voices::Additive,
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
    let mut additive = Additive::new(440.0, vec![2.0, 4.0, 6.0, 8.0]);
    let additive_client = additive.config.get_client().unwrap();
    let mixer_client = additive.mixer.config.get_client().unwrap();
    let fm = FM::new(additive, Sine::new(440.0));
    let mut ctrl = KeyboardController::new(fm);
    let ctrl_client = ctrl.config.get_client().unwrap();

    let mut synth = Synth::new();
    synth.play(ctrl);

    let ui_model = UIModel::new(
        KeyboardInputComponent {
            controller_clients: vec![ctrl_client],
        },
        NavigationContainer::new(
            vec![
                Box::new(MixerComponent {
                    client: mixer_client,
                }) as Box<dyn UIComponent + Send + 'static>,
                Box::new(AdditiveComponent {
                    client: additive_client,
                }) as Box<dyn UIComponent + Send + 'static>,
            ],
            Direction::Horizontal,
        ),
    );
    start(ui_model);
}

fn start<C: UIComponent + Send + 'static>(ui_model: UIModel<C>) {
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
