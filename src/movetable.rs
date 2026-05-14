use core::{
    assert,
    cmp::max,
    convert::{Into, TryFrom},
    default,
    fmt::Debug,
    usize,
};
use std::collections::VecDeque;
use std::sync::Arc;
use std::{collections::HashMap, fs::File};

use crate::{
    cubie::*,
    moves::{Move, SymMove},
};

// S(i) * M * S(i)^-1, where M: move; i: sym index
pub struct SymMoveTable {
    sym_move_table: Vec<u8>,
}

impl SymMoveTable {
    pub fn load_or_generate() -> Self {
        let mut ret = Self {
            sym_move_table: Vec::new(),
        };
        ret.generate_tables();
        ret
    }

    pub fn get_sym_move(&self, sym_idx: u8, move_action: u8) -> u8 {
        self.sym_move_table[Self::encode_sym_idx_move_action(sym_idx, move_action) as usize]
    }

    fn encode_sym_idx_move_action(sym_idx: u8, move_action: u8) -> u16 {
        sym_idx as u16 * 18 + move_action as u16
    }

    fn generate_tables(&mut self) {
        self.sym_move_table
            .resize(SYM_COUNT as usize * MOVE_COUNT as usize, 0);
        for sym_idx in 0..SYM_COUNT {
            for move_action in Move::ALL {
                match Move::move_cubie_to_move_action(
                    &(SymMove::sym_index_to_cubie_move(sym_idx)
                        * Move::move_action_to_move_cubie(move_action)
                        * SymMove::sym_index_to_inverse_cubie_move(sym_idx)),
                ) {
                    Ok(result_move_action) => {
                        self.sym_move_table[Self::encode_sym_idx_move_action(
                            sym_idx,
                            move_action as u8,
                        ) as usize] = result_move_action as u8;
                    }
                    Err(_) => {
                        println!("{sym_idx}, {move_action:?} failed");
                        panic!("Resulting move not primitive")
                    }
                }
            }
        }
    }
}

// Sym index, sym index -> sym index (multiplication)
pub struct SymMultTable {
    sym_mult_table: Vec<u8>,
}

impl SymMultTable {
    pub fn load_or_generate() -> Self {
        let mut ret = Self {
            sym_mult_table: Vec::new(),
        };
        ret.generate_tables();
        ret
    }

    pub fn get_sym_mult(&self, sym_idx_1: u8, sym_idx_2: u8) -> u8 {
        self.sym_mult_table[Self::encode_sym_idx_sym_idx(sym_idx_1, sym_idx_2) as usize]
    }

    fn encode_sym_idx_sym_idx(sym_idx_1: u8, sym_idx_2: u8) -> u16 {
        sym_idx_1 as u16 * SYM_COUNT as u16 + sym_idx_2 as u16
    }

    fn generate_tables(&mut self) {
        self.sym_mult_table
            .resize(SYM_COUNT as usize * SYM_COUNT as usize, 0);
        for sym_idx_1 in 0..SYM_COUNT {
            for sym_idx_2 in 0..SYM_COUNT {
                match SymMove::sym_action_to_sym_index(
                    &(SymMove::sym_index_to_cubie_move(sym_idx_1)
                        * SymMove::sym_index_to_cubie_move(sym_idx_2)),
                ) {
                    Ok(result_sym_move) => {
                        self.sym_mult_table
                            [Self::encode_sym_idx_sym_idx(sym_idx_1, sym_idx_2) as usize] =
                            result_sym_move;
                    }
                    Err(_) => panic!("Resulting multiplication not primitive"),
                }
            }
        }
    }
}

pub struct FlipUDSliceTable {
    class_idx_to_rep_encoded_raw_coord: Vec<u32>,
    rep_encoded_raw_coord_to_class_idx: HashMap<u32, u16>,
    class_idx_to_sym_state: Vec<u16>,
}

impl FlipUDSliceTable {
    pub fn load_or_generate() -> Self {
        // TODO: save to disk
        let mut ret = Self {
            class_idx_to_rep_encoded_raw_coord: Vec::new(),
            rep_encoded_raw_coord_to_class_idx: HashMap::new(),
            class_idx_to_sym_state: Vec::new(),
        };
        ret.generate_tables();
        ret
    }

    #[inline]
    fn encode_raw_coord(edge_orient_coord: u16, ud_slice_coord: u16) -> usize {
        edge_orient_coord as usize * UD_SLICE_COUNT as usize + ud_slice_coord as usize
    }

    // returns (edge orientation, ud slice)
    fn decode_raw_coord(raw_coord: u32) -> (u16, u16) {
        (
            (raw_coord / UD_SLICE_COUNT as u32) as u16,
            (raw_coord % UD_SLICE_COUNT as u32) as u16,
        )
    }

    fn set_sym_state(&mut self, class_idx: u16, index: usize) {
        self.class_idx_to_sym_state[class_idx as usize] |= 1 << index;
    }

    pub fn get_sym_states(&self, class_idx: u16) -> u16 {
        self.class_idx_to_sym_state[class_idx as usize]
    }

    fn generate_tables(&mut self) {
        let mut flip_ud_coord: u16 = 0;
        self.class_idx_to_rep_encoded_raw_coord
            .resize(FLIP_UD_SLICE_COUNT as usize, u32::MAX);
        self.class_idx_to_sym_state
            .resize(FLIP_UD_SLICE_COUNT as usize, 0);

        let mut raw_coord_used: Vec<bool> =
            vec![false; EDGE_ORIENTATION_COUNT as usize * UD_SLICE_COUNT as usize];

        for edge_orient_coord in 0..EDGE_ORIENTATION_COUNT {
            for ud_slice_coord in 0..UD_SLICE_COUNT {
                if raw_coord_used[Self::encode_raw_coord(edge_orient_coord, ud_slice_coord)] {
                    continue;
                }

                let min_coord = Self::encode_raw_coord(edge_orient_coord, ud_slice_coord) as u32;
                for sym_moves in 0..16 {
                    // TODO: make this faster
                    let mut cube: Cubie = Cubie::default();
                    cube.set_edge_orientation_coord(edge_orient_coord);
                    cube.set_ud_slice_coord(ud_slice_coord);
                    cube = SymMove::sym_index_to_inverse_cubie_move(sym_moves)
                        * cube
                        * SymMove::sym_index_to_cubie_move(sym_moves);

                    let new_raw_coord = Self::encode_raw_coord(
                        cube.edge_orientation_coord(),
                        cube.ud_slice_coord(),
                    );
                    raw_coord_used[new_raw_coord] = true;
                    if new_raw_coord as u32 == min_coord {
                        self.set_sym_state(flip_ud_coord, sym_moves as usize);
                    }
                }
                self.class_idx_to_rep_encoded_raw_coord[flip_ud_coord as usize] = min_coord;
                self.rep_encoded_raw_coord_to_class_idx
                    .insert(min_coord, flip_ud_coord);
                flip_ud_coord += 1;
            }
        }
    }

    pub fn raw_coord_to_sym_coord(&self, edge_orient_coord: u16, ud_slice_coord: u16) -> (u16, u8) {
        for i in 0..16 {
            let mut cube: Cubie = Cubie::default();
            cube.set_edge_orientation_coord(edge_orient_coord);
            cube.set_ud_slice_coord(ud_slice_coord);
            cube = SymMove::sym_index_to_cubie_move(i)
                * cube
                * SymMove::sym_index_to_inverse_cubie_move(i);
            match self.rep_encoded_raw_coord_to_class_idx.get(
                &(Self::encode_raw_coord(cube.edge_orientation_coord(), cube.ud_slice_coord())
                    as u32),
            ) {
                None => continue,
                Some(class_idx) => return (*class_idx, i),
            }
        }
        assert!(false);
        (0, 0)
    }

    // (class index, sym)
    pub fn decode_sym_coord(sym_coord: u32) -> (u16, u8) {
        ((sym_coord / 16) as u16, (sym_coord % 16) as u8)
    }

    pub fn encode_sym_coord(class_idx: u16, sym_idx: u8) -> u32 {
        class_idx as u32 * 16 + sym_idx as u32
    }

    pub fn class_idx_to_raw_coord(&self, class_idx: u16) -> (u16, u16) {
        Self::decode_raw_coord(self.class_idx_to_rep_encoded_raw_coord[class_idx as usize])
    }

    // edge orient, ud slice
    pub fn sym_coord_to_raw_coord(&self, class_idx: u16, sym_idx: u8) -> (u16, u16) {
        let (rep_eo, rep_uds) = self.class_idx_to_raw_coord(class_idx);
        let mut cube: Cubie = Cubie::default();
        cube.set_edge_orientation_coord(rep_eo);
        cube.set_ud_slice_coord(rep_uds);
        cube = SymMove::sym_index_to_inverse_cubie_move(sym_idx)
            * cube
            * SymMove::sym_index_to_cubie_move(sym_idx);
        (cube.edge_orientation_coord(), cube.ud_slice_coord())
    }
}

pub struct MoveTable {
    // phase 1
    pub corner_orient_table: Vec<u16>,
    pub flip_ud_slice_table: Vec<u32>,
    sym_move_table: Arc<SymMoveTable>,
    sym_mult_table: Arc<SymMultTable>,
    // phase 2
    pub corner_perm_table: Vec<u32>,
    pub phase2_edge_perm_table: Vec<u32>,
    pub phase2_ud_slice_table: Vec<u16>,
}

impl MoveTable {
    pub fn load_or_generate(
        flip_ud_slice_table: &FlipUDSliceTable,
        sym_move_table: Arc<SymMoveTable>,
        sym_mult_table: Arc<SymMultTable>,
    ) -> Self {
        // let file = File::create("movetable/.txt");
        let mut table: Self = Self {
            // phase 1
            corner_orient_table: Default::default(),
            flip_ud_slice_table: Default::default(),
            sym_move_table,
            sym_mult_table,
            // phase 2
            corner_perm_table: Default::default(),
            phase2_edge_perm_table: Default::default(),
            phase2_ud_slice_table: Default::default(),
        };

        // phase 1
        Self::generate_move_table(
            &mut table.corner_orient_table,
            0,
            CORNER_ORIENTATION_COUNT,
            |cube, coord| cube.set_corner_orientation_coord(coord),
            |cube| cube.corner_orientation_coord(),
        );
        Self::generate_move_table(
            &mut table.flip_ud_slice_table,
            0,
            FLIP_UD_SLICE_COUNT,
            |cube, flip_ud_coord| {
                let (edge_orient_coord, ud_slice_coord) =
                    flip_ud_slice_table.class_idx_to_raw_coord(flip_ud_coord);
                cube.set_edge_orientation_coord(edge_orient_coord);
                cube.set_ud_slice_coord(ud_slice_coord);
            },
            |cube| {
                let (class_idx, sym_idx) = flip_ud_slice_table
                    .raw_coord_to_sym_coord(cube.edge_orientation_coord(), cube.ud_slice_coord());
                FlipUDSliceTable::encode_sym_coord(class_idx, sym_idx)
            },
        );

        // phase 2
        Self::generate_move_table(
            &mut table.corner_perm_table,
            0,
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

    pub fn get_next_corner_orient_coord(&self, corner_orient_coord: u16, move_action: u8) -> u16 {
        Self::get_next_coord(
            &self.corner_orient_table,
            corner_orient_coord as usize,
            move_action,
        )
    }

    // flip_ud_coord, sym move
    pub fn get_next_flip_ud_slice_sym_coord(
        &self,
        flip_ud_slice_class_idx: u16,
        flip_ud_slice_sym_idx: u8,
        move_action: u8,
    ) -> (u16, u8) {
        let symmetry_move_action = self
            .sym_move_table
            .get_sym_move(flip_ud_slice_sym_idx, move_action);
        let sym_coord = Self::get_next_coord(
            &self.flip_ud_slice_table,
            flip_ud_slice_class_idx as usize,
            symmetry_move_action,
        );
        let (result_class_idx, result_sym_idx) = FlipUDSliceTable::decode_sym_coord(sym_coord);
        (
            result_class_idx,
            self.sym_mult_table
                .get_sym_mult(result_sym_idx, flip_ud_slice_sym_idx),
        )
    }

    pub fn get_next_corner_perm_coord(&self, corner_perm_coord: u32, move_action: u8) -> u32 {
        Self::get_next_coord(
            &self.corner_perm_table,
            corner_perm_coord as usize,
            move_action,
        )
    }
    pub fn get_next_phase2_edge_perm_coord(
        &self,
        phase2_edge_perm_coord: u32,
        move_action: u8,
    ) -> u32 {
        Self::get_next_coord(
            &self.phase2_edge_perm_table,
            phase2_edge_perm_coord as usize,
            move_action,
        )
    }
    pub fn get_next_phase2_ud_slice_coord(
        &self,
        phase2_ud_slice_coord: u16,
        move_action: u8,
    ) -> u16 {
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

    fn generate_move_table<T, U, FSet, FGet>(
        table: &mut Vec<T>,
        default_value: T,
        max_coord: U,
        mut set_coord_fn: FSet,
        get_coord_fn: FGet,
    ) where
        T: Copy,
        U: TryInto<usize> + TryFrom<usize>,
        <U as TryFrom<usize>>::Error: Debug,
        <U as TryInto<usize>>::Error: Debug,
        FSet: FnMut(&mut Cubie, U),
        FGet: Fn(&Cubie) -> T,
    {
        let max_coord: usize = max_coord.try_into().unwrap();
        table.resize(max_coord * 18, default_value);
        let mut cube: Cubie = Cubie::default();
        for coord in 0..max_coord {
            set_coord_fn(&mut cube, coord.try_into().unwrap());
            let mut move_idx = 0;
            for move_action in Move::ALL_UNIQUE {
                for _ in 0..3 {
                    cube = cube.apply_move(move_action);
                    Self::set_next_coord(table, coord, move_idx, get_coord_fn(&cube));
                    move_idx += 1;
                }
                cube = cube.apply_move(move_action);
            }
        }
    }

    fn generate_move_table_u16<FSet, FGet>(
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

pub struct TwistConjugateTable {
    twist_conjugate: Vec<u16>,
}

impl TwistConjugateTable {
    pub fn load_or_generate() -> Self {
        let mut ret = Self {
            twist_conjugate: Default::default(),
        };
        ret.generate_tables();
        ret
    }

    pub fn get_twist_conjugate(&self, corner_orient_coord: u16, sym_idx: u8) -> u16 {
        self.twist_conjugate
            [Self::encode_corner_orient_sym_idx(corner_orient_coord, sym_idx) as usize]
    }

    fn encode_corner_orient_sym_idx(corner_orient_coord: u16, sym_idx: u8) -> u16 {
        corner_orient_coord * SYM_COUNT as u16 + sym_idx as u16
    }

    fn generate_tables(&mut self) {
        let mut cube: Cubie = Cubie::default();
        self.twist_conjugate
            .resize(CORNER_ORIENTATION_COUNT as usize * SYM_COUNT as usize, 0);
        for corner_orient_coord in 0..CORNER_ORIENTATION_COUNT {
            for sym_idx in 0..SYM_COUNT {
                cube.set_corner_orientation_coord(corner_orient_coord);
                cube = SymMove::sym_index_to_cubie_move(sym_idx)
                    * cube
                    * SymMove::sym_index_to_inverse_cubie_move(sym_idx);
                self.twist_conjugate
                    [Self::encode_corner_orient_sym_idx(corner_orient_coord, sym_idx) as usize] =
                    cube.corner_orientation_coord();
            }
        }
    }
}

pub struct PruneTable {
    // phase 1 coordinate: corner orient x flip ud coord
    // 2187 x 64430 = 140,908,410
    phase1_table: Vec<u8>,
    // phase 2 coordinate: corner_perm (equiv) x phase2_edge_perm
    // 2768 x 40320 = 111,605,760
    phase2_table: Vec<u8>,
    twist_conjugate_table: Arc<TwistConjugateTable>,
}

impl PruneTable {
    pub fn load_or_generate(
        move_table: &MoveTable,
        twist_conjugate_table: Arc<TwistConjugateTable>,
    ) -> Self {
        let mut ret = Self {
            phase1_table: Default::default(),
            phase2_table: Default::default(),
            twist_conjugate_table,
        };
        ret.generate_phase1_prune_table(move_table);
        ret
    }

    pub fn get_phase_1_optimal_depth(
        &self,
        corner_orient_coord: u16,
        flip_ud_slice_class_idx: u16,
        flip_ud_slice_sym_idx: u8,
    ) -> u8 {
        let result_corner_orient_coord = self
            .twist_conjugate_table
            .get_twist_conjugate(corner_orient_coord, flip_ud_slice_sym_idx);
        self.get_phase1_table(result_corner_orient_coord, flip_ud_slice_class_idx)
    }

    fn encode_phase_1_coord(corner_orient_coord: u16, flip_ud_slice_coord: u16) -> u32 {
        CORNER_ORIENTATION_COUNT as u32 * flip_ud_slice_coord as u32 + corner_orient_coord as u32
    }

    // (corner orient, flip ud slice)
    fn decode_phase_1_coord(phase_1_coord: u32) -> (u16, u16) {
        (
            (phase_1_coord % CORNER_ORIENTATION_COUNT as u32) as u16,
            (phase_1_coord / CORNER_ORIENTATION_COUNT as u32) as u16,
        )
    }

    fn get_phase1_table(&self, corner_orient_coord: u16, flip_ud_slice_coord: u16) -> u8 {
        self.phase1_table
            [Self::encode_phase_1_coord(corner_orient_coord, flip_ud_slice_coord) as usize]
    }

    fn set_phase1_table(&mut self, corner_orient_coord: u16, flip_ud_slice_coord: u16, depth: u8) {
        self.phase1_table
            [Self::encode_phase_1_coord(corner_orient_coord, flip_ud_slice_coord) as usize] = depth;
    }

    fn generate_phase1_prune_table(&mut self, move_table: &MoveTable) {
        self.phase1_table.resize(
            CORNER_ORIENTATION_COUNT as usize * FLIP_UD_SLICE_COUNT as usize,
            u8::MAX,
        );

        self.set_phase1_table(0, 0, 0);
        // corner orient, flip ud coord
        let mut q: VecDeque<(u16, u16)> = VecDeque::new();
        q.push_back((0, 0));
        let mut curr = 0;
        let mut old = 0;
        while !q.is_empty() {
            curr += 1;
            if (curr / 1_000_000 != old) {
                old += 1;
                println!("{old}");
            }
            let top = q.pop_front().unwrap();
            let curr_depth = self.get_phase1_table(top.0, top.1);
            for move_action in Move::ALL {
                let next: (u16, u16) = (
                    move_table.get_next_corner_orient_coord(top.0, move_action as u8),
                    move_table
                        .get_next_flip_ud_slice_sym_coord(top.1, 0, move_action as u8)
                        .0, // class idx
                );
                match self.get_phase1_table(next.0, next.1) == u8::MAX {
                    true => {
                        q.push_back(next);
                        self.set_phase1_table(next.0, next.1, curr_depth + 1);
                    }
                    false => {}
                }
            }
        }
    }
}
