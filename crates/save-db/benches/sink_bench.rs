use criterion::{black_box, criterion_group, criterion_main, Criterion};

use civ_save_db::{format_session_saved_event_json, SaveDb};

fn sink_event_json(c: &mut Criterion) {
    c.bench_function("sink_format_session_saved_event_json", |b| {
        b.iter(|| {
            black_box(format_session_saved_event_json(
                "bench-session",
                "bench-save",
                "autosave",
                black_box(42_000),
                black_box(8_388_608),
            ))
        });
    });
}

fn sink_save_db_writes(c: &mut Criterion) {
    c.bench_function("sink_save_db_record_autosave_100_rows", |b| {
        b.iter(|| {
            let tempdir = tempfile::tempdir().expect("create tempdir");
            let db_path = tempdir.path().join("saves.sqlite");
            let db = SaveDb::open(&db_path).expect("open save db");
            for tick in 0..100 {
                db.record_autosave("bench-session", tick, "/tmp/bench.civis", 4096)
                    .expect("record autosave");
            }
            black_box(db.list_for_session("bench-session").expect("list saves").len())
        });
    });
}

criterion_group!(benches, sink_event_json, sink_save_db_writes);
criterion_main!(benches);
