use anyhow::Result;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info};

use super::{AudioCapture, AudioCaptureHandle};

const FRAME_SIZE: usize = 512; // ~32ms at 16kHz
const ENERGY_THRESHOLD: f32 = 500.0; // Adjust based on microphone sensitivity
const MIN_SPEECH_DURATION: Duration = Duration::from_millis(200);

pub struct WakeWordDetector {
    audio: Arc<AudioCapture>,
    _handle: AudioCaptureHandle,
    detection_tx: Sender<()>,
    detection_rx: Arc<Mutex<Receiver<()>>>,
    running: Arc<Mutex<bool>>,
}

impl WakeWordDetector {
    pub fn new() -> Result<Self> {
        info!("🎯 Initializing voice activity detection...");

        let (audio, handle) = AudioCapture::new()?;

        let (tx, rx) = channel();

        Ok(Self {
            audio: Arc::new(audio),
            _handle: handle,
            detection_tx: tx,
            detection_rx: Arc::new(Mutex::new(rx)),
            running: Arc::new(Mutex::new(false)),
        })
    }

    fn calculate_energy(samples: &[i16]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum: f64 = samples
            .iter()
            .map(|&s| {
                let normalized = s as f64 / 32768.0;
                normalized * normalized
            })
            .sum();

        ((sum / samples.len() as f64) * 10000.0) as f32
    }

    pub fn start(&self) -> Result<()> {
        info!("👂 Starting voice activity detection...");
        info!("   Clap twice or speak to trigger E.V.A.");

        *self.running.lock().unwrap() = true;

        let audio = Arc::clone(&self.audio);
        let tx = self.detection_tx.clone();
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            let mut speech_start: Option<Instant> = None;

            info!("✓ Voice activity detector running");

            while *running.lock().unwrap() {
                let samples = audio.read_samples(FRAME_SIZE);

                if samples.len() == FRAME_SIZE {
                    let energy = Self::calculate_energy(&samples);

                    if energy > ENERGY_THRESHOLD {
                        if speech_start.is_none() {
                            speech_start = Some(Instant::now());
                        } else if speech_start.unwrap().elapsed() >= MIN_SPEECH_DURATION {
                            info!("🔔 Voice detected! (energy: {:.1})", energy);
                            if tx.send(()).is_err() {
                                error!("Failed to send detection event");
                            }
                            speech_start = None;
                            // Cooldown to avoid multiple triggers
                            thread::sleep(Duration::from_secs(2));
                        }
                    } else {
                        speech_start = None;
                    }
                } else {
                    thread::sleep(Duration::from_millis(10));
                }
            }

            info!("Voice activity detector stopped");
        });

        Ok(())
    }

    pub fn wait_for_detection(&self) -> bool {
        if let Ok(rx) = self.detection_rx.lock() {
            rx.recv().is_ok()
        } else {
            false
        }
    }

    pub fn try_detection(&self) -> Option<()> {
        if let Ok(rx) = self.detection_rx.lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    }

    pub fn stop(&self) {
        info!("Stopping voice activity detector...");
        *self.running.lock().unwrap() = false;
    }
}

impl Drop for WakeWordDetector {
    fn drop(&mut self) {
        self.stop();
    }
}
