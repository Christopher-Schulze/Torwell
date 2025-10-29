use criterion::{black_box, criterion_group, criterion_main, Criterion};
use torwell84::load_bridge_presets_from_str;

fn bench_bridge_presets(c: &mut Criterion) {
  let data = include_str!("../src/lib/bridge_presets.json");
  c.bench_function("bridge_presets_parse", |b| {
    b.iter(|| {
      let presets = load_bridge_presets_from_str(black_box(data)).expect("presets should parse");
      black_box(presets)
    });
  });
}

criterion_group!(bootstrap, bench_bridge_presets);
criterion_main!(bootstrap);
