use criterion::{Criterion, criterion_group, criterion_main};
use cube_timer::{cubie::*, moves::Move, solver::Solver};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use rand_chacha::ChaCha8Rng;
use std::{default, hint::black_box, time::Duration};

pub const MOVE_LIMIT: u8 = 21;

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

fn multi_solve_benchmark(c: &mut Criterion) {
    const COUNT: usize = 1000;
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
    group.sample_size(20);
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

fn single_superflip(c: &mut Criterion) {
    let superflip_move_list =
        Move::move_list_from_str("U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2").unwrap();
    let mut cube = Cubie::default();
    cube = cube.apply_moves(&superflip_move_list);

    let solver = Solver::new();

    let solution = solver.solve(&cube, MOVE_LIMIT).unwrap();
    let solved_cube = cube.apply_moves(&solution);
    assert_eq!(solved_cube.corner_permutation_coord(), 0);
    assert_eq!(solved_cube.corner_orientation_coord(), 0);
    assert_eq!(solved_cube.edge_permutation_coord(), 0);
    assert_eq!(solved_cube.edge_orientation_coord(), 0);

    let mut group = c.benchmark_group("cube solver");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(60));
    group.noise_threshold(0.05);
    group.bench_function("superflip", |b| {
        b.iter(|| {
            solve_cube(
                black_box(&solver),
                cube.corner_permutation_coord(),
                cube.corner_orientation_coord(),
                cube.edge_permutation_coord(),
                cube.edge_orientation_coord(),
            );
        })
    });
    group.finish();
}

criterion_group!(benches, multi_solve_benchmark);
// criterion_group!(benches, single_superflip);
criterion_main!(benches);
