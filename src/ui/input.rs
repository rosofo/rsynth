use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    Left,
    Right,
    Up,
    Down,
    Back,
    Enter,
    Replace,
    SwitchMode,
    Unmapped(KeyCode),
}

pub fn parse_input_event(event: crossterm::event::Event) -> Option<InputEvent> {
    let parse_from_key_code = |key_code: KeyCode| match key_code {
        KeyCode::Char('h') => Some(InputEvent::Left),
        KeyCode::Char('l') => Some(InputEvent::Right),
        KeyCode::Char('j') => Some(InputEvent::Down),
        KeyCode::Char('k') => Some(InputEvent::Up),
        KeyCode::Enter => Some(InputEvent::Enter),
        KeyCode::Char('r') => Some(InputEvent::Replace),
        KeyCode::Char('b') => Some(InputEvent::Back),
        KeyCode::Tab => Some(InputEvent::SwitchMode),
        key_code => Some(InputEvent::Unmapped(key_code)),
    };

    match event {
        Event::Key(key_event) => parse_from_key_code(key_event.code),
        _ => None,
    }
}
