use std::sync::mpsc;

pub struct Config<C> {
    pub sender: mpsc::Sender<C>,
    receiver: mpsc::Receiver<C>,
    pub config: C,
}

impl<C> Config<C> {
    pub fn new(config: C) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            config,
            sender,
            receiver,
        }
    }

    pub fn try_update(&mut self) {
        if let Ok(new_config) = self.receiver.try_recv() {
            self.config = new_config;
        }
    }
}

pub trait ConfigReceiver {
    fn try_update_configs(&mut self);
}
