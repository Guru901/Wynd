#![cfg(feature = "bench")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;
use wynd::bench_support::{BroadcastContext, RoomContext};

// Create one multi-thread runtime for all async benches
fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn broadcast_benches(c: &mut Criterion) {
    let runtime = create_runtime();

    // Pre-create contexts outside the iterator
    let ctx_small = Arc::new(runtime.block_on(BroadcastContext::with_clients(32)));
    let ctx_large = Arc::new(runtime.block_on(BroadcastContext::with_clients(256)));

    c.bench_function("broadcaster_text_32", |b| {
        let ctx = ctx_small.clone();
        b.to_async(&runtime).iter(|| async {
            ctx.broadcaster.text(black_box("hello benches")).await;
        });
    });

    c.bench_function("broadcaster_text_256", |b| {
        let ctx = ctx_large.clone();
        b.to_async(&runtime).iter(|| async {
            ctx.broadcaster.text(black_box("hello benches")).await;
        });
    });
}

fn room_benches(c: &mut Criterion) {
    // A single-thread runtime makes sense for room tests
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let ctx_small = Arc::new(runtime.block_on(RoomContext::with_clients(16)));
    let ctx_large = Arc::new(runtime.block_on(RoomContext::with_clients(128)));

    c.bench_function("room_text_16", |b| {
        let ctx = ctx_small.clone();
        b.to_async(&runtime).iter(|| async {
            ctx.room.text(black_box("room broadcast")).await;
        });
    });

    c.bench_function("room_text_128", |b| {
        let ctx = ctx_large.clone();
        b.to_async(&runtime).iter(|| async {
            ctx.room.text(black_box("room broadcast")).await;
        });
    });
}

criterion_group!(wynd, broadcast_benches, room_benches);
criterion_main!(wynd);
