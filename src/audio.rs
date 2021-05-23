use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Sample, SampleFormat, Stream, StreamConfig, SupportedOutputConfigs,
    SupportedStreamConfigRange,
};

pub struct Audio {
    device: Device,
    config: StreamConfig,
    format: SampleFormat,
}

impl Audio {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.supported_output_configs().unwrap().next().unwrap();
        let format = config.sample_format();
        let config = config.with_max_sample_rate().config();

        Audio {
            device,
            config,
            format,
        }
    }
    pub fn stream_with<F>(&self, f: F) -> Stream
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        match self.format {
            SampleFormat::F32 => return self.stream_audio_with::<F, f32>(f),
            _ => panic!(""),
        };
    }

    fn stream_audio_with<F, T>(&self, mut f: F) -> Stream
    where
        F: FnMut(&mut [T]) + Send + 'static,
        T: Sample,
    {
        let data_callback = move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            f(data);
        };
        let error_callback = move |_err| {};

        self.device
            .build_output_stream(&self.config, data_callback, error_callback)
            .unwrap()
    }
}
