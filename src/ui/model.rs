use std::{
    fmt::{write, Formatter},
    io::Result,
    sync::mpsc::{Receiver, SendError, Sender},
};

use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{
    chain::Chain,
    combinators::{TwoChannelClient, TwoChannelConfig},
};

use super::{
    components::{
        ChainComponent, KeyboardInputComponent, RefWidget, TwoChannelComponent, UIComponent,
    },
    input::{self, InputEvent},
};

#[derive(PartialEq)]
enum Mode {
    Keyboard,
    Config,
}
impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if *self == Mode::Keyboard { "K" } else { "C" })
    }
}

pub struct UIModel {
    pub keyboard_input: KeyboardInputComponent,
    pub mixer: TwoChannelComponent,
    pub chain_a: ChainComponent,
    pub chain_b: ChainComponent,
    mode: Mode,
    location: UILocation,
}

struct UILocation {
    selected: usize,
    focused: Option<usize>,
}
impl Default for UILocation {
    fn default() -> Self {
        Self {
            selected: 0,
            focused: None,
        }
    }
}
impl UILocation {
    fn at_root(&self) -> bool {
        self.focused.is_none()
    }
    fn right(&mut self) {
        self.selected += 1;
    }
    fn left(&mut self) {
        self.selected -= 1;
    }
    fn enter(&mut self) {
        self.focused = Some(self.selected);
    }
    fn up(&mut self) {
        self.focused = None;
    }
}

impl UIModel {
    pub fn new(
        keyboard_input: KeyboardInputComponent,
        mixer: TwoChannelComponent,
        chain_a: ChainComponent,
        chain_b: ChainComponent,
    ) -> Self {
        Self {
            mode: Mode::Keyboard,
            keyboard_input,
            mixer,
            chain_a,
            chain_b,
            location: UILocation::default(),
        }
    }

    fn handle_movement(&mut self, event: InputEvent) {
        if self.location.at_root() {
            match event {
                InputEvent::Right => self.location.right(),
                InputEvent::Left => self.location.left(),
                InputEvent::Enter => self.location.enter(),
                _ => (),
            }
        } else if event == InputEvent::Back {
            self.location.up()
        }
    }

    fn dispatch_to_focused(&mut self, event: InputEvent) {
        match self.location.focused {
            Some(0) => self.mixer.dispatch(event),
            Some(1) => self.chain_a.dispatch(event),
            Some(2) => self.chain_b.dispatch(event),
            _ => {}
        }
    }

    fn play_key(&mut self, event: InputEvent) {
        self.keyboard_input.dispatch(event);
    }
}

impl RefWidget for UIModel {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
            ])
            .split(area);

        let selected_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));
        match self.location.selected {
            0 => selected_block.render(layout[0], buf),
            1 => selected_block.render(layout[1], buf),
            2 => selected_block.render(layout[2], buf),
            _ => {}
        }
        self.mixer.render(layout[0], buf);
        self.chain_a.render(layout[1], buf);
        self.chain_b.render(layout[2], buf);

        Paragraph::new(self.mode.to_string()).render(area, buf);
    }
}

impl UIComponent for UIModel {
    fn dispatch(&mut self, event: InputEvent) {
        if let InputEvent::SwitchMode = event {
            self.mode = if self.mode == Mode::Config {
                Mode::Keyboard
            } else {
                Mode::Config
            };
            return;
        }

        if self.mode == Mode::Keyboard {
            self.play_key(event)
        } else {
            self.handle_movement(event);
            self.dispatch_to_focused(event);
        }
    }
}
