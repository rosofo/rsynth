use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use crate::{
    chain::{Effect, Voice},
    config::{ComposeConfig, ComposeConfigClient, Config, ConfigReceiver, HasConfig},
    voices::{HasFreq, Waveform},
};

#[derive(Clone, Copy)]
pub enum KBConfigAction {
    Play(f32),
    Stop,
    ChangeBase(f32),
}

pub struct KeyboardController<V: Waveform<f32>> {
    voice: V,
    pub config:
        ComposeConfig<KBCConfig, KBConfigAction, fn(KBCConfig, KBConfigAction) -> KBCConfig>,
}

pub type KeyboardControllerClient =
    ComposeConfigClient<KBCConfig, KBConfigAction, fn(KBCConfig, KBConfigAction) -> KBCConfig>;

fn reduce_kb_config_action(mut config: KBCConfig, action: KBConfigAction) -> KBCConfig {
    match action {
        KBConfigAction::Play(hz) => config.playing_note = Some(hz),
        KBConfigAction::Stop => config.playing_note = None,
        KBConfigAction::ChangeBase(hz) => config.base_hz = hz,
    }

    config
}

impl<V: Waveform<f32>> KeyboardController<V> {
    pub fn new(voice: V) -> Self {
        Self {
            voice,
            config: ComposeConfig::new(KBCConfig::default(), reduce_kb_config_action),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KBCConfig {
    base_hz: f32,
    playing_note: Option<f32>,
}

impl Default for KBCConfig {
    fn default() -> Self {
        Self {
            base_hz: 0.0,
            playing_note: None,
        }
    }
}

impl<V: Waveform<f32>> ConfigReceiver for KeyboardController<V> {
    fn try_update_configs(&mut self) {
        self.config.try_update();
        self.voice.try_update_configs()
    }
}

impl<V: Waveform<f32>> Voice<f32> for KeyboardController<V> {
    fn generate(&mut self) -> f32 {
        if let Some(hz) = self.config.get().playing_note {
            self.voice.set_freq(hz + self.config.get().base_hz);
            self.voice.generate()
        } else {
            0.0
        }
    }
}
