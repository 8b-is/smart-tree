use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;

// ============================================================================
// Card 1: Ternary Lane Packing (4-Trits/Byte Bitwise vs. 5-Trits/Byte Base-3)
// ============================================================================

/// Compile-time lookup table mapping every valid 5-trit base-3 byte `b in 0..243`
/// directly to `[t_0, t_1, t_2, t_3, t_4]`, eliminating runtime division/modulo.
pub const UNPACK_5TRITS_LUT: [[u8; 5]; 243] = build_5trit_unpack_lut();

const fn build_5trit_unpack_lut() -> [[u8; 5]; 243] {
    let mut lut = [[0u8; 5]; 243];
    let mut b = 0usize;
    while b < 243 {
        let mut rem = b as u8;
        let mut i = 0;
        while i < 5 {
            lut[b][i] = rem % 3;
            rem /= 3;
            i += 1;
        }
        b += 1;
    }
    lut
}

/// Packs 4 trits `t_i in {0, 1, 2}` into a single byte using 2 bits per trit (little-endian).
#[inline(always)]
pub const fn pack_4trits_byte(t: [u8; 4]) -> u8 {
    (t[0] & 0b11) | ((t[1] & 0b11) << 2) | ((t[2] & 0b11) << 4) | ((t[3] & 0b11) << 6)
}

/// Unpacks 4 trits from a 2-bit-per-trit byte using bit-shifts.
#[inline(always)]
pub const fn unpack_4trits_byte(b: u8) -> [u8; 4] {
    [b & 0b11, (b >> 2) & 0b11, (b >> 4) & 0b11, (b >> 6) & 0b11]
}

/// Packs 5 trits `t_i in {0, 1, 2}` into a single byte via `b = \sum_{i=0}^4 t_i * 3^i` (max 242 < 256).
#[inline(always)]
pub const fn pack_5trits_byte(t: [u8; 5]) -> u8 {
    t[0] + 3 * t[1] + 9 * t[2] + 27 * t[3] + 81 * t[4]
}

/// Unpacks 5 trits `t_i = floor(b / 3^i) mod 3` in O(1) via the const LUT.
#[inline(always)]
pub const fn unpack_5trits_byte(b: u8) -> [u8; 5] {
    if (b as usize) < 243 {
        UNPACK_5TRITS_LUT[b as usize]
    } else {
        [0; 5]
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TernaryLaneReport {
    pub width: usize,
    pub height: usize,
    pub shannon_bits_per_trit: f64,
    pub four_trits_per_byte: PackingSchemeMetrics,
    pub five_trits_per_byte: PackingSchemeMetrics,
    pub frame_bytes_saved_by_5trit: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackingSchemeMetrics {
    pub trits_per_byte: usize,
    pub bits_stored_per_trit: f64,
    pub coding_efficiency_ratio: f64,
    pub row_bytes: usize,
    pub frame_bytes: usize,
}

pub fn evaluate_ternary_lane(width: usize, height: usize) -> TernaryLaneReport {
    let shannon_bits = 3.0_f64.log2();

    let row_bytes_4 = width.div_ceil(4);
    let frame_bytes_4 = height * row_bytes_4;

    let row_bytes_5 = width.div_ceil(5);
    let frame_bytes_5 = height * row_bytes_5;

    TernaryLaneReport {
        width,
        height,
        shannon_bits_per_trit: shannon_bits,
        four_trits_per_byte: PackingSchemeMetrics {
            trits_per_byte: 4,
            bits_stored_per_trit: 2.0,
            coding_efficiency_ratio: shannon_bits / 2.0,
            row_bytes: row_bytes_4,
            frame_bytes: frame_bytes_4,
        },
        five_trits_per_byte: PackingSchemeMetrics {
            trits_per_byte: 5,
            bits_stored_per_trit: 1.6,
            coding_efficiency_ratio: shannon_bits / 1.6,
            row_bytes: row_bytes_5,
            frame_bytes: frame_bytes_5,
        },
        frame_bytes_saved_by_5trit: frame_bytes_4.saturating_sub(frame_bytes_5),
    }
}

// ============================================================================
// Card 2: Drift-Free Integer Frame Timing (t_n = floor(1000*n / F) & F*r <= 10^9)
// ============================================================================

/// Computes the exact nominal millisecond timestamp `t_n = floor((1000 * n) / F)`
/// from the frame index `n`, avoiding cumulative rounding drift.
#[inline(always)]
pub const fn frame_timestamp_ms(frame_index_n: u64, fps: u64) -> u64 {
    if fps == 0 {
        return 0;
    }
    (1000 * frame_index_n) / fps
}

/// Checks whether a measured render duration `render_ns` (in nanoseconds) meets
/// the nominal frame budget `F * r <= 10^9` using exact integer arithmetic.
#[inline(always)]
pub const fn meets_frame_budget_ns(fps: u64, render_ns: u64) -> bool {
    (fps as u128) * (render_ns as u128) <= 1_000_000_000u128
}

#[derive(Debug, Clone, Serialize)]
pub struct FrameTimingReport {
    pub fps: u64,
    pub sample_render_ns: u64,
    pub meets_budget: bool,
    pub f_times_r_product: u128,
    pub budget_limit_product: u128,
    pub first_10_timestamps_ms: Vec<u64>,
    pub first_9_intervals_ms: Vec<u64>,
    pub timestamp_at_1_second_ms: u64,
    pub naive_rounded_1s_timestamp_ms: u64,
    pub cumulative_drift_eliminated_ms: i64,
}

pub fn evaluate_frame_timing(fps: u64, sample_render_ns: u64) -> FrameTimingReport {
    let f = fps.max(1);
    let timestamps: Vec<u64> = (0..10).map(|n| frame_timestamp_ms(n, f)).collect();
    let intervals: Vec<u64> = timestamps.windows(2).map(|w| w[1] - w[0]).collect();

    let exact_1s = frame_timestamp_ms(f, f);
    let naive_step = 1000 / f;
    let naive_1s = naive_step * f;

    FrameTimingReport {
        fps: f,
        sample_render_ns,
        meets_budget: meets_frame_budget_ns(f, sample_render_ns),
        f_times_r_product: (f as u128) * (sample_render_ns as u128),
        budget_limit_product: 1_000_000_000u128,
        first_10_timestamps_ms: timestamps,
        first_9_intervals_ms: intervals,
        timestamp_at_1_second_ms: exact_1s,
        naive_rounded_1s_timestamp_ms: naive_1s,
        cumulative_drift_eliminated_ms: (exact_1s as i64) - (naive_1s as i64),
    }
}

// ============================================================================
// Card 3: 64-Bit Ternary Quantizer Reconstruction Error & Optimal Thresholds
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct TernaryQuantizerEval {
    pub thresholds: (u8, u8),
    pub centroids: [u8; 3],
    pub reconstruction_error_sse: u64,
    pub mse: f64,
    pub level_counts: [u64; 3],
    pub level_occupancy_ratios: [f64; 3],
}

#[derive(Debug, Clone, Serialize)]
pub struct QuantizerQualityReport {
    pub num_pixels: u64,
    pub worst_case_error_bound_u64: u64,
    pub exceeds_u32_max: bool,
    pub u32_overflow_factor: f64,
    pub optimal_mse_quantizer: TernaryQuantizerEval,
    pub equal_occupancy_quantizer: TernaryQuantizerEval,
    pub sse_reduction_vs_equal_occupancy: u64,
}

/// Evaluates exact `u64` reconstruction error `E = \sum (x_i - c_{q(x_i)})^2` and finds
/// the global MSE-optimal ternary thresholds `(T_1, T_2)` and centroids `[c_0, c_1, c_2]`
/// in O(N + 256^2) time using a 256-bin histogram and prefix moments.
pub fn evaluate_ternary_quantizer(
    samples: &[u8],
    reference_pixel_count: usize,
) -> QuantizerQualityReport {
    let n_ref = reference_pixel_count.max(samples.len()).max(1) as u64;
    let worst_case_bound = 65_025_u64.saturating_mul(n_ref);
    let exceeds_u32 = worst_case_bound > (u32::MAX as u64);
    let overflow_factor = (worst_case_bound as f64) / (u32::MAX as f64);

    let mut hist = [0u64; 256];
    for &x in samples {
        hist[x as usize] += 1;
    }
    let total_samples = (samples.len() as u64).max(1);

    let mut pref_cnt = [0u64; 257];
    let mut pref_sum = [0u64; 257];
    let mut pref_sq = [0u64; 257];

    for val in 0..256 {
        let c = hist[val];
        let x = val as u64;
        pref_cnt[val + 1] = pref_cnt[val] + c;
        pref_sum[val + 1] = pref_sum[val] + c * x;
        pref_sq[val + 1] = pref_sq[val] + c * x * x;
    }

    let interval_stats = |lo: usize, hi: usize, fixed_centroid: Option<u8>| -> (u64, u8, u64) {
        if lo > hi {
            return (0, 0, 0);
        }
        let cnt = pref_cnt[hi + 1] - pref_cnt[lo];
        if cnt == 0 {
            let fallback_c = fixed_centroid.unwrap_or(((lo + hi) / 2) as u8);
            return (0, fallback_c, 0);
        }
        let sum_x = pref_sum[hi + 1] - pref_sum[lo];
        let sum_x2 = pref_sq[hi + 1] - pref_sq[lo];

        let c_u8 = fixed_centroid.unwrap_or_else(|| ((sum_x + (cnt / 2)) / cnt).min(255) as u8);
        let c = c_u8 as i128;

        let sse = (sum_x2 as i128) - 2 * c * (sum_x as i128) + (cnt as i128) * c * c;
        (cnt, c_u8, sse.max(0) as u64)
    };

    let mut best_eval: Option<TernaryQuantizerEval> = None;

    for t1 in 1..255usize {
        for t2 in (t1 + 1)..=255usize {
            let (cnt0, c0, sse0) = interval_stats(0, t1 - 1, None);
            let (cnt1, c1, sse1) = interval_stats(t1, t2 - 1, None);
            let (cnt2, c2, sse2) = interval_stats(t2, 255, None);

            let total_sse = sse0 + sse1 + sse2;
            let is_better = match &best_eval {
                None => true,
                Some(prev) => total_sse < prev.reconstruction_error_sse,
            };

            if is_better {
                best_eval = Some(TernaryQuantizerEval {
                    thresholds: (t1 as u8, t2 as u8),
                    centroids: [c0, c1, c2],
                    reconstruction_error_sse: total_sse,
                    mse: (total_sse as f64) / (total_samples as f64),
                    level_counts: [cnt0, cnt1, cnt2],
                    level_occupancy_ratios: [
                        (cnt0 as f64) / (total_samples as f64),
                        (cnt1 as f64) / (total_samples as f64),
                        (cnt2 as f64) / (total_samples as f64),
                    ],
                });
            }
        }
    }

    let target_third = total_samples / 3;
    let mut eq_t1 = 85usize;
    let mut eq_t2 = 170usize;
    for (v, &count) in pref_cnt.iter().enumerate().take(255).skip(1) {
        if count >= target_third {
            eq_t1 = v;
            break;
        }
    }
    for (v, &count) in pref_cnt.iter().enumerate().take(256).skip(eq_t1 + 1) {
        if count >= 2 * target_third {
            eq_t2 = v;
            break;
        }
    }

    let (ec0, _, esse0) = interval_stats(0, eq_t1 - 1, Some(0));
    let (ec1, _, esse1) = interval_stats(eq_t1, eq_t2 - 1, Some(128));
    let (ec2, _, esse2) = interval_stats(eq_t2, 255, Some(255));
    let eq_total_sse = esse0 + esse1 + esse2;

    let equal_occ_eval = TernaryQuantizerEval {
        thresholds: (eq_t1 as u8, eq_t2 as u8),
        centroids: [0, 128, 255],
        reconstruction_error_sse: eq_total_sse,
        mse: (eq_total_sse as f64) / (total_samples as f64),
        level_counts: [ec0, ec1, ec2],
        level_occupancy_ratios: [
            (ec0 as f64) / (total_samples as f64),
            (ec1 as f64) / (total_samples as f64),
            (ec2 as f64) / (total_samples as f64),
        ],
    };

    let optimal = best_eval.unwrap();
    let sse_saved = equal_occ_eval
        .reconstruction_error_sse
        .saturating_sub(optimal.reconstruction_error_sse);

    QuantizerQualityReport {
        num_pixels: n_ref,
        worst_case_error_bound_u64: worst_case_bound,
        exceeds_u32_max: exceeds_u32,
        u32_overflow_factor: overflow_factor,
        optimal_mse_quantizer: optimal,
        equal_occupancy_quantizer: equal_occ_eval,
        sse_reduction_vs_equal_occupancy: sse_saved,
    }
}

// ============================================================================
// Card 4: Prerequisite Deck Critical-Path C(v) vs. Sequential Duration \sum d(v)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StudyCardNode {
    pub id: usize,
    pub title: String,
    pub duration: u64,
    pub predecessors: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CardCompletionMetrics {
    pub id: usize,
    pub title: String,
    pub duration: u64,
    pub earliest_start: u64,
    pub earliest_completion_c: u64,
    pub latest_completion: u64,
    pub slack: u64,
    pub on_critical_path: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrerequisiteDeckReport {
    pub num_cards: usize,
    pub critical_path_duration: u64,
    pub sequential_total_duration: u64,
    pub max_parallelism_speedup: f64,
    pub critical_path_chain: Vec<usize>,
    pub card_schedule: Vec<CardCompletionMetrics>,
}

/// Computes `C(v) = d(v) + \max_{u \in pred(v)} C(u)`, overall critical-path duration
/// `\max_v C(v)`, and single-learner sequential duration `\sum_v d(v)`.
pub fn evaluate_prerequisite_deck(
    cards: &[StudyCardNode],
) -> Result<PrerequisiteDeckReport, String> {
    let n = cards.len();
    if n == 0 {
        return Ok(PrerequisiteDeckReport {
            num_cards: 0,
            critical_path_duration: 0,
            sequential_total_duration: 0,
            max_parallelism_speedup: 1.0,
            critical_path_chain: Vec::new(),
            card_schedule: Vec::new(),
        });
    }

    let mut in_degree = vec![0usize; n];
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut durations = vec![0u64; n];

    for card in cards {
        if card.id >= n {
            return Err(format!("Card ID {} out of bounds (0..{n})", card.id));
        }
        durations[card.id] = card.duration;
        for &u in &card.predecessors {
            if u >= n {
                return Err(format!("Predecessor ID {u} out of bounds (0..{n})"));
            }
            successors[u].push(card.id);
            in_degree[card.id] += 1;
        }
    }

    let mut queue: VecDeque<usize> = (0..n).filter(|&v| in_degree[v] == 0).collect();
    let mut topo_order = Vec::with_capacity(n);
    let mut max_pred_c = vec![0u64; n];
    let mut critical_pred: Vec<Option<usize>> = vec![None; n];
    let mut earliest_completion = vec![0u64; n];

    while let Some(u) = queue.pop_front() {
        topo_order.push(u);
        let c_u = durations[u] + max_pred_c[u];
        earliest_completion[u] = c_u;

        for &v in &successors[u] {
            if c_u > max_pred_c[v] || (c_u == max_pred_c[v] && critical_pred[v].is_none()) {
                max_pred_c[v] = c_u;
                critical_pred[v] = Some(u);
            }
            in_degree[v] -= 1;
            if in_degree[v] == 0 {
                queue.push_back(v);
            }
        }
    }

    if topo_order.len() != n {
        return Err("Prerequisite graph contains a dependency cycle".into());
    }

    let sequential_total: u64 = durations.iter().sum();
    let (crit_sink, &critical_path_duration) = earliest_completion
        .iter()
        .enumerate()
        .max_by_key(|&(_, &c)| c)
        .unwrap();

    let mut latest_completion = vec![critical_path_duration; n];
    for &u in topo_order.iter().rev() {
        if !successors[u].is_empty() {
            let min_succ_start = successors[u]
                .iter()
                .map(|&v| latest_completion[v].saturating_sub(durations[v]))
                .min()
                .unwrap_or(critical_path_duration);
            latest_completion[u] = min_succ_start;
        }
    }

    let mut chain = Vec::new();
    let mut cur = Some(crit_sink);
    while let Some(v) = cur {
        chain.push(v);
        cur = critical_pred[v];
    }
    chain.reverse();

    let mut schedule = Vec::with_capacity(n);
    for card in cards {
        let v = card.id;
        let c_v = earliest_completion[v];
        let l_v = latest_completion[v];
        let slack = l_v.saturating_sub(c_v);

        schedule.push(CardCompletionMetrics {
            id: v,
            title: card.title.clone(),
            duration: card.duration,
            earliest_start: max_pred_c[v],
            earliest_completion_c: c_v,
            latest_completion: l_v,
            slack,
            on_critical_path: slack == 0,
        });
    }

    let speedup = if critical_path_duration > 0 {
        (sequential_total as f64) / (critical_path_duration as f64)
    } else {
        1.0
    };

    Ok(PrerequisiteDeckReport {
        num_cards: n,
        critical_path_duration,
        sequential_total_duration: sequential_total,
        max_parallelism_speedup: speedup,
        critical_path_chain: chain,
        card_schedule: schedule,
    })
}

// ============================================================================
// Card 5: Discrete-Time Proportional Stability & Limit-Cycle Verifier
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct PidStabilityReport {
    pub plant_a: f64,
    pub plant_b: f64,
    pub gain_k: f64,
    pub k_min_exclusive: f64,
    pub k_max_exclusive: f64,
    pub deadbeat_gain_k: f64,
    pub closed_loop_eigenvalue: f64,
    pub is_linearly_stable: bool,
    pub ideal_equilibrium_y: f64,
    pub quantized_final_y: f64,
    pub quantized_residual_error: f64,
    pub persistent_limit_cycle_detected: bool,
    pub quantized_trajectory_tail: Vec<f64>,
}

/// Evaluates the exact linear stability range `(a - 1)/b < K < (a + 1)/b` for
/// `y_{k+1} = a * y_k + b * u_k` with `u_k = K * (r - y_k)`, and simulates the quantized
/// recurrence under integer control rounding, actuator saturation, and deadband.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_proportional_stability(
    a: f64,
    b: f64,
    k_gain: f64,
    setpoint_r: f64,
    y_init: f64,
    actuator_limit: f64,
    deadband: f64,
    integer_round_control: bool,
    steps: usize,
) -> Result<PidStabilityReport, String> {
    if b.abs() <= 1e-12 {
        return Err("Plant input gain b must be non-zero".into());
    }

    let bound1 = (a - 1.0) / b;
    let bound2 = (a + 1.0) / b;
    let k_min = bound1.min(bound2);
    let k_max = bound1.max(bound2);
    let deadbeat_k = a / b;

    let closed_loop_pole = a - b * k_gain;
    let is_stable = closed_loop_pole.abs() < 1.0;

    let denom = 1.0 - a + b * k_gain;
    let ideal_eq = if denom.abs() > 1e-12 {
        (b * k_gain * setpoint_r) / denom
    } else {
        setpoint_r
    };

    let sim_steps = steps.clamp(16, 512);
    let mut y_q = y_init;
    let mut trajectory = Vec::with_capacity(sim_steps);

    for _ in 0..sim_steps {
        let err = setpoint_r - y_q;
        let active_err = if err.abs() <= deadband { 0.0 } else { err };

        let mut u = k_gain * active_err;
        if integer_round_control {
            u = u.round();
        }
        u = u.clamp(-actuator_limit.abs(), actuator_limit.abs());

        y_q = a * y_q + b * u;
        trajectory.push(y_q);
    }

    let tail_len = 8.min(trajectory.len());
    let tail = trajectory[trajectory.len() - tail_len..].to_vec();
    let mut sign_flips = 0usize;
    let mut max_tail_delta = 0.0f64;

    for w in tail.windows(3) {
        let d1 = w[1] - w[0];
        let d2 = w[2] - w[1];
        max_tail_delta = max_tail_delta.max(d1.abs()).max(d2.abs());
        if d1 * d2 < -1e-9 {
            sign_flips += 1;
        }
    }

    let limit_cycle = sign_flips >= 3 && max_tail_delta > 1e-6;
    let final_y = *tail.last().unwrap_or(&y_q);

    Ok(PidStabilityReport {
        plant_a: a,
        plant_b: b,
        gain_k: k_gain,
        k_min_exclusive: k_min,
        k_max_exclusive: k_max,
        deadbeat_gain_k: deadbeat_k,
        closed_loop_eigenvalue: closed_loop_pole,
        is_linearly_stable: is_stable,
        ideal_equilibrium_y: ideal_eq,
        quantized_final_y: final_y,
        quantized_residual_error: (setpoint_r - final_y).abs(),
        persistent_limit_cycle_detected: limit_cycle,
        quantized_trajectory_tail: tail,
    })
}

// ============================================================================
// Unified MCP Tool Dispatcher Payload & Runner
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct TernaryDeckParams {
    #[serde(default = "default_width")]
    pub width: usize,
    #[serde(default = "default_height")]
    pub height: usize,
    #[serde(default = "default_fps")]
    pub fps: u64,
    #[serde(default = "default_render_ns")]
    pub sample_render_ns: u64,
    #[serde(default = "default_plant_a")]
    pub plant_a: f64,
    #[serde(default = "default_plant_b")]
    pub plant_b: f64,
    #[serde(default = "default_gain_k")]
    pub gain_k: f64,
}

fn default_width() -> usize {
    3840
}
fn default_height() -> usize {
    2160
}
fn default_fps() -> u64 {
    60
}
fn default_render_ns() -> u64 {
    16_500_000
}
fn default_plant_a() -> f64 {
    0.90
}
fn default_plant_b() -> f64 {
    0.25
}
fn default_gain_k() -> f64 {
    2.40
}

pub fn run_ternary_executable_deck(params: TernaryDeckParams) -> Result<Value, String> {
    let lane_card = evaluate_ternary_lane(params.width, params.height);
    let timing_card = evaluate_frame_timing(params.fps, params.sample_render_ns);

    let mut dark_scene = Vec::with_capacity(10_000);
    for i in 0..10_000usize {
        let pixel = if i < 8_000 {
            (i % 36) as u8
        } else if i < 9_500 {
            (90 + (i % 40)) as u8
        } else {
            (220 + (i % 36)) as u8
        };
        dark_scene.push(pixel);
    }
    let quantizer_card = evaluate_ternary_quantizer(&dark_scene, params.width * params.height);

    let study_cards = vec![
        StudyCardNode {
            id: 0,
            title: "Card 1: Ternary Lane Packing (4-Trit vs 5-Trit Base-3)".into(),
            duration: 15,
            predecessors: vec![],
        },
        StudyCardNode {
            id: 1,
            title: "Card 2: Drift-Free Integer Frame Timing (Fr <= 10^9)".into(),
            duration: 12,
            predecessors: vec![],
        },
        StudyCardNode {
            id: 2,
            title: "Card 3: 64-Bit Ternary Quantizer Reconstruction Error".into(),
            duration: 20,
            predecessors: vec![0],
        },
        StudyCardNode {
            id: 3,
            title: "Card 4: Prerequisite DAG Critical-Path vs Sequential Sum".into(),
            duration: 18,
            predecessors: vec![0, 1],
        },
        StudyCardNode {
            id: 4,
            title: "Card 5: Proportional Stability ((a-1)/b < K < (a+1)/b) & Limit Cycles".into(),
            duration: 25,
            predecessors: vec![2, 3],
        },
    ];
    let schedule_card = evaluate_prerequisite_deck(&study_cards)?;

    let pid_card = evaluate_proportional_stability(
        params.plant_a,
        params.plant_b,
        params.gain_k,
        100.0,
        0.0,
        500.0,
        0.5,
        true,
        64,
    )?;

    Ok(json!({
        "status": "verified",
        "card_1_ternary_lane": lane_card,
        "card_2_frame_timing": timing_card,
        "card_3_quantizer_quality": quantizer_card,
        "card_4_prerequisite_schedule": schedule_card,
        "card_5_pid_stability": pid_card
    }))
}

// ============================================================================
// Card 5 (Extended): Discrete-Time Z-Domain PID Jury Stability
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacteristicOrder {
    /// K_i == 0 and K_d == 0: P(z) = z - (a - b*K_p)
    FirstOrderP,
    /// K_i == 0 and K_d != 0: P(z) = z^2 + (b*(K_p + K_d) - a)*z + b*K_d
    SecondOrderPd,
    /// K_i != 0: P(z) = z^3 + a_2*z^2 + a_1*z + a_0
    ThirdOrderPid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ComplexRoot {
    pub re: f64,
    pub im: f64,
    pub magnitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuryStabilityCertificate {
    pub effective_order: CharacteristicOrder,
    /// Monic cubic coefficients [a_3 = 1.0, a_2, a_1, a_0] of P(z) = z^3 + a_2*z^2 + a_1*z + a_0.
    pub cubic_coefficients: [f64; 4],
    /// Condition 1: P(1) > 0 (or reduced order equivalent).
    pub cond1_p_at_one_val: f64,
    pub cond1_p_at_one_holds: bool,
    /// Condition 2: (-1)^n * P(-1) > 0.
    pub cond2_alternating_minus_one_val: f64,
    pub cond2_alternating_minus_one_holds: bool,
    /// Condition 3: |a_0| < a_n (1.0).
    pub cond3_abs_a0_val: f64,
    pub cond3_abs_a0_lt_one_holds: bool,
    /// Condition 4 (Cubic Jury Row 3): |b_0| > |b_2| where b_0 = a_0^2 - 1, b_2 = a_0*a_2 - a_1.
    pub jury_b0: f64,
    pub jury_b2: f64,
    pub cond4_jury_det_margin: f64,
    pub cond4_jury_det_holds: bool,
    pub is_jury_stable: bool,
    /// Exact closed-loop poles z_i and spectral radius max_i |z_i|.
    pub roots: Vec<ComplexRoot>,
    pub spectral_radius: f64,
    /// Proportional-only reference bounds (a - 1)/b < K_p < (a + 1)/b when K_i = K_d = 0.
    pub proportional_only_k_bounds: (f64, f64),
    pub proportional_deadbeat_k: f64,
}

/// Evaluates the Jury stability certificate and exact complex roots for the discrete-time
/// PID closed-loop system `P(z) = z(z - 1)(z - a) + b * [(K_p + K_i + K_d)z^2 - (K_p + 2K_d)z + K_d]`.
pub fn evaluate_jury_pid_certificate(
    a: f64,
    b: f64,
    kp: f64,
    ki: f64,
    kd: f64,
) -> Result<JuryStabilityCertificate, String> {
    if b.abs() <= 1e-12 {
        return Err("Plant input gain b must be non-zero".into());
    }

    let bound1 = (a - 1.0) / b;
    let bound2 = (a + 1.0) / b;
    let prop_bounds = (bound1.min(bound2), bound1.max(bound2));
    let deadbeat_k = a / b;

    // Monic cubic coefficients: z^3 + a_2*z^2 + a_1*z + a_0 = 0
    let a3 = 1.0;
    let a2 = b * (kp + ki + kd) - (1.0 + a);
    let a1 = a - b * (kp + 2.0 * kd);
    let a0 = b * kd;

    let eps = 1e-12;

    if ki.abs() <= eps && kd.abs() <= eps {
        // 1st-Order Proportional Reduction: z - (a - b*K_p) = 0
        let pole = a - b * kp;
        let q_at_1 = 1.0 - pole;
        let neg_q_at_m1 = 1.0 + pole;
        let abs_pole = pole.abs();
        let stable = abs_pole < 1.0;

        return Ok(JuryStabilityCertificate {
            effective_order: CharacteristicOrder::FirstOrderP,
            cubic_coefficients: [a3, a2, a1, a0],
            cond1_p_at_one_val: q_at_1,
            cond1_p_at_one_holds: q_at_1 > 0.0,
            cond2_alternating_minus_one_val: neg_q_at_m1,
            cond2_alternating_minus_one_holds: neg_q_at_m1 > 0.0,
            cond3_abs_a0_val: abs_pole,
            cond3_abs_a0_lt_one_holds: stable,
            jury_b0: 1.0 - pole * pole,
            jury_b2: 0.0,
            cond4_jury_det_margin: 1.0 - pole * pole,
            cond4_jury_det_holds: stable,
            is_jury_stable: stable,
            roots: vec![ComplexRoot {
                re: pole,
                im: 0.0,
                magnitude: abs_pole,
            }],
            spectral_radius: abs_pole,
            proportional_only_k_bounds: prop_bounds,
            proportional_deadbeat_k: deadbeat_k,
        });
    }

    if ki.abs() <= eps {
        // 2nd-Order PD Reduction (cancels z = 1 factor): Q(z) = z^2 + q_1*z + q_0 = 0
        let q1 = b * (kp + kd) - a;
        let q0 = b * kd;

        let cond1_val = 1.0 + q1 + q0; // Q(1) = 1 - a + b*K_p > 0
        let cond2_val = 1.0 - q1 + q0; // Q(-1) = 1 + a - b*(K_p + 2*K_d) > 0
        let cond3_val = q0.abs(); // |q_0| < 1

        let c1 = cond1_val > 0.0;
        let c2 = cond2_val > 0.0;
        let c3 = cond3_val < 1.0;
        let stable = c1 && c2 && c3;

        let roots = solve_quadratic_roots(q1, q0);
        let rho = roots.iter().map(|r| r.magnitude).fold(0.0, f64::max);

        return Ok(JuryStabilityCertificate {
            effective_order: CharacteristicOrder::SecondOrderPd,
            cubic_coefficients: [a3, a2, a1, a0],
            cond1_p_at_one_val: cond1_val,
            cond1_p_at_one_holds: c1,
            cond2_alternating_minus_one_val: cond2_val,
            cond2_alternating_minus_one_holds: c2,
            cond3_abs_a0_val: cond3_val,
            cond3_abs_a0_lt_one_holds: c3,
            jury_b0: q0 * q0 - 1.0,
            jury_b2: 0.0,
            cond4_jury_det_margin: 1.0 - cond3_val,
            cond4_jury_det_holds: c3,
            is_jury_stable: stable,
            roots,
            spectral_radius: rho,
            proportional_only_k_bounds: prop_bounds,
            proportional_deadbeat_k: deadbeat_k,
        });
    }

    // Full 3rd-Order PID Jury Test on P(z) = z^3 + a_2*z^2 + a_1*z + a_0
    let p_at_1 = 1.0 + a2 + a1 + a0; // = b * K_i
    let neg_p_at_m1 = 1.0 - a2 + a1 - a0; // = 2*(1 + a) - b*(2*K_p + K_i + 4*K_d)
    let abs_a0 = a0.abs();

    // Jury Row 3 determinants: b_0 = a_0^2 - a_3^2, b_2 = a_0*a_2 - a_3*a_1
    let jury_b0 = a0 * a0 - 1.0;
    let jury_b2 = a0 * a2 - a1;
    let det_margin = jury_b0.abs() - jury_b2.abs();

    let c1 = p_at_1 > 0.0;
    let c2 = neg_p_at_m1 > 0.0;
    let c3 = abs_a0 < 1.0;
    let c4 = jury_b0.abs() > jury_b2.abs();
    let is_jury_stable = c1 && c2 && c3 && c4;

    let roots = solve_monic_cubic_roots(a2, a1, a0);
    let rho = roots.iter().map(|r| r.magnitude).fold(0.0, f64::max);

    Ok(JuryStabilityCertificate {
        effective_order: CharacteristicOrder::ThirdOrderPid,
        cubic_coefficients: [a3, a2, a1, a0],
        cond1_p_at_one_val: p_at_1,
        cond1_p_at_one_holds: c1,
        cond2_alternating_minus_one_val: neg_p_at_m1,
        cond2_alternating_minus_one_holds: c2,
        cond3_abs_a0_val: abs_a0,
        cond3_abs_a0_lt_one_holds: c3,
        jury_b0,
        jury_b2,
        cond4_jury_det_margin: det_margin,
        cond4_jury_det_holds: c4,
        is_jury_stable,
        roots,
        spectral_radius: rho,
        proportional_only_k_bounds: prop_bounds,
        proportional_deadbeat_k: deadbeat_k,
    })
}

fn solve_quadratic_roots(q1: f64, q0: f64) -> Vec<ComplexRoot> {
    let disc = q1 * q1 - 4.0 * q0;
    if disc >= 0.0 {
        let s = disc.sqrt();
        let r1 = (-q1 + s) * 0.5;
        let r2 = (-q1 - s) * 0.5;
        vec![
            ComplexRoot {
                re: r1,
                im: 0.0,
                magnitude: r1.abs(),
            },
            ComplexRoot {
                re: r2,
                im: 0.0,
                magnitude: r2.abs(),
            },
        ]
    } else {
        let re = -0.5 * q1;
        let im = 0.5 * (-disc).sqrt();
        let mag = (re * re + im * im).sqrt();
        vec![
            ComplexRoot {
                re,
                im,
                magnitude: mag,
            },
            ComplexRoot {
                re,
                im: -im,
                magnitude: mag,
            },
        ]
    }
}

/// Computes the 3 complex roots of `z^3 + a2*z^2 + a1*z + a0 = 0` via depressed Cardano formulas.
fn solve_monic_cubic_roots(a2: f64, a1: f64, a0: f64) -> Vec<ComplexRoot> {
    let shift = a2 / 3.0;
    let p = a1 - (a2 * a2) / 3.0;
    let q = (2.0 * a2 * a2 * a2) / 27.0 - (a2 * a1) / 3.0 + a0;

    let half_q = 0.5 * q;
    let third_p = p / 3.0;
    let disc = half_q * half_q + third_p * third_p * third_p;

    let mut roots = Vec::with_capacity(3);

    if disc > 1e-14 {
        // 1 real root and 1 complex conjugate pair
        let sqrt_disc = disc.sqrt();
        let u = (-half_q + sqrt_disc).cbrt();
        let v = (-half_q - sqrt_disc).cbrt();

        let z1_re = (u + v) - shift;
        let z2_re = -0.5 * (u + v) - shift;
        let z2_im = 0.5 * 3.0_f64.sqrt() * (u - v);

        roots.push(ComplexRoot {
            re: z1_re,
            im: 0.0,
            magnitude: z1_re.abs(),
        });
        let mag2 = (z2_re * z2_re + z2_im * z2_im).sqrt();
        roots.push(ComplexRoot {
            re: z2_re,
            im: z2_im,
            magnitude: mag2,
        });
        roots.push(ComplexRoot {
            re: z2_re,
            im: -z2_im,
            magnitude: mag2,
        });
    } else if disc.abs() <= 1e-14 {
        // All real roots, at least two equal
        let u = (-half_q).cbrt();
        let z1 = 2.0 * u - shift;
        let z2 = -u - shift;
        roots.push(ComplexRoot {
            re: z1,
            im: 0.0,
            magnitude: z1.abs(),
        });
        roots.push(ComplexRoot {
            re: z2,
            im: 0.0,
            magnitude: z2.abs(),
        });
        roots.push(ComplexRoot {
            re: z2,
            im: 0.0,
            magnitude: z2.abs(),
        });
    } else {
        // Casus irreducibilis: 3 distinct real roots via trigonometric Viète formula
        let r_mag = (-third_p).max(0.0).sqrt();
        let cos_arg = if r_mag > 1e-15 {
            (-half_q / (r_mag * r_mag * r_mag)).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let phi = cos_arg.acos();

        for k in 0..3 {
            let angle = (phi - (k as f64) * 2.0 * std::f64::consts::PI) / 3.0;
            let zk = 2.0 * r_mag * angle.cos() - shift;
            roots.push(ComplexRoot {
                re: zk,
                im: 0.0,
                magnitude: zk.abs(),
            });
        }
    }

    roots.sort_by(|a, b| {
        b.magnitude
            .partial_cmp(&a.magnitude)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    roots
}
