#![allow(dead_code)]

use super::{scalar, SimdResult};
use core::arch::aarch64::*;

#[target_feature(enable = "neon")]
pub(super) unsafe fn apply_gain_inplace_neon(pixels: &mut [f32], gain: [f32; 4]) -> SimdResult<()> {
    let gain_vec = vld1q_f32(gain.as_ptr());
    for chunk in pixels.chunks_exact_mut(4) {
        let ptr = chunk.as_mut_ptr();
        let values = vld1q_f32(ptr);
        let result = vmulq_f32(values, gain_vec);
        vst1q_f32(ptr, result);
    }
    Ok(())
}

#[target_feature(enable = "neon")]
pub(super) unsafe fn apply_bias_inplace_neon(pixels: &mut [f32], bias: [f32; 4]) -> SimdResult<()> {
    let bias_vec = vld1q_f32(bias.as_ptr());
    for chunk in pixels.chunks_exact_mut(4) {
        let ptr = chunk.as_mut_ptr();
        let values = vld1q_f32(ptr);
        let result = vaddq_f32(values, bias_vec);
        vst1q_f32(ptr, result);
    }
    Ok(())
}

#[target_feature(enable = "neon")]
pub(super) unsafe fn luma_into_neon(pixels: &[f32], mask: &mut [f32]) -> SimdResult<()> {
    let pixel_count = scalar::ensure_pixel_chunks(pixels)?;
    if mask.len() != pixel_count {
        return Err(super::SimdError::MaskLengthMismatch {
            pixels: pixel_count,
            mask: mask.len(),
        });
    }
    let weights = vld1q_f32([0.2126_f32, 0.7152_f32, 0.0722_f32, 0.0_f32].as_ptr());
    for (chunk, out) in pixels.chunks_exact(4).zip(mask.iter_mut()) {
        let values = vld1q_f32(chunk.as_ptr());
        let mul = vmulq_f32(values, weights);
        let sum = vaddvq_f32(mul);
        *out = sum;
    }
    Ok(())
}
