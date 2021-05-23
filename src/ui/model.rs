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

pub struct UIModel<C: UIComponent> {
    pub keyboard_input: KeyboardInputComponent,
    pub component: C,
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

impl<C: UIComponent> UIModel<C> {
    pub fn new(keyboard_input: KeyboardInputComponent, component: C) -> Self {
        Self {
            mode: Mode::Keyboard,
            keyboard_input,
            component,
            location: UILocation::default(),
        }
    }

    fn play_key(&mut self, event: InputEvent) {
        self.keyboard_input.dispatch(event);
    }
}

impl<C: UIComponent> RefWidget for UIModel<C> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        self.component.render(area, buf);
        Paragraph::new(self.mode.to_string()).render(area, buf);
    }
}

impl<C: UIComponent> UIComponent for UIModel<C> {
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
            self.component.dispatch(event)
        }
    }
}
