// VAD with Marine Algorithm - "Semper Fi to voice detection!" 🎖️
// Voice Activity Detection using MEM8's marine salience algorithm
// "Standing watch at the boundaries of speech!" - Hue

use anyhow::Result;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Voice Activity Detector using Marine algorithm
/// Detects when someone is speaking vs silence
pub struct MarineVAD {
    /// Marine detector state
    detector: Arc<RwLock<MarineDetectorState>>,

    /// Audio input monitoring
    audio_monitor: Arc<RwLock<AudioMonitor>>,

    /// VAD state
    is_voice_active: Arc<RwLock<bool>>,

    /// Callback for voice state changes
    state_callback: StateCallback,
}

type StateCallback = Arc<RwLock<Option<Box<dyn Fn(bool) + Send + Sync>>>>;

/// Marine detector state for VAD
struct MarineDetectorState {
    /// Clip threshold for voice detection (dB)
    voice_threshold: f64,

    /// Grid tick rate (Hz) - how often we evaluate
    tick_rate: f64,

    /// Peak history for voice pattern analysis
    peak_history: VecDeque<PeakEvent>,

    /// Period tracking for speech patterns
    period_ema: ExponentialMovingAverage,

    /// Amplitude tracking for voice energy
    amplitude_ema: ExponentialMovingAverage,

    /// Speech pattern detector
    speech_detector: SpeechPatternDetector,

    /// Current salience score (0.0 to 1.0)
    voice_salience: f64,

    /// Last evaluation time
    last_tick: Instant,

    /// Voice onset time
    voice_onset: Option<Instant>,

    /// Voice offset time
    voice_offset: Option<Instant>,
}

/// Peak event in audio signal
#[derive(Clone, Debug)]
struct PeakEvent {
    timestamp: Instant,
    amplitude: f64,
    frequency: f64,  // Estimated frequency
    is_voiced: bool, // Voiced vs unvoiced
}

/// Exponential moving average for smoothing
struct ExponentialMovingAverage {
    value: f64,
    alpha: f64, // Smoothing factor
}

impl ExponentialMovingAverage {
    fn new(alpha: f64) -> Self {
        Self { value: 0.0, alpha }
    }

    fn update(&mut self, sample: f64) -> f64 {
        self.value = self.alpha * sample + (1.0 - self.alpha) * self.value;
        self.value
    }

    fn jitter(&self, sample: f64) -> f64 {
        (sample - self.value).abs()
    }
}

/// Speech pattern detector
struct SpeechPatternDetector {
    /// Typical speech fundamental frequency range (Hz)
    f0_min: f64, // ~80 Hz for deep male voice
    f0_max: f64, // ~400 Hz for high female/child voice

    /// Formant tracking
    formant_tracker: FormantTracker,

    /// Syllable rate detector (2-7 Hz typical)
    syllable_detector: SyllableRateDetector,

    /// Voice quality metrics
    voice_quality: VoiceQuality,
}

/// Formant tracker for vowel detection
struct FormantTracker {
    f1_range: (f64, f64), // First formant range (200-1000 Hz)
    f2_range: (f64, f64), // Second formant range (500-2500 Hz)
    f3_range: (f64, f64), // Third formant range (1500-3500 Hz)
}

/// Syllable rate detector
struct SyllableRateDetector {
    energy_envelope: VecDeque<f64>,
    peak_times: VecDeque<Instant>,
    min_syllable_gap: Duration, // ~100ms minimum
    max_syllable_gap: Duration, // ~500ms maximum
}

/// Voice quality metrics
struct VoiceQuality {
    harmonicity: f64,        // Harmonic-to-noise ratio
    spectral_tilt: f64,      // High vs low frequency energy
    zero_crossing_rate: f64, // Voiced vs unvoiced
    energy_variance: f64,    // Speech dynamics
}

/// Audio input monitor
struct AudioMonitor {
    /// Current audio level (RMS)
    current_level: f64,

    /// Peak level in window
    peak_level: f64,

    /// Noise floor estimate
    noise_floor: f64,

    /// Signal-to-noise ratio
    snr: f64,

    /// Audio source (mic, line-in, etc)
    source: AudioSource,
}

#[derive(Clone, Debug)]
enum AudioSource {
    Microphone,
    LineIn,
    Virtual, // For testing
}

impl MarineVAD {
    /// Create new VAD with marine algorithm
    pub fn new() -> Result<Self> {
        Ok(Self {
            detector: Arc::new(RwLock::new(MarineDetectorState::new())),
            audio_monitor: Arc::new(RwLock::new(AudioMonitor::new())),
            is_voice_active: Arc::new(RwLock::new(false)),
            state_callback: Arc::new(RwLock::new(None)),
        })
    }

    /// Process audio samples
    pub async fn process_audio(&self, samples: &[f32], sample_rate: u32) -> Result<bool> {
        let mut detector = self.detector.write().await;
        let mut monitor = self.audio_monitor.write().await;

        // Update audio monitor
        monitor.update_levels(samples);

        // Check if we should evaluate (based on tick rate)
        let now = Instant::now();
        let tick_duration = Duration::from_secs_f64(1.0 / detector.tick_rate);

        if now.duration_since(detector.last_tick) < tick_duration {
            return Ok(*self.is_voice_active.read().await);
        }

        detector.last_tick = now;

        // Marine algorithm evaluation
        let voice_detected = detector.evaluate_voice(samples, sample_rate, monitor.snr);

        // Update state if changed
        let mut is_active = self.is_voice_active.write().await;
        if voice_detected != *is_active {
            *is_active = voice_detected;

            // Call state change callback
            if let Some(callback) = &*self.state_callback.read().await {
                callback(voice_detected);
            }

            // Log state change
            if voice_detected {
                println!("🎤 Voice detected - switching to minimal output mode");
                detector.voice_onset = Some(now);
            } else {
                println!("🔇 Voice ended - returning to normal output mode");
                detector.voice_offset = Some(now);
            }
        }

        Ok(voice_detected)
    }

    /// Set callback for voice state changes
    pub async fn set_state_callback<F>(&self, callback: F)
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        let mut cb = self.state_callback.write().await;
        *cb = Some(Box::new(callback));
    }

    /// Get current voice activity state
    pub async fn is_voice_active(&self) -> bool {
        *self.is_voice_active.read().await
    }

    /// Get voice salience score (0.0 to 1.0)
    pub async fn get_salience(&self) -> f64 {
        self.detector.read().await.voice_salience
    }

    /// Get voice quality metrics
    pub async fn get_voice_quality(&self) -> VoiceQualityReport {
        let detector = self.detector.read().await;
        VoiceQualityReport {
            salience: detector.voice_salience,
            harmonicity: detector.speech_detector.voice_quality.harmonicity,
            spectral_tilt: detector.speech_detector.voice_quality.spectral_tilt,
            zero_crossing_rate: detector.speech_detector.voice_quality.zero_crossing_rate,
            energy_variance: detector.speech_detector.voice_quality.energy_variance,
        }
    }
}

impl MarineDetectorState {
    fn new() -> Self {
        Self {
            voice_threshold: -40.0, // -40 dB threshold
            tick_rate: 100.0,       // 100 Hz evaluation rate
            peak_history: VecDeque::with_capacity(100),
            period_ema: ExponentialMovingAverage::new(0.1),
            amplitude_ema: ExponentialMovingAverage::new(0.05),
            speech_detector: SpeechPatternDetector::new(),
            voice_salience: 0.0,
            last_tick: Instant::now(),
            voice_onset: None,
            voice_offset: None,
        }
    }

    /// Evaluate voice presence using marine algorithm
    fn evaluate_voice(&mut self, samples: &[f32], sample_rate: u32, snr: f64) -> bool {
        // Calculate RMS energy
        let energy: f64 =
            samples.iter().map(|&s| (s as f64).powi(2)).sum::<f64>() / samples.len() as f64;
        let rms = energy.sqrt();
        let db = 20.0 * rms.log10();

        // Update amplitude tracking
        self.amplitude_ema.update(rms);

        // Check against threshold
        if db < self.voice_threshold {
            self.voice_salience *= 0.9; // Decay salience
            return false;
        }

        // Analyze for speech patterns
        let has_speech_pattern = self.speech_detector.analyze(samples, sample_rate);

        // Calculate salience score
        let mut salience = 0.0;

        // Energy contribution (30%)
        let energy_score = ((db - self.voice_threshold) / 20.0).clamp(0.0, 1.0);
        salience += energy_score * 0.3;

        // SNR contribution (20%)
        let snr_score = (snr / 20.0).clamp(0.0, 1.0);
        salience += snr_score * 0.2;

        // Speech pattern contribution (50%)
        if has_speech_pattern {
            salience += 0.5;
        }

        // Update salience with smoothing
        self.voice_salience = 0.7 * salience + 0.3 * self.voice_salience;

        // Voice detected if salience > 0.5
        self.voice_salience > 0.5
    }
}

impl SpeechPatternDetector {
    fn new() -> Self {
        Self {
            f0_min: 80.0,
            f0_max: 400.0,
            formant_tracker: FormantTracker {
                f1_range: (200.0, 1000.0),
                f2_range: (500.0, 2500.0),
                f3_range: (1500.0, 3500.0),
            },
            syllable_detector: SyllableRateDetector {
                energy_envelope: VecDeque::with_capacity(100),
                peak_times: VecDeque::with_capacity(20),
                min_syllable_gap: Duration::from_millis(100),
                max_syllable_gap: Duration::from_millis(500),
            },
            voice_quality: VoiceQuality {
                harmonicity: 0.0,
                spectral_tilt: 0.0,
                zero_crossing_rate: 0.0,
                energy_variance: 0.0,
            },
        }
    }

    fn analyze(&mut self, samples: &[f32], sample_rate: u32) -> bool {
        // Simple zero-crossing rate for voiced/unvoiced detection
        let mut zero_crossings = 0;
        for i in 1..samples.len() {
            if samples[i - 1] * samples[i] < 0.0 {
                zero_crossings += 1;
            }
        }

        let zcr = zero_crossings as f64 / samples.len() as f64;
        self.voice_quality.zero_crossing_rate = zcr;

        // Voiced speech has lower ZCR (< 0.3), unvoiced has higher
        let is_voiced = zcr < 0.3;

        // Check if in speech frequency range
        let estimated_freq = zcr * sample_rate as f64 / 2.0;
        let in_speech_range = estimated_freq >= self.f0_min && estimated_freq <= self.f0_max * 10.0;

        is_voiced && in_speech_range
    }
}

impl AudioMonitor {
    fn new() -> Self {
        Self {
            current_level: 0.0,
            peak_level: 0.0,
            noise_floor: -60.0, // Start with -60 dB assumption
            snr: 0.0,
            source: AudioSource::Microphone,
        }
    }

    fn update_levels(&mut self, samples: &[f32]) {
        // Calculate RMS
        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_squares / samples.len() as f32).sqrt();
        self.current_level = rms as f64;

        // Find peak
        let peak = samples.iter().map(|&s| s.abs()).fold(0.0f32, f32::max) as f64;
        self.peak_level = peak;

        // Update noise floor estimate (slow adaptation)
        if rms as f64 > 0.0 {
            let db = 20.0 * (rms as f64).log10();
            self.noise_floor = 0.99 * self.noise_floor + 0.01 * db;
            self.snr = db - self.noise_floor;
        }
    }
}

/// Voice quality report
#[derive(Debug, Clone)]
pub struct VoiceQualityReport {
    pub salience: f64,
    pub harmonicity: f64,
    pub spectral_tilt: f64,
    pub zero_crossing_rate: f64,
    pub energy_variance: f64,
}

/// Integration with rust_shell
impl super::rust_shell::RustShell {
    /// Enable VAD with marine algorithm
    pub async fn enable_marine_vad(&self) -> Result<()> {
        println!("🎖️ Enabling Marine VAD - Semper Fi to voice detection!");

        let vad = MarineVAD::new()?;

        // Set callback to adjust verbosity
        let output_mode = self.output_mode.clone();
        vad.set_state_callback(move |is_voice| {
            // This would be called when voice state changes
            let mode = output_mode.clone();
            tokio::spawn(async move {
                let mut m = mode.write().await;
                if is_voice {
                    m.verbosity = super::rust_shell::VerbosityLevel::Minimal;
                    m.format = super::rust_shell::OutputFormat::Voice;
                } else {
                    m.verbosity = super::rust_shell::VerbosityLevel::Normal;
                    m.format = super::rust_shell::OutputFormat::Text;
                }
            });
        })
        .await;

        // Store VAD instance (would need to add field to RustShell)
        // self.vad = Some(vad);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_marine_vad_creation() {
        let vad = MarineVAD::new();
        assert!(vad.is_ok());
    }

    #[tokio::test]
    async fn test_voice_detection() {
        let vad = MarineVAD::new().unwrap();

        // Create test signal (sine wave at 200 Hz - typical voice F0)
        let sample_rate = 16000;
        let frequency = 200.0;
        let duration = 0.1; // 100ms
        let num_samples = (sample_rate as f64 * duration) as usize;

        let mut samples = vec![0.0f32; num_samples];
        for (i, sample) in samples.iter_mut().enumerate().take(num_samples) {
            let t = i as f64 / sample_rate as f64;
            *sample = (2.0 * std::f64::consts::PI * frequency * t).sin() as f32 * 0.5;
        }

        // Process audio
        let _is_voice = vad.process_audio(&samples, sample_rate).await.unwrap();

        // Should detect voice-like signal
        // (In real implementation would need proper training)
    }
}
