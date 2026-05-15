use core::convert::From;
use std::sync::Arc;

use crate::{
    cubie::{MOVE_COUNT, UD_SLICE_COUNT},
    moves::Move,
    movetable::{FlipUDSliceTable, MoveTable, PruneTable},
};

pub struct Solver {
    prune_table: Arc<PruneTable>,
    move_table: Arc<MoveTable>,
    flip_ud_slice_table: Arc<FlipUDSliceTable>,
}

impl Solver {
    pub fn new(
        prune_table: Arc<PruneTable>,
        move_table: Arc<MoveTable>,
        flip_ud_slice_table: Arc<FlipUDSliceTable>,
    ) -> Self {
        Self {
            prune_table,
            move_table,
            flip_ud_slice_table,
        }
    }
    pub fn solve_phase_1_optimal(
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
        self.solve_phase_1_recurse(
            corner_orient_coord,
            flip_ud_slice_class_idx,
            flip_ud_slice_sym_idx,
            optimal_depth,
            &mut ret,
        );
        Ok(ret)
    }

    fn solve_phase_1_recurse(
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
                self.solve_phase_1_recurse(
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
}
