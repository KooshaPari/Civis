use criterion::{black_box, criterion_group, criterion_main, Criterion};

use civ_traffic::TrafficGraph;
use civ_voxel::WorldCoord;

fn coord(x: i64, z: i64) -> WorldCoord {
    WorldCoord { x, y: 0, z }
}

fn transformer_record_traffic(c: &mut Criterion) {
    let path: Vec<_> = (0..512).map(|i| (coord(i, 0), coord(i + 1, 0))).collect();

    c.bench_function("transformer_record_traffic_512_edges", |b| {
        b.iter(|| {
            let mut graph = TrafficGraph::new();
            for (from, to) in &path {
                black_box(graph.record_traffic(*from, *to, 1.0));
            }
            black_box(graph.segments.len())
        });
    });
}

criterion_group!(benches, transformer_record_traffic);
criterion_main!(benches);
