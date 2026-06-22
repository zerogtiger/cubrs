use criterion::{Criterion, criterion_group, criterion_main};
use cube_timer::{cubie::*, solver::Solver};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use rand_chacha::ChaCha8Rng;
use std::{hint::black_box, time::Duration};

// fn fibonacci(n: u64) -> u64 {
//     match n {
//         0 => 1,
//         1 => 1,
//         n => fibonacci(n-1) + fibonacci(n-2),
//     }
// }

pub const MOVE_LIMIT: u8 = 20;
pub const COUNT: usize = 100;

fn solve_cube(
    solver: &Solver,
    corner_perm: u16,
    corner_orient: u16,
    edge_perm: u32,
    edge_orient: u16,
) {
    let mut cube = Cubie::default();
    cube.set_corner_permutation_coord(corner_perm);
    cube.set_corner_orientation_coord(corner_orient);
    cube.set_edge_permutation_coord(edge_perm);
    cube.set_edge_orientation_coord(edge_orient);
    let _ = solver.solve(&cube, MOVE_LIMIT);
}

fn criterion_benchmark(c: &mut Criterion) {
    const SEED: u64 = 1;

    let mut rng = ChaCha8Rng::seed_from_u64(SEED);

    let coords: Vec<(u16, u16, u32, u16)> = (0..COUNT)
        .map(|_| {
            loop {
                let mut cube: Cubie = Default::default();
                let cp = rand::random_range(0..CORNER_PERMUTATION_COUNT);
                let co = rand::random_range(0..CORNER_ORIENTATION_COUNT);
                let ep = rand::random_range(0..EDGE_PERMUTATION_COUNT);
                let eo = rand::random_range(0..EDGE_ORIENTATION_COUNT);
                cube.set_corner_orientation_coord(co);
                cube.set_corner_permutation_coord(cp);
                cube.set_edge_orientation_coord(eo);
                cube.set_edge_permutation_coord(ep);
                if cube.is_solvable() {
                    return (cp, co, ep, eo);
                }
            }
        })
        .collect();

    let solver = Solver::new();
    for (cp, co, ep, eo) in coords.iter() {
        let mut cube = Cubie::default();
        cube.set_corner_permutation_coord(*cp);
        cube.set_corner_orientation_coord(*co);
        cube.set_edge_permutation_coord(*ep);
        cube.set_edge_orientation_coord(*eo);
        let solution = solver.solve(&cube, MOVE_LIMIT).unwrap();
        cube = cube.apply_moves(&solution);
        assert_eq!(cube.corner_permutation_coord(), 0);
        assert_eq!(cube.corner_orientation_coord(), 0);
        assert_eq!(cube.edge_permutation_coord(), 0);
        assert_eq!(cube.edge_orientation_coord(), 0);
    }

    let mut group = c.benchmark_group("cube solver");
    group.sample_size(16);
    group.measurement_time(Duration::from_secs(60));
    group.noise_threshold(0.05);
    group.bench_function("100 Random states", |b| {
        b.iter(|| {
            for (cp, co, ep, eo) in coords.iter() {
                solve_cube(black_box(&solver), *cp, *co, *ep, *eo);
            }
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
