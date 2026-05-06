use core::{
    convert::{Into, TryFrom},
    default,
    fmt::Debug,
};
use std::fs::File;

use crate::{
    cubie::{
        CORNER_ORIENTATION_COUNT, CORNER_PERMUTATION_COUNT, Cubie, EDGE_ORIENTATION_COUNT,
        PHASE2_EDGE_PERMUTATION_COUNT, PHASE2_UD_SLICE_COUNT, UD_SLICE_COUNT,
    },
    moves::Move,
};

pub struct MoveTable {
    // phase 1
    pub corner_orient_table: Vec<u16>,
    pub edge_orient_table: Vec<u16>,
    pub ud_slice_table: Vec<u16>,
    // phase 2
    pub corner_perm_table: Vec<u32>,
    pub phase2_edge_perm_table: Vec<u32>,
    pub phase2_ud_slice_table: Vec<u16>,
}

impl MoveTable {
    pub fn load_or_generate() -> Self {
        // let file = File::create("movetable/.txt");
        let mut table: Self = Self {
            corner_orient_table: Default::default(),
            edge_orient_table: Default::default(),
            ud_slice_table: Default::default(),
            corner_perm_table: Default::default(),
            phase2_edge_perm_table: Default::default(),
            phase2_ud_slice_table: Default::default(),
        };
        Self::generate_move_table_u16(
            &mut table.corner_orient_table,
            CORNER_ORIENTATION_COUNT,
            |cube, coord| cube.set_corner_orientation_coord(coord),
            |cube| cube.corner_orientation_coord(),
        );
        Self::generate_move_table_u16(
            &mut table.edge_orient_table,
            EDGE_ORIENTATION_COUNT,
            |cube, coord| cube.set_edge_orientation_coord(coord),
            |cube| cube.edge_orientation_coord(),
        );
        Self::generate_move_table_u16(
            &mut table.ud_slice_table,
            UD_SLICE_COUNT,
            |cube, coord| cube.set_ud_slice_coord(coord),
            |cube| cube.ud_slice_coord(),
        );
        Self::generate_move_table_u32(
            &mut table.corner_perm_table,
            CORNER_PERMUTATION_COUNT,
            |cube, coord| cube.set_corner_permutation_coord(coord),
            |cube| cube.corner_permutation_coord(),
        );
        Self::generate_move_table_u32(
            &mut table.phase2_edge_perm_table,
            PHASE2_EDGE_PERMUTATION_COUNT,
            |cube, coord| cube.set_phase2_edge_permutation_coord(coord),
            |cube| cube.phase2_edge_permutation_coord(),
        );
        Self::generate_move_table_u16(
            &mut table.phase2_ud_slice_table,
            PHASE2_UD_SLICE_COUNT,
            |cube, coord| cube.set_phase2_ud_slice_coord(coord),
            |cube| cube.phase2_ud_slice_coord(),
        );
        table
    }

    pub fn get_corner_orient_coord(&self, corner_orient_coord: u16, move_action: u8) -> u16 {
        Self::get_next_coord(
            &self.corner_orient_table,
            corner_orient_coord as usize,
            move_action,
        )
    }

    pub fn get_edge_orient_coord(&self, edge_orient_coord: u16, move_action: u8) -> u16 {
        Self::get_next_coord(
            &self.edge_orient_table,
            edge_orient_coord as usize,
            move_action,
        )
    }

    pub fn get_ud_slice_coord(&self, ud_slice_coord: u16, move_action: u8) -> u16 {
        Self::get_next_coord(
            &self.ud_slice_table,
            ud_slice_coord as usize,
            move_action,
        )
    }
    pub fn get_corner_perm_coord(&self, corner_perm_coord: u32, move_action: u8) -> u32 {
        Self::get_next_coord(
            &self.corner_perm_table,
            corner_perm_coord as usize,
            move_action,
        )
    }
    pub fn get_phase2_edge_perm_coord(&self, phase2_edge_perm_coord: u32, move_action: u8) -> u32 {
        Self::get_next_coord(
            &self.phase2_edge_perm_table,
            phase2_edge_perm_coord as usize,
            move_action,
        )
    }
    pub fn get_phase2_ud_slice_coord(&self, phase2_ud_slice_coord: u16, move_action: u8) -> u16 {
        Self::get_next_coord(
            &self.phase2_ud_slice_table,
            phase2_ud_slice_coord as usize,
            move_action,
        )
    }

    fn get_next_coord<T: Copy>(table: &Vec<T>, coord: usize, move_action: u8) -> T {
        table[coord * 18 + move_action as usize]
    }

    fn set_next_coord<T>(table: &mut Vec<T>, coord: usize, move_action: u8, value: T) {
        table[coord * 18 + move_action as usize] = value;
    }

    // Intentionally duplicated for u16 and u32 instead of generics.
    // numeric conversion traits like From, Into, TryFrom make abstraction unnecessarily complex
    // especially only for two types; 
    // for some reason u32 does not implement Into<usize>
    // tried workarounds for like an hour and thought I've learned enough can spend time better
    fn generate_move_table_u16<FSet, FGet> (
        table: &mut Vec<u16>,
        max_coord: u16,
        mut set_coord_fn: FSet,
        get_coord_fn: FGet,
    ) where
        FSet: FnMut(&mut Cubie, u16),
        FGet: Fn(&Cubie) -> u16,
    {
        table.resize(max_coord as usize * 18, 0);
        let mut cube: Cubie = Cubie::default();
        for coord in 0..max_coord {
            set_coord_fn(&mut cube, coord);
            let mut move_idx = 0;
            for move_action in Move::ALL_UNIQUE {
                for it in 0..4 {
                    cube = cube.apply_move(move_action);
                    if it != 3 {
                        Self::set_next_coord(table, coord as usize, move_idx, get_coord_fn(&cube));
                        move_idx += 1;
                    }
                }
            }
        }
    }

    fn generate_move_table_u32<FSet, FGet>(
        table: &mut Vec<u32>,
        max_coord: u32,
        mut set_coord_fn: FSet,
        get_coord_fn: FGet,
    ) where
        FSet: FnMut(&mut Cubie, u32),
        FGet: Fn(&Cubie) -> u32,
    {
        table.resize(max_coord as usize * 18, 0);
        let mut cube: Cubie = Cubie::default();
        for coord in 0..max_coord {
            set_coord_fn(&mut cube, coord);
            let mut move_idx = 0;
            for move_action in Move::ALL_UNIQUE {
                for it in 0..4 {
                    cube = cube.apply_move(move_action);
                    if it != 3 {
                        Self::set_next_coord(table, coord as usize, move_idx, get_coord_fn(&cube));
                        move_idx += 1;
                    }
                }
            }
        }
    }
}
