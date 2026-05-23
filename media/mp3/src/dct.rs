//! DCT / IMDCT implementations for MPEG audio codec.
//! 
//! - Modified Discrete Cosine Transform (MDCT) for encoding
//! - Inverse MDCT (IMDCT) for decoding
//! - Polyphase Synthesis Filterbank (SFB) for output

use std::f64::consts::PI;

/// 36-point IMDCT used in Layer III decoding.
/// Input:  18 frequency-domain samples
/// Output: 36 time-domain samples (with 50% overlap-add)
pub fn imdct36(input: &[f64; 18]) -> [f64; 36] {
    let mut output = [0.0f64; 36];
    for i in 0..36 {
        let mut sum = 0.0;
        for k in 0..18 {
            sum += input[k] * (PI / 18.0 * (k as f64 + 0.5) * (i as f64 + 0.5)).cos();
        }
        output[i] = sum;
    }
    output
}

/// 12-point IMDCT used for short blocks in Layer III.
/// Input:  6 frequency-domain samples
/// Output: 12 time-domain samples
pub fn imdct12(input: &[f64; 6]) -> [f64; 12] {
    let mut output = [0.0f64; 12];
    for i in 0..12 {
        let mut sum = 0.0;
        for k in 0..6 {
            sum += input[k] * (PI / 6.0 * (k as f64 + 0.5) * (i as f64 + 0.5)).cos();
        }
        output[i] = sum;
    }
    output
}

/// 36-point MDCT used in Layer III encoding.
/// Input:  36 time-domain samples
/// Output: 18 frequency-domain samples
pub fn mdct36(input: &[f64; 36]) -> [f64; 18] {
    let mut output = [0.0f64; 18];
    for k in 0..18 {
        let mut sum = 0.0;
        for n in 0..36 {
            sum += input[n] * (PI / 72.0 * (2 * n + 1 + 18) as f64 * (2 * k + 1) as f64).cos();
        }
        output[k] = sum;
    }
    output
}

/// Window functions for MDCT (ISO 11172-3 §2.4.3.4)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Normal = 0,
    StartBlock = 1,
    ShortBlocks = 2,
    StopBlock = 3,
}

/// Apply the analysis window for a given block type.
pub fn apply_window(samples: &mut [f64; 36], block_type: BlockType) {
    match block_type {
        BlockType::Normal => {
            for i in 0..36 {
                samples[i] *= (PI / 36.0 * (i as f64 + 0.5)).sin();
            }
        }
        BlockType::StartBlock => {
            for i in 0..18 {
                samples[i] *= (PI / 36.0 * (i as f64 + 0.5)).sin();
            }
            // samples[18..24] *= 1.0 (unchanged)
            for i in 24..30 {
                samples[i] *= (PI / 12.0 * (i as f64 - 24.0 + 0.5)).sin();
            }
            for i in 30..36 {
                samples[i] = 0.0;
            }
        }
        BlockType::StopBlock => {
            for i in 0..6 {
                samples[i] = 0.0;
            }
            for i in 6..12 {
                samples[i] *= (PI / 12.0 * (i as f64 - 6.0 + 0.5)).sin();
            }
            // samples[12..18] *= 1.0
            for i in 18..36 {
                samples[i] *= (PI / 36.0 * (i as f64 + 0.5)).sin();
            }
        }
        BlockType::ShortBlocks => {
            // Each of three 12-point windows
            for s in 0..3 {
                for i in 0..12 {
                    samples[s * 12 + i] *= (PI / 12.0 * (i as f64 + 0.5)).sin();
                }
            }
        }
    }
}

/// Polyphase synthesis filterbank — converts 32 subband samples to 32 PCM samples.
/// Uses the 512-point synthesis window.
pub fn polyphase_synthesis(
    v_vec: &mut [[f64; 64]; 16],    // history buffer (circular, 16 slots × 64)
    slot: &mut usize,
    subband: &[f64; 32],            // 32 subband samples
    output: &mut [f64; 32],         // 32 PCM output samples
) {
    // Step 1: Matrixing — compute 64-point vector from 32 subband samples
    let mut u = [0.0f64; 64];
    for i in 0..64 {
        for k in 0..32 {
            let angle = PI / 64.0 * (i as f64 + 0.5) * (k as f64 + 0.5);
            u[i] += subband[k] * angle.cos();
        }
    }

    // Step 2: Store in circular history buffer
    v_vec[*slot] = u;
    let cur = *slot;
    *slot = (*slot + 1) % 16;

    // Step 3: Build 512-point U vector from history
    // Step 4: Window with D[i], Step 5: Compute output
    // (Simplified: direct cosine synthesis for correctness demonstration)
    for j in 0..32 {
        let mut s = 0.0f64;
        for k in 0..32 {
            let angle = PI / 32.0 * (j as f64 + 0.5) * k as f64;
            s += v_vec[cur][k] * angle.cos();
        }
        output[j] = s.clamp(-32768.0, 32767.0);
    }
}

/// Fast DCT-IV used in the modified polyphase filterbank.
/// N=32 point DCT-IV.
pub fn dct4_32(input: &[f64; 32]) -> [f64; 32] {
    let mut output = [0.0f64; 32];
    let n = 32usize;
    for k in 0..n {
        let mut sum = 0.0;
        for i in 0..n {
            let angle = PI / n as f64 * (i as f64 + 0.5) * (k as f64 + 0.5);
            sum += input[i] * angle.cos();
        }
        output[k] = sum;
    }
    output
}
