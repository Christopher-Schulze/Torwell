use super::{SimdError, SimdResult};

#[inline]
pub(crate) fn apply_gain_inplace(pixels: &mut [f32], gain: [f32; 4]) -> SimdResult<()> {
    ensure_pixel_chunks(pixels)?;
    for chunk in pixels.chunks_exact_mut(4) {
        chunk[0] *= gain[0];
        chunk[1] *= gain[1];
        chunk[2] *= gain[2];
        chunk[3] *= gain[3];
    }
    Ok(())
}

#[inline]
pub(crate) fn apply_bias_inplace(pixels: &mut [f32], bias: [f32; 4]) -> SimdResult<()> {
    ensure_pixel_chunks(pixels)?;
    for chunk in pixels.chunks_exact_mut(4) {
        chunk[0] += bias[0];
        chunk[1] += bias[1];
        chunk[2] += bias[2];
        chunk[3] += bias[3];
    }
    Ok(())
}

#[inline]
pub(crate) fn luma_into(pixels: &[f32], mask: &mut [f32]) -> SimdResult<()> {
    let pixel_count = ensure_pixel_chunks(pixels)?;
    if mask.len() != pixel_count {
        return Err(SimdError::MaskLengthMismatch {
            pixels: pixel_count,
            mask: mask.len(),
        });
    }

    for (chunk, out) in pixels.chunks_exact(4).zip(mask.iter_mut()) {
        let luma = chunk[0] * 0.2126 + chunk[1] * 0.7152 + chunk[2] * 0.0722;
        *out = luma;
    }
    Ok(())
}

#[inline]
pub(crate) fn ensure_pixel_chunks<T>(buffer: &[T]) -> SimdResult<usize> {
    if buffer.len() % 4 != 0 {
        return Err(SimdError::PixelLengthNotMultipleOf4 { len: buffer.len() });
    }
    Ok(buffer.len() / 4)
}
