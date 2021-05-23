use std::{
    convert::TryInto,
    marker::PhantomData,
    sync::mpsc::{self, Sender},
};

pub struct Config<C> {
    receiver: mpsc::Receiver<C>,
    client: Option<ConfigClient<C>>,
    pub config: C,
}

pub struct ConfigClient<C> {
    sender: Sender<C>,
    current: C,
}

impl<C: Clone> ConfigClient<C> {
    pub fn new(sender: Sender<C>, config: C) -> Self {
        Self {
            sender,
            current: config,
        }
    }

    pub fn update<F: FnOnce(&mut C)>(&mut self, f: F) {
        f(&mut self.current);
        self.sender.send(self.current.clone()).unwrap();
    }

    pub fn get(&self) -> C {
        self.current.clone()
    }
}

impl<C: Clone> Config<C> {
    pub fn new(config: C) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            config: config.clone(),
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
    fn get<'a>(&'a self) -> &'a C;
}

pub struct ComposeConfig<C, D, F: Fn(C, D) -> C> {
    pub config: Config<C>,
    client: Option<ComposeConfigClient<C, D, F>>,
    _phantom: PhantomData<D>,
}

impl<C: Clone, D, F: Fn(C, D) -> C> ComposeConfig<C, D, F> {
    pub fn new(default: C, f: F) -> Self {
        let mut config = Config::new(default);
        let config_client = config.get_client().unwrap();

        Self {
            config,
            client: Some(ComposeConfigClient::new(f, config_client)),
            _phantom: PhantomData::default(),
        }
    }

    pub fn try_update(&mut self) {
        self.config.try_update()
    }

    pub fn get_client(&mut self) -> Option<ComposeConfigClient<C, D, F>> {
        std::mem::take(&mut self.client)
    }
}

impl<C: Clone> ValidatedConfig<C> {
    pub fn new_validated<V: Fn(&C) -> bool + 'static + Send>(default: C, validator: V) -> Self {
        Self::new(
            default,
            Box::new(move |old, new| if validator(&new) { new } else { old }),
        )
    }
}

pub struct ComposeConfigClient<C, D, F: Fn(C, D) -> C> {
    f: F,
    client: ConfigClient<C>,
    _phantom: PhantomData<D>,
}

impl<C: Clone, D, F: Fn(C, D) -> C> ComposeConfigClient<C, D, F> {
    pub fn new(f: F, client: ConfigClient<C>) -> Self {
        Self {
            f,
            client,
            _phantom: PhantomData::default(),
        }
    }

    pub fn update<G: Fn(&C) -> D>(&mut self, g: G) {
        let x = g(&self.client.current);

        let c = (self.f)(self.client.get(), x);
        self.client.update(|old| *old = c);
    }

    pub fn get(&self) -> C {
        self.client.get()
    }
}

impl<C: Clone, D, F: Fn(C, D) -> C> HasConfig<C> for ComposeConfig<C, D, F> {
    fn get(&self) -> &C {
        &self.config.config
    }
}

pub type ValidatedConfig<C> = ComposeConfig<C, C, Box<dyn Fn(C, C) -> C + 'static + Send>>;
pub type ValidatedConfigClient<C> =
    ComposeConfigClient<C, C, Box<dyn Fn(C, C) -> C + 'static + Send>>;
