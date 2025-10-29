#![allow(dead_code)]

use super::{scalar, SimdResult};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[target_feature(enable = "avx", enable = "sse4.1")]
pub(super) unsafe fn apply_gain_inplace_avx(pixels: &mut [f32], gain: [f32; 4]) -> SimdResult<()> {
    let gain_vec = _mm256_set_ps(
        gain[3], gain[2], gain[1], gain[0], gain[3], gain[2], gain[1], gain[0],
    );
    let mut chunks = pixels.chunks_exact_mut(8);
    for chunk in &mut chunks {
        let ptr = chunk.as_mut_ptr();
        // SAFETY: `chunk` is aligned to at least 4 bytes and contains 8 f32 values.
        let values = _mm256_loadu_ps(ptr);
        let result = _mm256_mul_ps(values, gain_vec);
        // SAFETY: Same pointer that was loaded, still valid and mutable.
        _mm256_storeu_ps(ptr, result);
    }
    scalar::apply_gain_inplace(chunks.into_remainder(), gain)
}

#[target_feature(enable = "avx2", enable = "sse4.1")]
pub(super) unsafe fn apply_gain_inplace_avx2(pixels: &mut [f32], gain: [f32; 4]) -> SimdResult<()> {
    apply_gain_inplace_avx(pixels, gain)
}

#[target_feature(enable = "avx", enable = "sse4.1")]
pub(super) unsafe fn apply_bias_inplace_avx(pixels: &mut [f32], bias: [f32; 4]) -> SimdResult<()> {
    let bias_vec = _mm256_set_ps(
        bias[3], bias[2], bias[1], bias[0], bias[3], bias[2], bias[1], bias[0],
    );
    let mut chunks = pixels.chunks_exact_mut(8);
    for chunk in &mut chunks {
        let ptr = chunk.as_mut_ptr();
        let values = _mm256_loadu_ps(ptr);
        let result = _mm256_add_ps(values, bias_vec);
        _mm256_storeu_ps(ptr, result);
    }
    scalar::apply_bias_inplace(chunks.into_remainder(), bias)
}

#[target_feature(enable = "avx2", enable = "sse4.1")]
pub(super) unsafe fn apply_bias_inplace_avx2(pixels: &mut [f32], bias: [f32; 4]) -> SimdResult<()> {
    apply_bias_inplace_avx(pixels, bias)
}

#[target_feature(enable = "avx", enable = "sse4.1")]
pub(super) unsafe fn luma_into_avx(pixels: &[f32], mask: &mut [f32]) -> SimdResult<()> {
    let weights = _mm_set_ps(0.0, 0.0722, 0.7152, 0.2126);
    let mut pixel_chunks = pixels.chunks_exact(8);
    let mut mask_chunks = mask.chunks_exact_mut(2);

    for (pixel_chunk, mask_chunk) in pixel_chunks.by_ref().zip(mask_chunks.by_ref()) {
        let ptr = pixel_chunk.as_ptr();
        let values = _mm256_loadu_ps(ptr);
        let low = _mm256_castps256_ps128(values);
        let high = _mm256_extractf128_ps(values, 1);
        let l_low = _mm_dp_ps(low, weights, 0b0111_0001);
        let l_high = _mm_dp_ps(high, weights, 0b0111_0001);
        mask_chunk[0] = _mm_cvtss_f32(l_low);
        mask_chunk[1] = _mm_cvtss_f32(l_high);
    }

    let remainder_pixels = pixel_chunks.remainder();
    let remainder_mask = mask_chunks.into_remainder();
    if !remainder_pixels.is_empty() {
        debug_assert_eq!(remainder_pixels.len(), 4);
        debug_assert_eq!(remainder_mask.len(), 1);
        scalar::luma_into(remainder_pixels, remainder_mask)
    } else {
        Ok(())
    }
}

#[target_feature(enable = "avx2", enable = "sse4.1")]
pub(super) unsafe fn luma_into_avx2(pixels: &[f32], mask: &mut [f32]) -> SimdResult<()> {
    luma_into_avx(pixels, mask)
}
