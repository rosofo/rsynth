use std::sync::mpsc::{self, Sender};

pub struct Config<C> {
    receiver: mpsc::Receiver<C>,
    client: Option<ConfigClient<C>>,
    pub config: C,
}

pub struct ConfigClient<C> {
    sender: Sender<C>,
    current: C,
}

impl<C: Copy> ConfigClient<C> {
    pub fn new(sender: Sender<C>, config: C) -> Self {
        Self {
            sender,
            current: config,
        }
    }

    pub fn update(&mut self, new_config: C) {
        self.sender.send(new_config).unwrap();
        self.current = new_config
    }

    pub fn get(&self) -> C {
        self.current
    }
}

impl<C: Copy> Config<C> {
    pub fn new(config: C) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            config,
            client: Some(ConfigClient::new(sender, config)),
            receiver,
        }
    }

    pub fn try_update(&mut self) {
        for new_config in self.receiver.try_iter() {
            self.config = new_config;
        }
    }

    pub fn get_client(&mut self) -> Option<ConfigClient<C>> {
        std::mem::take(&mut self.client)
    }
}

pub trait ConfigReceiver {
    fn try_update_configs(&mut self);
}

pub trait HasConfig<C> {
    fn get(&self) -> C;
}

pub struct ValidatedConfig<C: Copy, F: Fn(C) -> bool> {
    pub config: Config<C>,
    validator: Option<F>,
}

impl<C: Copy, F: Fn(C) -> bool> ValidatedConfig<C, F> {
    pub fn new(initial: C, validator: F) -> Self {
        Self {
            config: Config::new(initial),
            validator: Some(validator),
        }
    }

    pub fn try_update(&mut self) {
        self.config.try_update();
    }

    pub fn get_client(&mut self) -> Option<ValidatedConfigClient<C, F>> {
        Some(ValidatedConfigClient::new(
            self.config.get_client()?,
            std::mem::take(&mut self.validator)?,
        ))
    }
}

impl<C: Copy, F: Fn(C) -> bool> HasConfig<C> for ValidatedConfig<C, F> {
    fn get(&self) -> C {
        self.config.config
    }
}

pub struct ValidatedConfigClient<C: Copy, F: Fn(C) -> bool> {
    client: ConfigClient<C>,
    validator: F,
}

impl<C: Copy, F: Fn(C) -> bool> ValidatedConfigClient<C, F> {
    pub fn new(client: ConfigClient<C>, validator: F) -> Self {
        Self { client, validator }
    }

    pub fn get(&self) -> C {
        self.client.get()
    }

    pub fn update(&mut self, new_config: C) -> bool {
        if (self.validator)(new_config) {
            self.client.update(new_config);
            true
        } else {
            false
        }
    }
}
