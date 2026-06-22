use core::convert::From;
use std::sync::Arc;

use crate::{
    cubie::{
        CORNER_ORIENTATION_COUNT, CORNER_PERMUTATION_COUNT, Cubie, EDGE_ORIENTATION_COUNT,
        EDGE_PERMUTATION_COUNT,
    },
    moves::Move,
    movetable::{
        CornerPermSymTable, Edge8PosConjugateTable, FlipUDSliceTable, MoveTable, PruneTable,
        SymMoveTable, SymMultTable, TwistConjugateTable,
    },
};

pub struct Solver {
    prune_table: Arc<PruneTable>,
    move_table: Arc<MoveTable>,
    flip_ud_slice_table: Arc<FlipUDSliceTable>,
    corner_perm_sym_table: Arc<CornerPermSymTable>,
}

impl Solver {
    pub fn new() -> Self {
        let sym_mult_table = Arc::new(SymMultTable::load_or_generate());
        let sym_move_table = Arc::new(SymMoveTable::load_or_generate());
        let flip_ud_slice_table = Arc::new(FlipUDSliceTable::load_or_generate());
        let twist_conjugate_table = Arc::new(TwistConjugateTable::load_or_generate());
        let corner_perm_sym_table = Arc::new(CornerPermSymTable::load_or_generate());
        let edge8_pos_conjugate_table = Arc::new(Edge8PosConjugateTable::load_or_generate());
        let move_table = Arc::new(MoveTable::load_or_generate(
            &flip_ud_slice_table,
            &corner_perm_sym_table,
            sym_move_table,
            sym_mult_table,
        ));
        let prune_table = Arc::new(PruneTable::load_or_generate(
            &move_table,
            twist_conjugate_table,
            edge8_pos_conjugate_table,
            flip_ud_slice_table.clone(),
            corner_perm_sym_table.clone(),
        ));
        Self {
            prune_table,
            move_table,
            flip_ud_slice_table,
            corner_perm_sym_table,
        }
    }

    pub fn generate_scramble(&self, move_limit: u8) -> (Cubie, Vec<Move>) {
        loop {
            let mut cube: Cubie = Default::default();
            let co = rand::random_range(0..CORNER_ORIENTATION_COUNT);
            let eo = rand::random_range(0..EDGE_ORIENTATION_COUNT);
            let cp = rand::random_range(0..CORNER_PERMUTATION_COUNT);
            let ep = rand::random_range(0..EDGE_PERMUTATION_COUNT);
            cube.set_corner_orientation_coord(co);
            cube.set_corner_permutation_coord(cp);
            cube.set_edge_orientation_coord(eo);
            cube.set_edge_permutation_coord(ep);
            if cube.is_solvable() {
                let mut sol = self.solve(&cube, move_limit).unwrap();
                Move::invert_moves(&mut sol);
                return (cube, sol);
            }
        }
    }

    pub fn solve(&self, cube: &Cubie, move_limit: u8) -> Result<Vec<Move>, ()> {
        if !cube.is_solvable() {
            return Err(());
        }
        let corner_orient_coord = cube.corner_orientation_coord();
        let edge_orient_coord = cube.edge_orientation_coord();
        let ud_slice_coord = cube.ud_slice_coord();
        let (flip_ud_slice_class_idx, flip_ud_slice_sym_idx) = self
            .flip_ud_slice_table
            .raw_coord_to_sym_coord(edge_orient_coord, ud_slice_coord);
        let mut solution = Vec::new();
        let phase1_optimal_depth = self.prune_table.get_phase_1_optimal_depth(
            corner_orient_coord,
            flip_ud_slice_class_idx,
            flip_ud_slice_sym_idx,
        );
        for phase_1_limit in phase1_optimal_depth..move_limit {
            if self.solve_phase_1_recurse(
                corner_orient_coord,
                flip_ud_slice_class_idx,
                flip_ud_slice_sym_idx,
                phase_1_limit,
                phase_1_limit,
                move_limit,
                &mut solution,
                cube.corner_orientation_coord(),
                cube.edge_orientation_coord(),
                cube.corner_permutation_coord(),
                cube.edge_permutation_coord(),
                None,
                None,
                cube,
            ) {
                return Ok(solution);
            }
        }
        Err(())
    }

    fn solve_phase_1_recurse(
        &self,
        corner_orient_coord: u16,
        flip_ud_slice_class_idx: u16,
        flip_ud_slice_sym_idx: u8,
        remaining_move_count: u8,
        total_phase_1_move_limit: u8,
        max_move_limit: u8,
        solution: &mut Vec<Move>,
        orig_corner_orient: u16,
        orig_edge_orient: u16,
        orig_corner_perm: u16,
        orig_edge_perm: u32,
        last_move2: Option<u8>, // move before most recent move
        last_move1: Option<u8>, // most recent move
        cube: &Cubie,
    ) -> bool {
        if corner_orient_coord == 0 && flip_ud_slice_class_idx == 0 && remaining_move_count == 0 {
            let phase_2_move_limit = max_move_limit - total_phase_1_move_limit;
            let (corner_perm_sym_class_idx, corner_perm_sym_idx) = self
                .corner_perm_sym_table
                .raw_coord_to_sym_coord(cube.corner_permutation_coord());
            return self.solve_phase_2_recurse(
                corner_perm_sym_class_idx,
                corner_perm_sym_idx,
                cube.phase2_edge_permutation_coord(),
                cube.phase2_ud_slice_coord(),
                phase_2_move_limit,
                solution,
                last_move2,
                last_move1,
            );
        }
        if remaining_move_count == 0 {
            return false;
        }
        let forbidden_move_bitmask = match last_move1 {
            Some(last_move1) => match last_move2 {
                Some(last_move2) => {
                    if Move::is_same_dimension(last_move1, last_move2) {
                        Move::get_same_class_move_bitmask(last_move1)
                            | Move::get_same_class_move_bitmask(last_move2)
                    } else {
                        Move::get_same_class_move_bitmask(last_move1)
                    }
                }
                None => Move::get_same_class_move_bitmask(last_move1),
            },
            None => 0,
        };
        for move_action in Move::ALL {
            if ((1 << ((move_action as u8) as u32)) & forbidden_move_bitmask) != 0 {
                continue;
            }
            let next_corner_orient_coord = self
                .move_table
                .get_next_corner_orient_coord(corner_orient_coord, move_action as u8);
            let (next_flip_ud_slice_class_idx, next_flip_ud_slice_sym_idx) =
                self.move_table.get_next_flip_ud_slice_sym_coord(
                    flip_ud_slice_class_idx,
                    flip_ud_slice_sym_idx,
                    move_action as u8,
                );

            let next_optimal_depth = self.prune_table.get_phase_1_optimal_depth(
                next_corner_orient_coord,
                next_flip_ud_slice_class_idx,
                next_flip_ud_slice_sym_idx,
            );
            if next_optimal_depth == 0 && remaining_move_count > 1 {
                continue;
            }
            if next_optimal_depth <= remaining_move_count - 1 {
                solution.push(Move::from(move_action));
                if self.solve_phase_1_recurse(
                    next_corner_orient_coord,
                    next_flip_ud_slice_class_idx,
                    next_flip_ud_slice_sym_idx,
                    remaining_move_count - 1,
                    total_phase_1_move_limit,
                    max_move_limit,
                    solution,
                    orig_corner_orient,
                    orig_edge_orient,
                    orig_corner_perm,
                    orig_edge_perm,
                    last_move1,
                    Some(move_action as u8),
                    &cube.apply_move(Move::from(move_action)),
                ) {
                    return true;
                }
                solution.pop();
            }
        }
        false
    }

    fn solve_phase_2_recurse(
        &self,
        corner_perm_sym_class_idx: u16,
        corner_perm_sym_idx: u8,
        phase2_edge_perm_coord: u16,
        phase2_ud_slice_coord: u8,
        remaining_move_count: u8,
        solution: &mut Vec<Move>,
        last_move2: Option<u8>, // move before most recent move
        last_move1: Option<u8>, // most recent move
    ) -> bool {
        if corner_perm_sym_class_idx == 0
            && phase2_edge_perm_coord == 0
            && phase2_ud_slice_coord == 0
        {
            return true;
        }
        if remaining_move_count == 0 {
            return false;
        }
        let forbidden_move_bitmask = match last_move1 {
            Some(last_move1) => match last_move2 {
                Some(last_move2) => {
                    if Move::is_same_dimension(last_move1, last_move2) {
                        Move::get_same_class_move_bitmask(last_move1)
                            | Move::get_same_class_move_bitmask(last_move2)
                    } else {
                        Move::get_same_class_move_bitmask(last_move1)
                    }
                }
                None => Move::get_same_class_move_bitmask(last_move1),
            },
            None => 0,
        };
        for move_action in Move::G1PRESERVING {
            if ((1 << ((move_action as u8) as u32)) & forbidden_move_bitmask) != 0 {
                continue;
            }
            let next_phase2_edge_perm_coord = self
                .move_table
                .get_next_phase2_edge_perm_coord(phase2_edge_perm_coord, move_action as u8);
            let (next_corner_perm_sym_class_idx, next_corner_perm_sym_idx) =
                self.move_table.get_next_corner_perm_sym_coord(
                    corner_perm_sym_class_idx,
                    corner_perm_sym_idx,
                    move_action as u8,
                );
            let next_phase2_ud_slice_coord = self
                .move_table
                .get_next_phase2_ud_slice_coord(phase2_ud_slice_coord, move_action as u8);
            if self.prune_table.get_phase_2_optimal_depth(
                next_corner_perm_sym_class_idx,
                next_corner_perm_sym_idx,
                next_phase2_edge_perm_coord,
            ) <= remaining_move_count - 1
            {
                solution.push(move_action);
                if self.solve_phase_2_recurse(
                    next_corner_perm_sym_class_idx,
                    next_corner_perm_sym_idx,
                    next_phase2_edge_perm_coord,
                    next_phase2_ud_slice_coord,
                    remaining_move_count - 1,
                    solution,
                    last_move1,
                    Some(move_action as u8),
                ) {
                    return true;
                }
                solution.pop();
            }
        }
        false
    }

    fn solve_phase_1_optimal(
        &self,
        corner_orient_coord: u16,
        edge_orient_coord: u16,
        ud_slice_coord: u16,
        move_limit: u8,
    ) -> Result<Vec<Move>, ()> {
        let (flip_ud_slice_class_idx, flip_ud_slice_sym_idx) = self
            .flip_ud_slice_table
            .raw_coord_to_sym_coord(edge_orient_coord, ud_slice_coord);

        let optimal_depth = self.prune_table.get_phase_1_optimal_depth(
            corner_orient_coord,
            flip_ud_slice_class_idx,
            flip_ud_slice_sym_idx,
        );
        if optimal_depth > move_limit {
            return Err(());
        }
        let mut ret = Vec::new();
        self.solve_phase_1_optimal_recurse(
            corner_orient_coord,
            flip_ud_slice_class_idx,
            flip_ud_slice_sym_idx,
            optimal_depth,
            &mut ret,
        );
        Ok(ret)
    }

    fn solve_phase_1_optimal_recurse(
        &self,
        corner_orient_coord: u16,
        flip_ud_slice_class_idx: u16,
        flip_ud_slice_sym_idx: u8,
        move_limit: u8,
        solution: &mut Vec<Move>,
    ) {
        if corner_orient_coord == 0 && flip_ud_slice_class_idx == 0 {
            return;
        }
        for move_action in Move::ALL {
            let next_corner_orient_coord = self
                .move_table
                .get_next_corner_orient_coord(corner_orient_coord, move_action as u8);
            let (next_flip_ud_slice_class_idx, next_flip_ud_slice_sym_idx) =
                self.move_table.get_next_flip_ud_slice_sym_coord(
                    flip_ud_slice_class_idx,
                    flip_ud_slice_sym_idx,
                    move_action as u8,
                );

            if self.prune_table.get_phase_1_optimal_depth(
                next_corner_orient_coord,
                next_flip_ud_slice_class_idx,
                next_flip_ud_slice_sym_idx,
            ) <= move_limit - 1
            {
                solution.push(Move::from(move_action));
                self.solve_phase_1_optimal_recurse(
                    next_corner_orient_coord,
                    next_flip_ud_slice_class_idx,
                    next_flip_ud_slice_sym_idx,
                    move_limit - 1,
                    solution,
                );
                break;
            }
        }
    }

    fn solve_phase_2_optimal(
        &self,
        corner_perm_coord: u16,
        phase2_edge_perm_coord: u16,
        _phase2_ud_slice_coord: u8,
        move_limit: u8,
    ) -> Result<Vec<Move>, ()> {
        let (corner_perm_sym_class_idx, corner_perm_sym_idx) = self
            .corner_perm_sym_table
            .raw_coord_to_sym_coord(corner_perm_coord);

        let optimal_depth = self.prune_table.get_phase_2_optimal_depth(
            corner_perm_sym_class_idx,
            corner_perm_sym_idx,
            phase2_edge_perm_coord,
        );
        if optimal_depth > move_limit {
            return Err(());
        }
        let mut ret = Vec::new();
        self.solve_phase_2_optimal_recurse(
            corner_perm_sym_class_idx,
            corner_perm_sym_idx,
            phase2_edge_perm_coord,
            optimal_depth,
            &mut ret,
        );
        Ok(ret)
    }

    fn solve_phase_2_optimal_recurse(
        &self,
        corner_perm_sym_class_idx: u16,
        corner_perm_sym_idx: u8,
        phase2_edge_perm_coord: u16,
        move_limit: u8,
        solution: &mut Vec<Move>,
    ) {
        if corner_perm_sym_class_idx == 0 && phase2_edge_perm_coord == 0 {
            return;
        }
        for move_action in Move::G1PRESERVING {
            let next_phase2_edge_perm_coord = self
                .move_table
                .get_next_phase2_edge_perm_coord(phase2_edge_perm_coord, move_action as u8);
            let (next_corner_perm_sym_class_idx, next_corner_perm_sym_idx) =
                self.move_table.get_next_corner_perm_sym_coord(
                    corner_perm_sym_class_idx,
                    corner_perm_sym_idx,
                    move_action as u8,
                );

            if self.prune_table.get_phase_2_optimal_depth(
                next_corner_perm_sym_class_idx,
                next_corner_perm_sym_idx,
                next_phase2_edge_perm_coord,
            ) <= move_limit - 1
            {
                solution.push(Move::from(move_action));
                self.solve_phase_2_optimal_recurse(
                    next_corner_perm_sym_class_idx,
                    next_corner_perm_sym_idx,
                    next_phase2_edge_perm_coord,
                    move_limit - 1,
                    solution,
                );
                break;
            }
        }
    }
}
