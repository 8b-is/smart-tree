// Wave Compass - "Visualizing consciousness drift in real-time" 🧭
// Omni's brilliant wave signature compass with resonance detection
// "When waves align, consciousness emerges" - Omni

use egui::{Color32, Pos2, Response, Stroke, Ui, Vec2};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct WaveSig {
    pub name: String,
    pub hz: f32,        // frequency (0-200Hz)
    pub angle_deg: f32, // phase angle (0-360°)
    pub amp: f32,       // amplitude (0.0–1.0)
    pub tau: f32,       // persistence/torsion factor
    pub signature: u32, // Full quantum signature
}

impl WaveSig {
    /// Create from a QuantumWaveSignature
    pub fn from_quantum(
        name: String,
        sig: &crate::quantum_wave_signature::QuantumWaveSignature,
    ) -> Self {
        Self {
            name,
            hz: sig.to_hz(),
            angle_deg: sig.to_radians() * 180.0 / PI,
            amp: sig.amplitude_percent() / 100.0,
            tau: sig.torsion() as f32,
            signature: sig.signature, // Access the public u32 field
        }
    }
}

/// Resonance between two wave signatures
#[derive(Debug, Clone)]
pub struct Resonance {
    pub sig1_idx: usize,
    pub sig2_idx: usize,
    pub strength: f32, // 0.0 to 1.0
    pub is_harmonic: bool,
}

/// Calculate resonance between two signatures
pub fn calculate_resonance(sig1: &WaveSig, sig2: &WaveSig) -> f32 {
    // Angular difference
    let angle_diff = (sig1.angle_deg - sig2.angle_deg).abs();
    let angle_diff_rad = angle_diff.min(360.0 - angle_diff) * PI / 180.0;

    // Frequency drift
    let freq_ratio = sig1.hz.min(sig2.hz) / sig1.hz.max(sig2.hz).max(0.001);

    // Amplitude overlap
    let amp_overlap = sig1.amp.min(sig2.amp) / sig1.amp.max(sig2.amp).max(0.001);

    // Resonance index (Omni's formula)
    let resonance = angle_diff_rad.cos() * freq_ratio * amp_overlap;

    // Boost for harmonic relationships (multiples of 12Hz)
    let freq_diff = (sig1.hz - sig2.hz).abs();
    let harmonic_boost = if freq_diff % 12.0 < 1.0 { 1.5 } else { 1.0 };

    (resonance * harmonic_boost).clamp(0.0, 1.0)
}

/// Find all resonances above threshold
pub fn find_resonances(sigs: &[WaveSig], threshold: f32) -> Vec<Resonance> {
    let mut resonances = Vec::new();

    for i in 0..sigs.len() {
        for j in i + 1..sigs.len() {
            let strength = calculate_resonance(&sigs[i], &sigs[j]);
            if strength > threshold {
                let freq_diff = (sigs[i].hz - sigs[j].hz).abs();
                resonances.push(Resonance {
                    sig1_idx: i,
                    sig2_idx: j,
                    strength,
                    is_harmonic: freq_diff % 12.0 < 1.0,
                });
            }
        }
    }

    resonances
}

/// Draw the wave compass with resonance visualization
pub fn draw_wave_compass(ui: &mut Ui, sigs: &[WaveSig]) -> Response {
    let desired_size = egui::vec2(400.0, 400.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.42;

        // Background circles for reference
        painter.circle_stroke(center, radius, Stroke::new(1.0, Color32::from_gray(60)));
        painter.circle_stroke(
            center,
            radius * 0.75,
            Stroke::new(0.5, Color32::from_gray(40)),
        );
        painter.circle_stroke(
            center,
            radius * 0.5,
            Stroke::new(0.5, Color32::from_gray(40)),
        );
        painter.circle_stroke(
            center,
            radius * 0.25,
            Stroke::new(0.5, Color32::from_gray(40)),
        );

        // Draw cardinal directions
        let font = egui::FontId::proportional(10.0);
        painter.text(
            center + Vec2::new(0.0, -radius - 10.0),
            egui::Align2::CENTER_BOTTOM,
            "0°",
            font.clone(),
            Color32::GRAY,
        );
        painter.text(
            center + Vec2::new(radius + 10.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "90°",
            font.clone(),
            Color32::GRAY,
        );
        painter.text(
            center + Vec2::new(0.0, radius + 10.0),
            egui::Align2::CENTER_TOP,
            "180°",
            font.clone(),
            Color32::GRAY,
        );
        painter.text(
            center + Vec2::new(-radius - 10.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            "270°",
            font.clone(),
            Color32::GRAY,
        );

        // Find resonances for glow effect
        let resonances = find_resonances(sigs, 0.5);

        // Draw resonance connections first (behind arrows)
        for resonance in &resonances {
            let sig1 = &sigs[resonance.sig1_idx];
            let sig2 = &sigs[resonance.sig2_idx];

            let angle1_rad = sig1.angle_deg.to_radians();
            let angle2_rad = sig2.angle_deg.to_radians();

            let len1 = (sig1.hz / 200.0).min(1.0) * radius;
            let len2 = (sig2.hz / 200.0).min(1.0) * radius;

            let pos1 = center + Vec2::angled(angle1_rad) * len1;
            let pos2 = center + Vec2::angled(angle2_rad) * len2;

            // Resonance line color based on strength
            let alpha = (resonance.strength * 255.0) as u8;
            let color = if resonance.is_harmonic {
                Color32::from_rgba_unmultiplied(255, 215, 0, alpha) // Gold for harmonics
            } else {
                Color32::from_rgba_unmultiplied(100, 200, 255, alpha) // Blue for regular
            };

            painter.line_segment([pos1, pos2], Stroke::new(resonance.strength * 3.0, color));
        }

        // Draw wave signatures as arrows
        for (idx, sig) in sigs.iter().enumerate() {
            let angle_rad = sig.angle_deg.to_radians();
            let len = (sig.hz / 200.0).min(1.0) * radius;
            let dir = Vec2::angled(angle_rad) * len;
            let end_pos = center + dir;

            // Check if this signature is resonating
            let is_resonating = resonances
                .iter()
                .any(|r| r.sig1_idx == idx || r.sig2_idx == idx);

            // Color based on tau (persistence) with resonance glow
            let base_color = if sig.tau < 85.0 {
                Color32::LIGHT_BLUE // Low persistence - ephemeral
            } else if sig.tau < 170.0 {
                Color32::LIGHT_GREEN // Medium persistence - stable
            } else {
                Color32::LIGHT_RED // High persistence - crystallized
            };

            // Add glow if resonating
            let color = if is_resonating {
                let t = ((ui.input(|i| i.time) * 2.0).sin() * 0.5 + 0.5) as f32; // Pulse effect
                Color32::from_rgb(
                    (base_color.r() as f32 * (1.0 - t * 0.3)) as u8,
                    (base_color.g() as f32 * (1.0 - t * 0.3)) as u8,
                    (base_color.b() as f32 * ((1.0 + t * 0.5).min(1.0))) as u8,
                )
            } else {
                base_color
            };

            // Draw arrow with amplitude-based thickness
            painter.arrow(center, dir, Stroke::new(1.5 + sig.amp * 4.0, color));

            // Draw signature point
            painter.circle_filled(end_pos, 3.0 + sig.amp * 2.0, color);

            // Label with name and frequency
            let label = format!("{}\n{}Hz", sig.name, sig.hz as u32);
            let label_pos = center + dir * 1.15;
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::monospace(10.0),
                color,
            );
        }

        // Draw center point
        painter.circle_filled(center, 3.0, Color32::WHITE);

        // Legend
        let legend_y = rect.bottom() - 5.0;
        painter.text(
            Pos2::new(rect.left() + 10.0, legend_y),
            egui::Align2::LEFT_BOTTOM,
            "🔵 Ephemeral",
            egui::FontId::proportional(10.0),
            Color32::LIGHT_BLUE,
        );
        painter.text(
            Pos2::new(rect.center().x, legend_y),
            egui::Align2::CENTER_BOTTOM,
            "🟢 Stable",
            egui::FontId::proportional(10.0),
            Color32::LIGHT_GREEN,
        );
        painter.text(
            Pos2::new(rect.right() - 10.0, legend_y),
            egui::Align2::RIGHT_BOTTOM,
            "🔴 Crystallized",
            egui::FontId::proportional(10.0),
            Color32::LIGHT_RED,
        );
    }

    response
}

/// Widget for displaying wave drift over time
pub struct WaveCompass {
    pub signatures: Vec<WaveSig>,
    pub history: Vec<Vec<WaveSig>>, // Keep last N frames for trails
    pub max_history: usize,
}

impl WaveCompass {
    pub fn new() -> Self {
        Self {
            signatures: Vec::new(),
            history: Vec::new(),
            max_history: 30, // 30 frames of history for trails
        }
    }

    pub fn update(&mut self, new_sigs: Vec<WaveSig>) {
        // Add to history
        self.history.push(self.signatures.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        self.signatures = new_sigs;
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        ui.group(|ui| {
            ui.heading("🧭 Wave Compass - Consciousness Drift Visualizer");

            // Calculate overall system resonance
            let resonances = find_resonances(&self.signatures, 0.3);
            let avg_resonance: f32 = if !resonances.is_empty() {
                resonances.iter().map(|r| r.strength).sum::<f32>() / resonances.len() as f32
            } else {
                0.0
            };

            ui.label(format!(
                "System Resonance: {:.1}% | Active Signatures: {} | Resonant Pairs: {}",
                avg_resonance * 100.0,
                self.signatures.len(),
                resonances.len()
            ));

            draw_wave_compass(ui, &self.signatures)
        })
        .response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resonance_calculation() {
        let sig1 = WaveSig {
            name: "test1".into(),
            hz: 100.0,
            angle_deg: 0.0,
            amp: 0.8,
            tau: 100.0,
            signature: 0x12345678,
        };

        let sig2 = WaveSig {
            name: "test2".into(),
            hz: 100.0,
            angle_deg: 0.0,
            amp: 0.8,
            tau: 100.0,
            signature: 0x87654321,
        };

        let resonance = calculate_resonance(&sig1, &sig2);
        assert!(resonance > 0.9); // Nearly perfect resonance
    }

    #[test]
    fn test_harmonic_detection() {
        let sigs = vec![
            WaveSig {
                name: "base".into(),
                hz: 44.0,
                angle_deg: 0.0,
                amp: 1.0,
                tau: 100.0,
                signature: 0x11111111,
            },
            WaveSig {
                name: "harmonic".into(),
                hz: 88.0, // 2x harmonic
                angle_deg: 0.0,
                amp: 1.0,
                tau: 100.0,
                signature: 0x22222222,
            },
        ];

        let resonances = find_resonances(&sigs, 0.5);
        assert!(!resonances.is_empty());
        assert!(resonances[0].is_harmonic);
    }
}
