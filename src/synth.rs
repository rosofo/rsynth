use std::{
    borrow::BorrowMut,
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex},
};

use cpal::{traits::StreamTrait, Stream};

use crate::{
    audio::Audio,
    chain::{Chain, Voice},
    voices::Sine,
};

pub struct Synth<V: Voice<f32> + Send + 'static> {
    audio: Audio,
    stream: Option<Stream>,
    _phantom: PhantomData<V>,
}

impl<V: Voice<f32> + Send> Synth<V> {
    pub fn new() -> Self {
        Self {
            audio: Audio::new(),
            stream: None,
            _phantom: PhantomData,
        }
    }

    pub fn play(&mut self, mut voice: V) {
        if self.stream.is_none() {
            let stream = self.audio.stream_with(move |data: &mut [f32]| {
                voice.try_update_configs();
                put_samples(&mut voice, data);
            });
            stream.play().unwrap();
            self.stream = Some(stream);
        }
    }
}

fn put_samples<V: Voice<f32>>(voice: &mut V, data: &mut [f32]) {
    for slot in data {
        *slot = voice.generate();
    }
}
