use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig};
use ringbuf::{HeapRb, Rb};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

const SAMPLE_RATE: u32 = 16000; // 16kHz for voice
const BUFFER_SIZE: usize = 512 * 60; // ~2 seconds of audio

pub struct AudioCapture {
    buffer: Arc<Mutex<HeapRb<i16>>>,
    sample_rate: u32,
}

pub struct AudioCaptureHandle {
    _host: Host,
    _device: Device,
    _config: StreamConfig,
    _stream: Stream,
}

impl AudioCapture {
    pub fn new() -> Result<(Self, AudioCaptureHandle)> {
        info!("🎤 Initializing audio capture...");

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No input device available")?;

        info!("   Using device: {}", device.name()?);

        let config = StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer = Arc::new(Mutex::new(HeapRb::<i16>::new(BUFFER_SIZE)));
        let buffer_clone = Arc::clone(&buffer);

        let err_fn = |err| error!("Audio stream error: {}", err);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Convert f32 samples to i16
                let samples: Vec<i16> = data
                    .iter()
                    .map(|&sample| (sample * 32767.0).clamp(-32768.0, 32767.0) as i16)
                    .collect();

                if let Ok(mut buf) = buffer_clone.lock() {
                    for &sample in &samples {
                        buf.push_overwrite(sample);
                    }
                }
            },
            err_fn,
            None,
        )?;

        stream.play()?;

        info!("✓ Audio capture started");

        let capture = Self {
            buffer,
            sample_rate: SAMPLE_RATE,
        };

        let handle = AudioCaptureHandle {
            _host: host,
            _device: device,
            _config: config,
            _stream: stream,
        };

        Ok((capture, handle))
    }

    pub fn read_samples(&self, count: usize) -> Vec<i16> {
        if let Ok(mut buffer) = self.buffer.lock() {
            let mut samples = Vec::with_capacity(count);
            for _ in 0..count.min(buffer.occupied_len()) {
                if let Some(sample) = buffer.pop() {
                    samples.push(sample);
                }
            }
            samples
        } else {
            Vec::new()
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
