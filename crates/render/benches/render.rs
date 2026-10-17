// SPDX-License-Identifier: GPL-3.0-or-later

use criterion::{Criterion, criterion_group, criterion_main};
use fractalforge_core::Params;
use fractalforge_render::{Control, PassMode, Render};

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("render 640x480");
    group.sample_size(10);
    for mode in [PassMode::One, PassMode::Guess] {
        for name in ["mandel", "julia"] {
            let params = Params::for_type(name).unwrap();
            group.bench_function(format!("{name} {mode:?}"), |b| {
                b.iter(|| {
                    let mut r = Render::new(params.clone(), 640, 480, mode).unwrap();
                    r.run(&Control::default());
                })
            });
        }
    }
    group.finish();

    // Cost of taking a checkpoint mid-render (cloning state and buffer).
    let mut r = Render::new(
        Params::for_type("mandel").unwrap(),
        1920,
        1080,
        PassMode::Guess,
    )
    .unwrap();
    r.run(&Control {
        stop_after: Some(0.5),
        ..Default::default()
    });
    c.bench_function("checkpoint 1920x1080", |b| {
        b.iter(|| (r.checkpoint(), r.buffer().clone()))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
