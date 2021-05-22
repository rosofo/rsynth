use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

use crossterm::event::KeyCode;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{BarChart, Block, Borders, Clear, Paragraph, Widget},
};

use crate::{
    chain::Voice,
    combinators::{TwoChannelClient, TwoChannelConfig},
    config::ConfigClient,
    controllers::{KBConfigAction, KeyboardControllerClient},
    voices::HasFreq,
};

use super::input::InputEvent;

pub trait UIComponent: RefWidget {
    fn dispatch(&mut self, event: InputEvent);
}

pub trait RefWidget {
    fn render(&self, area: Rect, buf: &mut Buffer);
}

pub struct TwoChannelComponent {
    pub client: Arc<Mutex<TwoChannelClient>>,
    pub selected_channel: isize,
}

impl UIComponent for TwoChannelComponent {
    fn dispatch(&mut self, event: InputEvent) {
        match event {
            InputEvent::Left => {
                self.selected_channel = (self.selected_channel as isize - 1).abs() % 2 as isize
            }
            InputEvent::Right => {
                self.selected_channel = (self.selected_channel as isize + 1).abs() % 2 as isize
            }
            InputEvent::Down => {
                increment_channel(
                    self.client.lock().unwrap().borrow_mut(),
                    self.selected_channel,
                    -0.1,
                );
            }
            InputEvent::Up => {
                increment_channel(
                    self.client.lock().unwrap().borrow_mut(),
                    self.selected_channel,
                    0.1,
                );
            }
            _ => {}
        };
    }
}

fn increment_channel(mixer: &mut TwoChannelClient, ch: isize, amount: f32) {
    mixer.update(|cf: TwoChannelConfig| TwoChannelConfig {
        a_mix: if ch == 0 { cf.a_mix + amount } else { cf.a_mix },
        b_mix: if ch == 1 { cf.b_mix + amount } else { cf.b_mix },
    });
}

impl RefWidget for TwoChannelComponent {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let TwoChannelConfig { a_mix, b_mix } = self.client.lock().unwrap().get();

        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(5)
            .split(area);

        let a_label = if self.selected_channel == 0 {
            "(A)"
        } else {
            " A "
        };
        let b_label = if self.selected_channel == 1 {
            "(B)"
        } else {
            " B "
        };
        let data = [
            (a_label, (a_mix * 10.0) as u64),
            (b_label, (b_mix * 10.0) as u64),
        ];
        let bars = BarChart::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Two Channel Mixer"),
            )
            .bar_width(3)
            .bar_gap(3)
            .data(&data)
            .max(10);

        Clear.render(area, buf);
        bars.render(rects[0], buf);
    }
}

pub struct ChainComponent {
    pub components: Vec<Box<dyn UIComponent + Send>>,
}

impl RefWidget for ChainComponent {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(self.components.len() as u16)])
            .split(area);

        for (i, component) in self.components.iter().enumerate() {
            component.render(layout[i], buf);
        }
    }
}

impl UIComponent for ChainComponent {
    fn dispatch(&mut self, event: InputEvent) {
        for component in self.components.iter_mut() {
            component.dispatch(event);
        }
    }
}

pub struct VoiceComponent<C> {
    voice_client: Arc<Mutex<ConfigClient<C>>>,
}

impl<C> VoiceComponent<C> {
    pub fn new(voice_client: ConfigClient<C>) -> Self {
        Self {
            voice_client: Arc::new(Mutex::new(voice_client)),
        }
    }
}

impl<C: HasFreq + Copy> UIComponent for VoiceComponent<C> {
    fn dispatch(&mut self, event: InputEvent) {
        let mut client = self.voice_client.lock().unwrap();
        match event {
            InputEvent::Down => client.update(|mut config| {
                config.set_freq(config.get_freq() - 10.0);
                config
            }),
            InputEvent::Up => client.update(|mut config| {
                config.set_freq(config.get_freq() + 10.0);
                config
            }),
            _ => {}
        }
    }
}

impl<C: HasFreq + Copy> RefWidget for VoiceComponent<C> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let voice_config = self.voice_client.lock().unwrap().get();
        let p = Paragraph::new(voice_config.get_freq().to_string())
            .block(Block::default().borders(Borders::ALL));
        p.render(area, buf);
    }
}

pub struct WrapperWidget<F: FnOnce(Rect, &mut Buffer)> {
    pub func: F,
}

impl<F: FnOnce(Rect, &mut Buffer)> Widget for WrapperWidget<F> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (self.func)(area, buf)
    }
}

pub struct KeyboardInputComponent {
    pub controller_clients: Vec<KeyboardControllerClient>,
}

impl UIComponent for KeyboardInputComponent {
    fn dispatch(&mut self, event: InputEvent) {
        if let Some(action) = parse_keyboard_action(event) {
            for client in self.controller_clients.iter_mut() {
                client.update(|_| action);
            }
        }
    }
}

fn parse_keyboard_action(event: InputEvent) -> Option<KBConfigAction> {
    if let InputEvent::Unmapped(KeyCode::Char(char)) = event {
        match char {
            'a' => Some(KBConfigAction::Play(300.0)),
            's' => Some(KBConfigAction::Play(320.0)),
            'd' => Some(KBConfigAction::Play(340.0)),
            'f' => Some(KBConfigAction::Play(360.0)),
            'g' => Some(KBConfigAction::Play(380.0)),
            'q' => Some(KBConfigAction::Play(400.0)),
            'w' => Some(KBConfigAction::Play(420.0)),
            'e' => Some(KBConfigAction::Play(440.0)),
            'r' => Some(KBConfigAction::Play(460.0)),
            't' => Some(KBConfigAction::Play(480.0)),
            _ => None,
        }
    } else {
        None
    }
}

impl RefWidget for KeyboardInputComponent {
    fn render(&self, area: Rect, buf: &mut Buffer) {}
}
