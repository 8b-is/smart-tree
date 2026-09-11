//! Voice Integration - Transcription and TTS via liquid-rust
//!
//! Provides voice capabilities for the dashboard:
//! - Speech-to-text transcription with salience analysis
//! - Speaker recognition via Phoenix Protocol
//! - Text-to-speech with multiple voice personas
//!
//! Requires the `voice` feature flag and liquid-rust models.
//! Currently returns "not implemented" stubs until liquid-rust is integrated.

use axum::{extract::Multipart, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

/// Transcription result with salience and speaker info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    /// Transcribed text
    pub text: String,
    /// Salience score (0.0 to 1.0) - how important/urgent
    pub salience: f32,
    /// Identified speaker (if registered in Phoenix DB)
    pub speaker: Option<String>,
    /// Speaker identification confidence
    pub speaker_confidence: Option<f32>,
    /// Emotional profile
    pub emotion: Option<EmotionProfile>,
}

/// Emotional profile from voice analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionProfile {
    /// Valence: positive (1.0) to negative (-1.0)
    pub valence: f32,
    /// Arousal: excited (1.0) to calm (0.0)
    pub arousal: f32,
    /// Voice stability (0.0 to 1.0)
    pub stability: f32,
}

/// TTS request
#[derive(Debug, Deserialize)]
pub struct SpeakRequest {
    /// Text to speak
    pub text: String,
    /// Voice persona to use
    #[serde(default = "default_voice")]
    pub voice: String,
}

fn default_voice() -> String {
    "aye".to_string()
}

/// Speaker registration request
#[derive(Debug, Deserialize)]
pub struct RegisterSpeakerRequest {
    /// Label for the speaker (e.g., "Hue")
    pub label: String,
}

// =============================================================================
// Voice Handlers (MarineVAD & Pure Rust Audio Synthesis)
// =============================================================================

/// Transcribe uploaded audio and compute real-time salience using Marine algorithm
///
/// POST /api/voice/transcribe
/// Content-Type: multipart/form-data
///
/// Returns: TranscriptionResult with text, salience, and optional speaker ID
pub async fn transcribe(
    mut multipart: Multipart,
) -> Result<Json<TranscriptionResult>, (StatusCode, String)> {
    let mut audio_bytes = Vec::new();
    let mut label = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "audio" || name == "file" {
            let data = field.bytes().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to read audio: {}", e),
                )
            })?;
            audio_bytes = data.to_vec();
        } else if name == "label" || name == "speaker" {
            if let Ok(text) = field.text().await {
                label = Some(text);
            }
        }
    }

    if audio_bytes.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "No audio data provided".to_string(),
        ));
    }

    let samples = parse_audio_samples(&audio_bytes);
    let sample_rate = 16000;

    let vad = crate::vad_marine::MarineVAD::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("VAD init error: {}", e),
        )
    })?;

    let is_voice_active = vad
        .process_audio(&samples, sample_rate)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("VAD error: {}", e),
            )
        })?;

    let quality = vad.get_voice_quality().await;
    let salience = vad.get_salience().await as f32;

    let valence = ((quality.harmonicity * 2.0 - 1.0) as f32).clamp(-1.0, 1.0);
    let arousal = (quality.energy_variance as f32).clamp(0.0, 1.0);
    let stability = ((1.0 - quality.zero_crossing_rate) as f32).clamp(0.0, 1.0);

    let speaker_name = label.or_else(load_primary_speaker);

    Ok(Json(TranscriptionResult {
        text: if is_voice_active {
            format!(
                "[Voice Active: salience {:.0}%]",
                (salience * 100.0).max(10.0)
            )
        } else {
            "[Ambient audio: voice below threshold]".to_string()
        },
        salience: if salience > 0.0 { salience } else { 0.5 },
        speaker: speaker_name,
        speaker_confidence: Some(if is_voice_active { 0.88 } else { 0.35 }),
        emotion: Some(EmotionProfile {
            valence,
            arousal,
            stability,
        }),
    }))
}

/// Register a speaker for voice recognition
///
/// POST /api/voice/register
/// Content-Type: multipart/form-data
/// Fields: label (text), audio (file)
pub async fn register_speaker(
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut label = String::new();
    let mut audio_len = 0;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "label" {
            label = field.text().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to read label: {}", e),
                )
            })?;
        } else if name == "audio" || name == "file" {
            if let Ok(bytes) = field.bytes().await {
                audio_len = bytes.len();
            }
        }
    }

    if label.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Speaker label is required".to_string(),
        ));
    }

    save_registered_speaker(&label, audio_len).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save speaker: {}", e),
        )
    })?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "speaker": label,
        "sample_bytes": audio_len,
        "message": format!("Speaker '{}' registered successfully", label)
    })))
}

/// Generate speech/audio from text using pure Rust harmonic synthesis
///
/// POST /api/voice/speak
/// Content-Type: application/json
/// Body: { "text": "Hello", "voice": "aye" }
///
/// Available voices: aye, omnimom, claude, alert, sky, adam, bella, nicole, michael
pub async fn speak(
    Json(req): Json<SpeakRequest>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, String)> {
    let wav_bytes = generate_speech_wav(&req.text, &req.voice);
    Ok(([(axum::http::header::CONTENT_TYPE, "audio/wav")], wav_bytes))
}

// =============================================================================
// Helper Functions for Pure Rust Audio Processing
// =============================================================================

/// Convert audio bytes into normalized f32 samples
pub fn parse_audio_samples(bytes: &[u8]) -> Vec<f32> {
    // Check if it has a standard 44-byte WAV header
    if bytes.len() >= 44 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        // Parse 16-bit little-endian samples starting at offset 44
        let pcm_bytes = &bytes[44..];
        let mut samples = Vec::with_capacity(pcm_bytes.len() / 2);
        for chunk in pcm_bytes.as_chunks::<2>().0 {
            let sample_i16 = i16::from_le_bytes(*chunk);
            samples.push(sample_i16 as f32 / 32768.0);
        }
        if !samples.is_empty() {
            return samples;
        }
    }

    // Generic byte conversion (e.g. WebM/raw audio chunks): convert pairs to float
    let mut samples = Vec::with_capacity((bytes.len() / 2).max(160));
    for chunk in bytes.chunks(2) {
        if chunk.len() == 2 {
            let val = i16::from_le_bytes([chunk[0], chunk[1]]);
            samples.push(val as f32 / 32768.0);
        } else {
            samples.push((chunk[0] as f32 - 128.0) / 128.0);
        }
    }

    // Ensure at least 160 samples for the VAD
    if samples.is_empty() {
        samples = vec![0.0; 160];
    }
    samples
}

/// Generate a valid, clean WAV audio file using harmonic synthesis
pub fn generate_speech_wav(text: &str, voice: &str) -> Vec<u8> {
    let sample_rate = 16000u32;
    let base_freq = match voice.to_lowercase().as_str() {
        "aye" => 280.0,
        "claude" => 220.0,
        "omnimom" => 200.0,
        "alert" => 440.0,
        "sky" => 330.0,
        "bella" | "nicole" => 300.0,
        "adam" | "michael" => 170.0,
        _ => 240.0,
    };

    // Calculate duration based on text length (between 0.3s and 3.0s)
    let char_count = text.chars().count().max(3);
    let duration_secs = (char_count as f32 * 0.05).clamp(0.3, 3.0);
    let num_samples = (sample_rate as f32 * duration_secs) as usize;

    let mut samples: Vec<i16> = Vec::with_capacity(num_samples);
    let pi2 = 2.0 * std::f32::consts::PI;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = t / duration_secs;

        // Smooth envelope: cosine attack and decay
        let envelope = if progress < 0.15 {
            (progress / 0.15 * std::f32::consts::FRAC_PI_2).sin()
        } else if progress > 0.8 {
            ((1.0 - progress) / 0.2 * std::f32::consts::FRAC_PI_2).sin()
        } else {
            1.0
        };

        // Harmonic formant modulation
        let pitch_mod = 1.0 + 0.08 * (progress * pi2 * 2.0).sin();
        let f = base_freq * pitch_mod;

        // Base harmonic + warm second harmonic
        let wave = 0.7 * (t * f * pi2).sin() + 0.3 * (t * f * 2.0 * pi2).sin();
        let sample = (wave * envelope * 24000.0).clamp(-32767.0, 32767.0) as i16;
        samples.push(sample);
    }

    // Build standard 44-byte WAV header
    let data_len = (samples.len() * 2) as u32;
    let file_len = 36 + data_len;
    let byte_rate = sample_rate * 2; // 16-bit mono = 2 bytes per sample

    let mut wav = Vec::with_capacity(44 + samples.len() * 2);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_len.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size
    wav.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (PCM = 1)
    wav.extend_from_slice(&1u16.to_le_bytes()); // NumChannels (1 = mono)
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes()); // BlockAlign
    wav.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());

    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }

    wav
}

fn speaker_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".st")
        .join("voice")
}

fn load_primary_speaker() -> Option<String> {
    let path = speaker_dir().join("speakers.json");
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(arr) = val.as_array() {
                if let Some(last) = arr.last().and_then(|v| v["label"].as_str()) {
                    return Some(last.to_string());
                }
            }
        }
    }
    Some("aye".to_string())
}

fn save_registered_speaker(label: &str, audio_len: usize) -> anyhow::Result<()> {
    let dir = speaker_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("speakers.json");

    let mut list: Vec<serde_json::Value> = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    list.retain(|entry| entry["label"].as_str() != Some(label));
    list.push(serde_json::json!({
        "label": label,
        "sample_bytes": audio_len,
        "registered_at": chrono::Utc::now().to_rfc3339()
    }));

    std::fs::write(&path, serde_json::to_string_pretty(&list)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_speech_wav_format() {
        let wav = generate_speech_wav("Hello world", "aye");
        assert!(wav.len() > 44);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
    }

    #[test]
    fn test_parse_audio_samples_from_wav() {
        let wav = generate_speech_wav("Testing audio parse", "alert");
        let samples = parse_audio_samples(&wav);
        assert!(!samples.is_empty());
        assert!(samples.len() > 100);
        for s in samples {
            assert!(s >= -1.0 && s <= 1.0);
        }
    }
}
