use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::SeedableRng;
use rand::{rngs::StdRng, Rng};
type SimdResult<T> = torwell84::lib::simd::SimdResult<T>;

fn prepare_pixels(len: usize) -> Vec<f32> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..len).map(|_| rng.gen_range(0.0_f32..=1.0)).collect()
}

fn bench_gain(c: &mut Criterion) {
    let backend = torwell84::lib::simd::backend();
    let mut group = c.benchmark_group(format!("simd_gain/{backend}"));
    group.bench_function("apply_gain_4k", |b| {
        b.iter_batched(
            || prepare_pixels(4096),
            |mut pixels| -> SimdResult<()> {
                torwell84::lib::simd::apply_gain_inplace(&mut pixels, [1.15, 0.9, 0.8, 1.0])
            },
            BatchSize::SmallInput,
        )
        .unwrap();
    });
    group.finish();
}

fn bench_bias(c: &mut Criterion) {
    let backend = torwell84::lib::simd::backend();
    let mut group = c.benchmark_group(format!("simd_bias/{backend}"));
    group.bench_function("apply_bias_4k", |b| {
        b.iter_batched(
            || prepare_pixels(4096),
            |mut pixels| -> SimdResult<()> {
                torwell84::lib::simd::apply_bias_inplace(&mut pixels, [-0.02, 0.04, -0.01, 0.0])
            },
            BatchSize::SmallInput,
        )
        .unwrap();
    });
    group.finish();
}

fn bench_luma(c: &mut Criterion) {
    let backend = torwell84::lib::simd::backend();
    let mut group = c.benchmark_group(format!("simd_luma/{backend}"));
    group.bench_function("luma_into_4k", |b| {
        b.iter_batched(
            || {
                let pixels = prepare_pixels(4096);
                let mask = vec![0.0_f32; pixels.len() / 4];
                (pixels, mask)
            },
            |(pixels, mut mask)| -> SimdResult<(Vec<f32>, Vec<f32>)> {
                let mut pixels = pixels;
                torwell84::lib::simd::luma_into(&mut pixels, &mut mask)?;
                Ok((pixels, mask))
            },
            BatchSize::SmallInput,
        )
        .unwrap();
    });
    group.finish();
}

criterion_group!(pixel_filters, bench_gain, bench_bias, bench_luma);
criterion_main!(pixel_filters);
