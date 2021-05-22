pub mod components;
pub mod input;
pub mod model;

use std::{
    cell::RefCell,
    io::Stdout,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use crossterm::terminal::enable_raw_mode;
use tui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    terminal::CompletedFrame,
    widgets::{BarChart, Block, Borders, Clear, Paragraph, Row, Table, Widget},
    Terminal,
};

use crate::{
    chain::Voice,
    combinators::{TwoChannelClient, TwoChannelConfig},
    config::ValidatedConfigClient,
    synth::Synth,
};

use self::{
    components::{RefWidget, WrapperWidget},
    model::UIModel,
};

pub fn get_terminal() -> std::io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn draw_synth<'a, B: Backend>(
    terminal: &'a mut Terminal<B>,
    ui_model: &Arc<Mutex<UIModel>>,
) -> std::io::Result<CompletedFrame<'a>> {
    let model = ui_model.lock().unwrap();

    terminal.draw(|frame| {
        frame.render_widget(
            WrapperWidget {
                func: |area, buf| model.render(area, buf),
            },
            frame.size(),
        );
    })
}
