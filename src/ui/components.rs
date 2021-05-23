use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

use crossterm::event::KeyCode;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{BarChart, Block, Borders, Clear, Paragraph, Widget},
};

use crate::{
    chain::Voice,
    combinators::{MixerClient, TwoChannelClient, TwoChannelConfig},
    config::{ComposeConfigClient, ConfigClient},
    controllers::{KBConfigAction, KeyboardControllerClient},
    voices::{AdditiveAction, AdditiveConfig, HasFreq},
};

use super::input::InputEvent;

pub trait UIComponent: RefWidget {
    fn dispatch(&mut self, event: InputEvent);
}

impl UIComponent for Box<dyn UIComponent + Send + 'static> {
    fn dispatch(&mut self, event: InputEvent) {
        (**self).dispatch(event)
    }
}
impl RefWidget for Box<dyn UIComponent + Send + 'static> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        (**self).render(area, buf)
    }
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
    mixer.update(|cf| TwoChannelConfig {
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

pub struct MixerComponent {
    pub client: MixerClient,
}

impl RefWidget for MixerComponent {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(5)
            .split(area);

        let data = self
            .client
            .get()
            .channels
            .iter()
            .enumerate()
            .map(|(channel, volume)| (channel.to_string(), *volume as u64))
            .collect::<Vec<(String, u64)>>();

        let data = data
            .iter()
            .map(|(label, v)| (label.as_str(), *v))
            .collect::<Vec<(&str, u64)>>();

        let bars = BarChart::default()
            .block(Block::default().borders(Borders::ALL).title("Mixer"))
            .bar_width(3)
            .bar_gap(3)
            .data(data.as_slice());

        Clear.render(area, buf);
        bars.render(rects[0], buf);
    }
}

impl UIComponent for MixerComponent {
    fn dispatch(&mut self, event: InputEvent) {}
}

pub struct NavigationContainer<C: UIComponent> {
    components: Vec<C>,
    direction: Direction,
    selected: usize,
    focused: Option<usize>,
}

impl<C: UIComponent> NavigationContainer<C> {
    pub fn new(components: Vec<C>, direction: Direction) -> Self {
        Self {
            components,
            direction,
            focused: None,
            selected: 0,
        }
    }

    fn handle_movement(&mut self, event: InputEvent) {
        match event {
            InputEvent::Enter => {
                self.focused = Some(self.selected);
                return;
            }
            _ => {}
        }

        match self.direction {
            Direction::Horizontal => match event {
                InputEvent::Left => {
                    if self.selected > 0 {
                        self.selected -= 1
                    }
                }
                InputEvent::Right => {
                    if self.selected < self.components.len() - 1 {
                        self.selected += 1
                    }
                }
                _ => {}
            },
            Direction::Vertical => match event {
                InputEvent::Up => {
                    if self.selected > 0 {
                        self.selected -= 1
                    }
                }
                InputEvent::Down => {
                    if self.selected < self.components.len() - 1 {
                        self.selected += 1
                    }
                }
                _ => {}
            },
        }
    }

    fn render_indicator_blocks(&self, index: usize, rect: Rect, buf: &mut Buffer) {
        if index == self.selected {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .render(rect, buf);
        }

        if let Some(focused) = self.focused {
            if index == focused {
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .render(rect, buf);
            }
        }
    }
}

impl<C: UIComponent> RefWidget for NavigationContainer<C> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let constraints: Vec<Constraint> =
            [Constraint::Percentage(100 / (self.components.len() as u16))]
                .iter()
                .cycle()
                .copied()
                .take(self.components.len())
                .collect();
        let row = Layout::default()
            .direction(self.direction.clone())
            .constraints(constraints)
            .split(area);

        for (index, (rect, component)) in row.into_iter().zip(self.components.iter()).enumerate() {
            self.render_indicator_blocks(index, rect, buf);

            component.render(rect, buf);
        }
    }
}

impl<C: UIComponent> UIComponent for NavigationContainer<C> {
    fn dispatch(&mut self, event: InputEvent) {
        if let Some(index) = self.focused {
            self.components[index].dispatch(event);
        } else {
            self.handle_movement(event);
        }
    }
}

pub struct AdditiveComponent {
    pub client: ComposeConfigClient<
        AdditiveConfig,
        AdditiveAction,
        fn(AdditiveConfig, AdditiveAction) -> AdditiveConfig,
    >,
}

impl RefWidget for AdditiveComponent {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(format!("{:?}", self.client.get())).render(area, buf);
    }
}

impl UIComponent for AdditiveComponent {
    fn dispatch(&mut self, event: InputEvent) {}
}
