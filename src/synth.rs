use std::sync::{Arc, Mutex};

use cpal::{traits::StreamTrait, Stream};

use crate::{
    audio::Audio,
    chain::{Chain, Voice},
    voices::Sine,
};

pub struct Synth<V: Voice<f32> + Send + 'static> {
    pub voice: Arc<Mutex<V>>,
    audio: Audio,
    stream: Option<Stream>,
}

impl<V: Voice<f32> + Send> Synth<V> {
    pub fn new(voice: V) -> Self {
        Self {
            voice: Arc::new(Mutex::new(voice)),
            audio: Audio::new(),
            stream: None,
        }
    }

    pub fn play(&mut self) {
        if self.stream.is_none() {
            let voice = Arc::clone(&self.voice);
            let stream = self
                .audio
                .stream_with(move |data: &mut [f32]| put_samples(Arc::clone(&voice), data));
            stream.play().unwrap();
            self.stream = Some(stream);
        }
    }
}

fn put_samples<V: Voice<f32>>(voice: Arc<Mutex<V>>, data: &mut [f32]) {
    for slot in data {
        *slot = voice.lock().unwrap().generate();
    }
}
