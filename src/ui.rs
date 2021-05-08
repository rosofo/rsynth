use std::{cell::RefCell, io::Stdout, rc::Rc};

use tui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::Rect,
    terminal::CompletedFrame,
    widgets::{Paragraph, Widget},
    Terminal,
};

use crate::{
    chain::Voice,
    combinators::{TwoChannelClient, TwoChannelConfig},
    config::ValidatedConfigClient,
    synth::Synth,
};

pub fn get_terminal() -> std::io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn draw_synth<'a, B: Backend>(
    terminal: &'a mut Terminal<B>,
    two_channel_client: &TwoChannelClient,
) -> std::io::Result<CompletedFrame<'a>> {
    let synth_config_widget = SynthConfigWidget {
        two_channel_widget: TwoChannelWidget {
            client: two_channel_client,
        },
    };

    terminal.draw(|frame| {
        frame.render_widget(synth_config_widget, frame.size());
    })
}

pub struct SynthConfigWidget<'a> {
    pub two_channel_widget: TwoChannelWidget<'a>,
}

impl<'a> Widget for SynthConfigWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.two_channel_widget.render(area, buf);
    }
}

pub struct TwoChannelWidget<'a> {
    pub client: &'a TwoChannelClient,
}

impl<'a> Widget for TwoChannelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let TwoChannelConfig { a_mix, b_mix } = self.client.get();

        let p = Paragraph::new(format!("a: {}; b: {}", a_mix, b_mix));
        p.render(area, buf);
    }
}
