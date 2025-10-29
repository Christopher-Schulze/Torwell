use once_cell::sync::Lazy;
use std::error::Error;
use std::fmt;

#[cfg(target_arch = "aarch64")]
mod neon;
mod scalar;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

/// Result type used by SIMD helpers.
pub type SimdResult<T> = Result<T, SimdError>;

/// Errors returned by SIMD helpers when buffer shapes do not match expectations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimdError {
    /// Pixel buffer length is not divisible by 4 (RGBA layout expected).
    PixelLengthNotMultipleOf4 { len: usize },
    /// Luma mask length does not match the number of pixels in the source buffer.
    MaskLengthMismatch { pixels: usize, mask: usize },
}

impl fmt::Display for SimdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimdError::PixelLengthNotMultipleOf4 { len } => {
                write!(f, "pixel buffer length {len} is not a multiple of 4")
            }
            SimdError::MaskLengthMismatch { pixels, mask } => {
                write!(f, "mask length {mask} does not match pixel count {pixels}",)
            }
        }
    }
}

impl Error for SimdError {}

/// SIMD backend detected at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Avx2,
    Avx,
    Neon,
    Scalar,
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Backend::Avx2 => write!(f, "avx2"),
            Backend::Avx => write!(f, "avx"),
            Backend::Neon => write!(f, "neon"),
            Backend::Scalar => write!(f, "scalar"),
        }
    }
}

static BACKEND: Lazy<Backend> = Lazy::new(detect_backend);

fn detect_backend() -> Backend {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            return Backend::Avx2;
        }
        if std::arch::is_x86_feature_detected!("avx") {
            return Backend::Avx;
        }
        return Backend::Scalar;
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return Backend::Neon;
        }
        return Backend::Scalar;
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64",)))]
    {
        Backend::Scalar
    }
}

/// Return the backend detected for SIMD workloads.
#[inline]
pub fn backend() -> Backend {
    *BACKEND
}

/// Multiply RGBA channels by the supplied gain factors in-place.
#[inline]
pub fn apply_gain_inplace(pixels: &mut [f32], gain: [f32; 4]) -> SimdResult<()> {
    scalar::ensure_pixel_chunks(pixels)?;
    match backend() {
        Backend::Avx2 => {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                // SAFETY: feature detection in `backend` guarantees that AVX2 is available.
                x86::apply_gain_inplace_avx2(pixels, gain)
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                unreachable!("avx2 backend selected on non-x86 target")
            }
        }
        Backend::Avx => {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                // SAFETY: feature detection in `backend` guarantees that AVX is available.
                x86::apply_gain_inplace_avx(pixels, gain)
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                unreachable!("avx backend selected on non-x86 target")
            }
        }
        Backend::Neon => {
            #[cfg(target_arch = "aarch64")]
            unsafe {
                // SAFETY: feature detection ensures NEON support on the running CPU.
                neon::apply_gain_inplace_neon(pixels, gain)
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                unreachable!("neon backend selected on non-aarch64 target")
            }
        }
        Backend::Scalar => scalar::apply_gain_inplace(pixels, gain),
    }
}

/// Add the supplied bias to RGBA channels in-place.
#[inline]
pub fn apply_bias_inplace(pixels: &mut [f32], bias: [f32; 4]) -> SimdResult<()> {
    scalar::ensure_pixel_chunks(pixels)?;
    match backend() {
        Backend::Avx2 => {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                x86::apply_bias_inplace_avx2(pixels, bias)
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                unreachable!("avx2 backend selected on non-x86 target")
            }
        }
        Backend::Avx => {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                x86::apply_bias_inplace_avx(pixels, bias)
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                unreachable!("avx backend selected on non-x86 target")
            }
        }
        Backend::Neon => {
            #[cfg(target_arch = "aarch64")]
            unsafe {
                neon::apply_bias_inplace_neon(pixels, bias)
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                unreachable!("neon backend selected on non-aarch64 target")
            }
        }
        Backend::Scalar => scalar::apply_bias_inplace(pixels, bias),
    }
}

/// Compute luminance (Rec. 709) for each RGBA pixel into `mask`.
#[inline]
pub fn luma_into(pixels: &[f32], mask: &mut [f32]) -> SimdResult<()> {
    scalar::ensure_pixel_chunks(pixels)?;
    match backend() {
        Backend::Avx2 => {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                x86::luma_into_avx2(pixels, mask)
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                unreachable!("avx2 backend selected on non-x86 target")
            }
        }
        Backend::Avx => {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            unsafe {
                x86::luma_into_avx(pixels, mask)
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                unreachable!("avx backend selected on non-x86 target")
            }
        }
        Backend::Neon => {
            #[cfg(target_arch = "aarch64")]
            unsafe {
                neon::luma_into_neon(pixels, mask)
            }
            #[cfg(not(target_arch = "aarch64"))]
            {
                unreachable!("neon backend selected on non-aarch64 target")
            }
        }
        Backend::Scalar => scalar::luma_into(pixels, mask),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const GAIN_RANGE: std::ops::Range<f32> = -0.5..2.5;
    const PIXEL_RANGE: std::ops::Range<f32> = -4.0..4.0;

    prop_compose! {
        fn pixel_data()(len in 1usize..32)
            (values in prop::collection::vec(PIXEL_RANGE, len * 4)) -> (usize, Vec<f32>) {
            (len, values)
        }
    }

    fn approx_eq(lhs: &[f32], rhs: &[f32]) -> bool {
        lhs.iter()
            .zip(rhs.iter())
            .all(|(l, r)| (l - r).abs() <= 1e-5 * l.abs().max(r.abs()).max(1.0))
    }

    proptest! {
        #[test]
        fn gain_matches_scalar((_, mut pixels) in pixel_data(), gain in prop::array::uniform4(GAIN_RANGE)) {
            let mut expected = pixels.clone();
            scalar::apply_gain_inplace(&mut expected, gain).unwrap();
            apply_gain_inplace(&mut pixels, gain).unwrap();
            prop_assert!(approx_eq(&expected, &pixels));
        }
    }

    proptest! {
        #[test]
        fn bias_matches_scalar((_, mut pixels) in pixel_data(), bias in prop::array::uniform4(GAIN_RANGE)) {
            let mut expected = pixels.clone();
            scalar::apply_bias_inplace(&mut expected, bias).unwrap();
            apply_bias_inplace(&mut pixels, bias).unwrap();
            prop_assert!(approx_eq(&expected, &pixels));
        }
    }

    proptest! {
        #[test]
        fn luma_matches_scalar((len, pixels) in pixel_data()) {
            let mut expected = vec![0.0; len];
            let mut actual = vec![0.0; len];
            scalar::luma_into(&pixels, &mut expected).unwrap();
            luma_into(&pixels, &mut actual).unwrap();
            prop_assert!(approx_eq(&expected, &actual));
        }
    }

    #[test]
    fn rejects_invalid_lengths() {
        let mut pixels = vec![0.0f32; 5];
        let err = apply_gain_inplace(&mut pixels, [1.0, 1.0, 1.0, 1.0]).unwrap_err();
        assert!(matches!(err, SimdError::PixelLengthNotMultipleOf4 { .. }));
    }

    #[test]
    fn rejects_mask_mismatch() {
        let pixels = vec![0.0f32; 8];
        let mut mask = vec![0.0f32; 1];
        let err = luma_into(&pixels, &mut mask).unwrap_err();
        assert!(matches!(err, SimdError::MaskLengthMismatch { .. }));
    }

    #[test]
    fn snapshot_pipeline_roundtrip() {
        let mut pixels = vec![
            0.05, 0.12, 0.87, 1.0, 0.98, 0.45, 0.22, 1.0, 0.32, 0.76, 0.19, 0.5, 0.14, 0.24, 0.62,
            0.8,
        ];
        apply_gain_inplace(&mut pixels, [1.4, 0.9, 0.75, 1.0]).unwrap();
        apply_bias_inplace(&mut pixels, [-0.02, 0.01, 0.02, 0.0]).unwrap();
        let mut luma = vec![0.0; pixels.len() / 4];
        luma_into(&pixels, &mut luma).unwrap();
        insta::assert_debug_snapshot!("simd_pipeline_luma", (&pixels, &luma, backend()));
    }
}
