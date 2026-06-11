use criterion::{black_box, criterion_group, criterion_main, Criterion};

use civ_emergence_metrics::{shannon::ShannonEntropy, Histogram, Metric};

fn analyzer_entropy(c: &mut Criterion) {
    let histogram = Histogram::from_counts((1..=512).map(|n| n * 17).collect());
    let metric = ShannonEntropy;

    c.bench_function("analyzer_shannon_entropy_512_bins", |b| {
        b.iter(|| black_box(metric.compute(black_box(&histogram))));
    });
}

criterion_group!(benches, analyzer_entropy);
criterion_main!(benches);
