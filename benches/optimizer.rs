use criterion::*;
use cut_optimizer_2d::*;
use rand::prelude::*;

fn build_optimizer() -> Optimizer {
    let mut rng: StdRng = SeedableRng::seed_from_u64(1);

    let mut optimizer = Optimizer::new();
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::ParallelToWidth,
        price: 0,
        quantity: None,
    });
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 96,
        pattern_direction: PatternDirection::ParallelToLength,
        price: 0,
        quantity: None,
    });
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 120,
        pattern_direction: PatternDirection::ParallelToWidth,
        price: 0,
        quantity: None,
    });
    optimizer.add_stock_piece(StockPiece {
        width: 48,
        length: 120,
        pattern_direction: PatternDirection::ParallelToLength,
        price: 0,
        quantity: None,
    });

    let num_cut_pieces = 20;

    for i in 0..num_cut_pieces {
        optimizer.add_cut_piece(CutPiece {
            external_id: Some(i),
            width: rng.gen_range(1..=48),
            length: rng.gen_range(1..=120),
            pattern_direction: if rng.gen_bool(0.5) {
                PatternDirection::ParallelToWidth
            } else {
                PatternDirection::ParallelToLength
            },
            can_rotate: true,
        });
    }

    optimizer
}

pub fn benchmark_guillotine(c: &mut Criterion) {
    c.bench_function("guillotine random cut pieces", |b| b.iter(|| {
        let _ = build_optimizer()
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_guillotine(|_| {});
    }));
}

pub fn benchmark_maxrects(c: &mut Criterion) {
    c.bench_function("maxrects random cut pieces", |b| b.iter(|| {
        let _ = build_optimizer()
            .set_cut_width(1)
            .set_random_seed(1)
            .optimize_guillotine(|_| {});
    }));
}

criterion_group!(benches, benchmark_guillotine, benchmark_maxrects);
criterion_main!(benches);
