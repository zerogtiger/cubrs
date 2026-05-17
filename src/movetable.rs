use std::io::{BufReader, BufWriter, Read, Write, stdout};

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

use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};

use crate::{
    cubie::*,
    moves::{Move, SymMove},
};

fn save_vec<T: bytemuck::Pod>(path: &str, vec: &Vec<T>) {
    let p = std::path::Path::new(path);

    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = BufWriter::new(File::create(path).unwrap());
    let bytes = bytemuck::cast_slice(vec.as_slice());
    f.write_all(&(vec.len() as u64).to_le_bytes()).unwrap();
    f.write_all(bytes).unwrap();
}

fn load_vec<T: bytemuck::Pod + Default + Clone>(path: &str) -> Vec<T> {
    let mut f = BufReader::new(File::open(path).unwrap());
    let mut len_bytes = [0u8; 8];
    f.read_exact(&mut len_bytes).unwrap();
    let len = u64::from_le_bytes(len_bytes) as usize;
    let mut vec = vec![T::default(); len];
    f.read_exact(bytemuck::cast_slice_mut(vec.as_mut_slice()))
        .unwrap();
    vec
}

fn save_hashmap<K, V>(path: &str, map: &HashMap<K, V>)
where
    K: bytemuck::Pod + bytemuck::Zeroable,
    V: bytemuck::Pod + bytemuck::Zeroable,
{
    let p = std::path::Path::new(path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = BufWriter::new(File::create(path).unwrap());
    f.write_all(&(map.len() as u64).to_le_bytes()).unwrap();
    for (k, v) in map.iter() {
        f.write_all(bytemuck::bytes_of(k)).unwrap();
        f.write_all(bytemuck::bytes_of(v)).unwrap();
    }
}

fn load_hashmap<K, V>(path: &str) -> HashMap<K, V>
where
    K: bytemuck::Pod + bytemuck::Zeroable + Eq + std::hash::Hash,
    V: bytemuck::Pod + bytemuck::Zeroable,
{
    let mut f = BufReader::new(File::open(path).unwrap());
    let mut len_bytes = [0u8; 8];
    f.read_exact(&mut len_bytes).unwrap();
    let len = u64::from_le_bytes(len_bytes) as usize;
    let mut map = HashMap::with_capacity(len);
    let k_size = std::mem::size_of::<K>();
    let v_size = std::mem::size_of::<V>();
    for _ in 0..len {
        let mut k_bytes = vec![0u8; k_size];
        let mut v_bytes = vec![0u8; v_size];
        f.read_exact(&mut k_bytes).unwrap();
        f.read_exact(&mut v_bytes).unwrap();
        let k = *bytemuck::from_bytes::<K>(&k_bytes);
        let v = *bytemuck::from_bytes::<V>(&v_bytes);
        map.insert(k, v);
    }
    map
}

fn raw_mode_print(str: &str) {
    enable_raw_mode().unwrap();
    execute!(
        stdout(),
        cursor::MoveToColumn(0),
        Clear(ClearType::CurrentLine)
    )
    .unwrap();
    print!("{}", str);
    stdout().flush().unwrap();
    disable_raw_mode().unwrap();
}

// S(i) * M * S(i)^-1, where M: move; i: sym index
pub struct SymMoveTable {
    sym_move_table: Vec<u8>,
}

impl SymMoveTable {
    pub fn load_or_generate() -> Self {
        if std::path::Path::new("tables/sym_move_table.bin").exists() {
            return Self {
                sym_move_table: load_vec("tables/sym_move_table.bin"),
            };
        }
        let mut ret = Self {
            sym_move_table: Vec::new(),
        };
        raw_mode_print("Generating sym move table");
        ret.generate_tables();
        save_vec("tables/sym_move_table.bin", &ret.sym_move_table);
        raw_mode_print("sym move table generated: stored at tables/sym_move_table.bin");
        println!("");
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
        if std::path::Path::new("tables/sym_mult_table.bin").exists() {
            return Self {
                sym_mult_table: load_vec("tables/sym_mult_table.bin"),
            };
        }
        let mut ret = Self {
            sym_mult_table: Vec::new(),
        };
        raw_mode_print("Generating sym mult table");
        ret.generate_tables();
        save_vec("tables/sym_mult_table.bin", &ret.sym_mult_table);
        raw_mode_print("sym mult table generated: stored at tables/sym_mult_table.bin");
        println!("");
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
        if std::path::Path::new("tables/flip_ud_class_to_rep.bin").exists()
            && std::path::Path::new("tables/flip_ud_rep_to_class.bin").exists()
            && std::path::Path::new("tables/flip_ud_class_to_sym.bin").exists()
        {
            return Self {
                class_idx_to_rep_encoded_raw_coord: load_vec("tables/flip_ud_class_to_rep.bin"),
                rep_encoded_raw_coord_to_class_idx: load_hashmap("tables/flip_ud_rep_to_class.bin"),
                class_idx_to_sym_state: load_vec("tables/flip_ud_class_to_sym.bin"),
            };
        }
        let mut ret = Self {
            class_idx_to_rep_encoded_raw_coord: Vec::new(),
            rep_encoded_raw_coord_to_class_idx: HashMap::new(),
            class_idx_to_sym_state: Vec::new(),
        };
        raw_mode_print("Generating flip ud slice tables");
        ret.generate_tables();
        save_vec(
            "tables/flip_ud_class_to_rep.bin",
            &ret.class_idx_to_rep_encoded_raw_coord,
        );
        save_hashmap(
            "tables/flip_ud_rep_to_class.bin",
            &ret.rep_encoded_raw_coord_to_class_idx,
        );
        save_vec(
            "tables/flip_ud_class_to_sym.bin",
            &ret.class_idx_to_sym_state,
        );
        raw_mode_print("flip ud slice table generated: tables/flip_ud_*.bin");
        println!("");
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

pub struct CornerPermSymTable {
    class_idx_to_rep_raw_coord: Vec<u16>,
    rep_raw_coord_to_class_idx: HashMap<u16, u16>,
    class_idx_to_sym_state: Vec<u16>,
}

impl CornerPermSymTable {
    pub fn load_or_generate() -> Self {
        if std::path::Path::new("tables/corner_perm_sym_class_idx_to_rep.bin").exists()
            && std::path::Path::new("tables/corner_perm_sym_rep_to_class_idx.bin").exists()
            && std::path::Path::new("tables/corner_perm_sym_class_idx_to_sym.bin").exists()
        {
            return Self {
                class_idx_to_rep_raw_coord: load_vec("tables/corner_perm_sym_class_idx_to_rep.bin"),
                rep_raw_coord_to_class_idx: load_hashmap(
                    "tables/corner_perm_sym_rep_to_class_idx.bin",
                ),
                class_idx_to_sym_state: load_vec("tables/corner_perm_sym_class_idx_to_sym.bin"),
            };
        }
        let mut ret = Self {
            class_idx_to_rep_raw_coord: Vec::new(),
            rep_raw_coord_to_class_idx: HashMap::new(),
            class_idx_to_sym_state: Vec::new(),
        };
        raw_mode_print("Generating corner perm sym table");
        ret.generate_table();
        save_vec(
            "tables/corner_perm_sym_class_idx_to_rep.bin",
            &ret.class_idx_to_rep_raw_coord,
        );
        save_hashmap(
            "tables/corner_perm_sym_rep_to_class_idx.bin",
            &ret.rep_raw_coord_to_class_idx,
        );
        save_vec(
            "tables/corner_perm_sym_class_idx_to_sym.bin",
            &ret.class_idx_to_sym_state,
        );
        raw_mode_print("corner perm sym table generated: stored at tables/corner_perm_sym_*.bin");
        println!("");
        ret
    }

    pub fn encode_sym_coord(class_idx: u16, sym_idx: u8) -> u32 {
        class_idx as u32 * 16 + sym_idx as u32
    }

    pub fn decode_sym_coord(sym_coord: u32) -> (u16, u8) {
        ((sym_coord / 16) as u16, (sym_coord % 16) as u8)
    }

    fn set_sym_state(&mut self, class_idx: u16, index: usize) {
        self.class_idx_to_sym_state[class_idx as usize] |= 1 << index;
    }

    pub fn get_sym_states(&self, class_idx: u16) -> u16 {
        self.class_idx_to_sym_state[class_idx as usize]
    }

    // class idx, sym idx
    pub fn raw_coord_to_sym_coord(&self, corner_perm_coord: u16) -> (u16, u8) {
        for i in 0..16 {
            let mut cube: Cubie = Cubie::default();
            cube.set_corner_permutation_coord(corner_perm_coord);
            cube = SymMove::sym_index_to_cubie_move(i)
                * cube
                * SymMove::sym_index_to_inverse_cubie_move(i);
            match self
                .rep_raw_coord_to_class_idx
                .get(&cube.corner_permutation_coord())
            {
                None => continue,
                Some(class_idx) => return (*class_idx, i),
            }
        }
        assert!(false);
        (0, 0)
    }

    pub fn class_idx_to_raw_coord(&self, class_idx: u16) -> u16 {
        self.class_idx_to_rep_raw_coord[class_idx as usize]
    }

    fn generate_table(&mut self) {
        let mut corner_perm_sym_coord: u16 = 0;
        self.class_idx_to_rep_raw_coord
            .resize(CORNER_PERMUTATION_SYM_COUNT as usize, u16::MAX);
        self.class_idx_to_sym_state
            .resize(CORNER_PERMUTATION_SYM_COUNT as usize, 0);
        let mut raw_coord_used: Vec<bool> = vec![false; CORNER_PERMUTATION_COUNT as usize];

        for corner_perm_coord in 0..CORNER_PERMUTATION_COUNT {
            if raw_coord_used[corner_perm_coord as usize] {
                continue;
            }

            let rep = corner_perm_coord;
            for sym_idx in 0..16 {
                let mut cube = Cubie::default();
                cube.set_corner_permutation_coord(corner_perm_coord);
                cube = SymMove::sym_index_to_inverse_cubie_move(sym_idx)
                    * cube
                    * SymMove::sym_index_to_cubie_move(sym_idx);
                let new_corner_perm_coord = cube.corner_permutation_coord();
                raw_coord_used[new_corner_perm_coord as usize] = true;
                if new_corner_perm_coord == rep {
                    self.set_sym_state(corner_perm_sym_coord, sym_idx as usize);
                }
            }
            self.class_idx_to_rep_raw_coord[corner_perm_sym_coord as usize] = rep;
            self.rep_raw_coord_to_class_idx
                .insert(rep, corner_perm_sym_coord);
            corner_perm_sym_coord += 1;
        }
    }
}

pub struct MoveTable {
    // phase 1
    pub corner_orient_table: Vec<u16>,
    pub flip_ud_slice_table: Vec<u32>,
    sym_move_table: Arc<SymMoveTable>,
    sym_mult_table: Arc<SymMultTable>,
    // phase 2
    pub corner_perm_sym_table: Vec<u32>,
    pub phase2_edge_perm_table: Vec<u16>,
    pub phase2_ud_slice_table: Vec<u8>,
}

impl MoveTable {
    pub fn load_or_generate(
        flip_ud_slice_table: &FlipUDSliceTable,
        corner_perm_sym_table: &CornerPermSymTable,
        sym_move_table: Arc<SymMoveTable>,
        sym_mult_table: Arc<SymMultTable>,
    ) -> Self {
        // let file = File::create("movetable/.txt");
        if std::path::Path::new("tables/move_corner_orient.bin").exists()
            && std::path::Path::new("tables/move_flip_ud_slice.bin").exists()
            && std::path::Path::new("tables/move_corner_perm_sym.bin").exists()
            && std::path::Path::new("tables/move_phase2_edge_perm.bin").exists()
            && std::path::Path::new("tables/move_phase2_ud_slice.bin").exists()
        {
            return Self {
                corner_orient_table: load_vec("tables/move_corner_orient.bin"),
                flip_ud_slice_table: load_vec("tables/move_flip_ud_slice.bin"),
                sym_move_table,
                sym_mult_table,
                corner_perm_sym_table: load_vec("tables/move_corner_perm_sym.bin"),
                phase2_edge_perm_table: load_vec("tables/move_phase2_edge_perm.bin"),
                phase2_ud_slice_table: load_vec("tables/move_phase2_ud_slice.bin"),
            };
        }
        let mut table: Self = Self {
            // phase 1
            corner_orient_table: Default::default(),
            flip_ud_slice_table: Default::default(),
            sym_move_table,
            sym_mult_table,
            // phase 2
            corner_perm_sym_table: Default::default(),
            phase2_edge_perm_table: Default::default(),
            phase2_ud_slice_table: Default::default(),
        };

        raw_mode_print("Generating move table, this may take a few seconds");
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
            &mut table.corner_perm_sym_table,
            0,
            CORNER_PERMUTATION_SYM_COUNT,
            |cube, corner_perm_sym_coord| {
                let corner_perm_coord =
                    corner_perm_sym_table.class_idx_to_raw_coord(corner_perm_sym_coord);
                cube.set_corner_permutation_coord(corner_perm_coord);
            },
            |cube| {
                let (class_idx, sym_idx) =
                    corner_perm_sym_table.raw_coord_to_sym_coord(cube.corner_permutation_coord());
                CornerPermSymTable::encode_sym_coord(class_idx, sym_idx)
            },
        );
        Self::generate_move_table(
            &mut table.phase2_edge_perm_table,
            0,
            PHASE2_EDGE_PERMUTATION_COUNT,
            |cube, coord| cube.set_phase2_edge_permutation_coord(coord),
            |cube| cube.phase2_edge_permutation_coord(),
        );
        Self::generate_move_table(
            &mut table.phase2_ud_slice_table,
            0,
            PHASE2_UD_SLICE_COUNT,
            |cube, coord| cube.set_phase2_ud_slice_coord(coord),
            |cube| cube.phase2_ud_slice_coord(),
        );
        save_vec("tables/move_corner_orient.bin", &table.corner_orient_table);
        save_vec("tables/move_flip_ud_slice.bin", &table.flip_ud_slice_table);
        save_vec(
            "tables/move_corner_perm_sym.bin",
            &table.corner_perm_sym_table,
        );
        save_vec(
            "tables/move_phase2_edge_perm.bin",
            &table.phase2_edge_perm_table,
        );
        save_vec(
            "tables/move_phase2_ud_slice.bin",
            &table.phase2_ud_slice_table,
        );
        raw_mode_print("move table generated: stored at tables/move_*.bin");
        println!("");
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

    pub fn get_next_corner_perm_sym_coord(
        &self,
        corner_perm_sym_coord: u16,
        sym_idx: u8,
        move_action: u8,
    ) -> (u16, u8) {
        let sym_move = self.sym_move_table.get_sym_move(sym_idx, move_action);
        let sym_coord = Self::get_next_coord(
            &self.corner_perm_sym_table,
            corner_perm_sym_coord as usize,
            sym_move,
        );
        let (corner_perm_sym_class_idx, corner_perm_sym_idx) =
            CornerPermSymTable::decode_sym_coord(sym_coord);
        (
            corner_perm_sym_class_idx,
            self.sym_mult_table
                .get_sym_mult(corner_perm_sym_idx, sym_idx),
        )
    }

    pub fn get_next_phase2_edge_perm_coord(
        &self,
        phase2_edge_perm_coord: u16,
        move_action: u8,
    ) -> u16 {
        Self::get_next_coord(
            &self.phase2_edge_perm_table,
            phase2_edge_perm_coord as usize,
            move_action,
        )
    }

    pub fn get_next_phase2_ud_slice_coord(&self, phase2_ud_slice_coord: u8, move_action: u8) -> u8 {
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
}

pub struct TwistConjugateTable {
    twist_conjugate: Vec<u16>,
}

impl TwistConjugateTable {
    pub fn load_or_generate() -> Self {
        if std::path::Path::new("tables/twist_conjugate.bin").exists() {
            return Self {
                twist_conjugate: load_vec("tables/twist_conjugate.bin"),
            };
        }
        let mut ret = Self {
            twist_conjugate: Default::default(),
        };
        raw_mode_print("Generating twist conjugate table");
        ret.generate_tables();
        save_vec("tables/twist_conjugate.bin", &ret.twist_conjugate);
        raw_mode_print("sym move table generated: stored at tables/twist_conjugate.bin");
        println!("");
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

pub struct Edge8PosConjugateTable {
    edge8_pos_conjugate_table: Vec<u16>,
}

impl Edge8PosConjugateTable {
    pub fn load_or_generate() -> Self {
        if std::path::Path::new("tables/edge8_pos_conjugate.bin").exists() {
            return Self {
                edge8_pos_conjugate_table: load_vec("tables/edge8_pos_conjugate.bin"),
            };
        }
        let mut ret = Self {
            edge8_pos_conjugate_table: Vec::new(),
        };
        raw_mode_print("Generating edge 8 pos conjugate table");
        ret.generate_tables();
        save_vec(
            "tables/edge8_pos_conjugate.bin",
            &ret.edge8_pos_conjugate_table,
        );
        raw_mode_print(
            "edge8 pos conjugate table generated: stored at tables/edge8_pos_conjugate.bin",
        );
        println!("");
        ret
    }

    pub fn get_edge8_pos_conjugate(&self, phase2_edge_perm_coord: u16, sym_idx: u8) -> u16 {
        self.edge8_pos_conjugate_table
            [Self::encode_phase2_edge_perm_sym_idx(phase2_edge_perm_coord, sym_idx) as usize]
    }

    fn encode_phase2_edge_perm_sym_idx(phase2_edge_perm_coord: u16, sym_idx: u8) -> u32 {
        phase2_edge_perm_coord as u32 * SYM_COUNT as u32 + sym_idx as u32
    }

    fn generate_tables(&mut self) {
        let mut cube: Cubie = Cubie::default();
        self.edge8_pos_conjugate_table.resize(
            PHASE2_EDGE_PERMUTATION_COUNT as usize * SYM_COUNT as usize,
            0,
        );
        for phase2_edge_perm_coord in 0..PHASE2_EDGE_PERMUTATION_COUNT {
            for sym_idx in 0..SYM_COUNT {
                cube.set_phase2_edge_permutation_coord(phase2_edge_perm_coord);
                cube = SymMove::sym_index_to_cubie_move(sym_idx)
                    * cube
                    * SymMove::sym_index_to_inverse_cubie_move(sym_idx);
                self.edge8_pos_conjugate_table[Self::encode_phase2_edge_perm_sym_idx(
                    phase2_edge_perm_coord,
                    sym_idx,
                ) as usize] = cube.phase2_edge_permutation_coord();
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
    edge8_pos_conjugate_table: Arc<Edge8PosConjugateTable>,
    flip_ud_slice_table: Arc<FlipUDSliceTable>,
    corner_perm_sym_table: Arc<CornerPermSymTable>,
}

impl PruneTable {
    pub fn load_or_generate(
        move_table: &MoveTable,
        twist_conjugate_table: Arc<TwistConjugateTable>,
        edge8_pos_conjugate_table: Arc<Edge8PosConjugateTable>,
        flip_ud_slice_table: Arc<FlipUDSliceTable>,
        corner_perm_sym_table: Arc<CornerPermSymTable>,
    ) -> Self {
        if std::path::Path::new("tables/phase_1_prune.bin").exists()
            && std::path::Path::new("tables/phase_2_prune.bin").exists()
        {
            return Self {
                phase1_table: load_vec("tables/phase_1_prune.bin"),
                phase2_table: load_vec("tables/phase_2_prune.bin"),
                twist_conjugate_table,
                edge8_pos_conjugate_table,
                flip_ud_slice_table,
                corner_perm_sym_table,
            };
        }
        let mut ret = Self {
            phase1_table: Vec::new(),
            phase2_table: Vec::new(),
            twist_conjugate_table,
            edge8_pos_conjugate_table,
            flip_ud_slice_table,
            corner_perm_sym_table,
        };

        raw_mode_print("Generating prune tables, this may take a few minutes");
        println!("");
        raw_mode_print("Phase 1/2: ");
        println!("");
        ret.generate_phase1_prune_table(move_table);
        save_vec("tables/phase_1_prune.bin", &ret.phase1_table);
        raw_mode_print("phase 1 prune table generated: stored at tables/phase_1_prune.bin");
        println!("");
        raw_mode_print("Phase 2/2: ");
        println!("");
        ret.generate_phase2_prune_table(move_table);
        save_vec("tables/phase_2_prune.bin", &ret.phase2_table);
        raw_mode_print("phase 2 prune table generated: stored at tables/phase_2_prune.bin");
        println!("");
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

    pub fn get_phase_2_optimal_depth(
        &self,
        corner_perm_sym_class_idx: u16,
        corner_perm_sym_idx: u8,
        phase2_edge_perm_coord: u16,
    ) -> u8 {
        let result_edge_perm = self
            .edge8_pos_conjugate_table
            .get_edge8_pos_conjugate(phase2_edge_perm_coord, corner_perm_sym_idx);
        self.get_phase2_table(corner_perm_sym_class_idx, result_edge_perm)
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

    fn encode_phase_2_coord(corner_perm_sym_coord: u16, phase2_edge_perm_coord: u16) -> u32 {
        CORNER_PERMUTATION_SYM_COUNT as u32 * phase2_edge_perm_coord as u32
            + corner_perm_sym_coord as u32
    }

    // (corner perm sym, edge perm)
    fn decode_phase_2_coord(phase_2_coord: u32) -> (u16, u16) {
        (
            (phase_2_coord % CORNER_PERMUTATION_SYM_COUNT as u32) as u16,
            (phase_2_coord / CORNER_PERMUTATION_SYM_COUNT as u32) as u16,
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

    fn get_phase2_table(&self, corner_perm_sym_class_idx: u16, phase2_edge_perm_coord: u16) -> u8 {
        self.phase2_table
            [Self::encode_phase_2_coord(corner_perm_sym_class_idx, phase2_edge_perm_coord) as usize]
    }

    fn set_phase2_table(
        &mut self,
        corner_perm_sym_coord: u16,
        phase2_edge_perm_coord: u16,
        depth: u8,
    ) {
        self.phase2_table
            [Self::encode_phase_2_coord(corner_perm_sym_coord, phase2_edge_perm_coord) as usize] =
            depth;
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

        let mut distribution = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut max_depth = 0;

        let mut curr = 0;
        let mut old = 0;
        while !q.is_empty() {
            curr += 1;
            if curr / 1_000_000 != old {
                old += 1;
                raw_mode_print(&format!("{old}/138"));
            }
            let (curr_corner_orient, curr_flip_ud_class_idx) = q.pop_front().unwrap();
            let curr_depth = self.get_phase1_table(curr_corner_orient, curr_flip_ud_class_idx);
            max_depth = max(max_depth, curr_depth);
            for move_action in Move::ALL {
                let (next_flip_ud_class_idx, next_flip_ud_sym_idx) = move_table
                    .get_next_flip_ud_slice_sym_coord(curr_flip_ud_class_idx, 0, move_action as u8);

                let next: (u16, u16) = (
                    self.twist_conjugate_table.get_twist_conjugate(
                        move_table
                            .get_next_corner_orient_coord(curr_corner_orient, move_action as u8),
                        next_flip_ud_sym_idx,
                    ),
                    next_flip_ud_class_idx,
                );
                match self.get_phase1_table(next.0, next.1) == u8::MAX {
                    true => {
                        q.push_back(next);
                        distribution[curr_depth as usize + 1] += 1;
                        self.set_phase1_table(next.0, next.1, curr_depth + 1);

                        let sym_state = self.flip_ud_slice_table.get_sym_states(next.1);
                        if sym_state != 1 {
                            let mut sym = sym_state >> 1;
                            for sym_idx in 1..16 {
                                if sym & 1 == 1 {
                                    let alt_twist = self
                                        .twist_conjugate_table
                                        .get_twist_conjugate(next.0, sym_idx);
                                    if self.get_phase1_table(alt_twist, next.1) == u8::MAX {
                                        self.set_phase1_table(alt_twist, next.1, curr_depth + 1);
                                        distribution[curr_depth as usize + 1] += 1;
                                    }
                                }
                                sym >>= 1;
                            }
                        }
                    }
                    false => {}
                }
            }
        }
        // println!("Max depth: {max_depth}");
        // println!("Distribution: {distribution:?}");
    }

    fn generate_phase2_prune_table(&mut self, move_table: &MoveTable) {
        self.phase2_table.resize(
            CORNER_PERMUTATION_SYM_COUNT as usize * PHASE2_EDGE_PERMUTATION_COUNT as usize,
            u8::MAX,
        );

        self.set_phase2_table(0, 0, 0);
        // corner perm sym, phase2 edge perm
        let mut q: VecDeque<(u16, u16)> = VecDeque::new();
        q.push_back((0, 0));

        let mut distribution = [0; 20];
        let mut max_depth = 0;

        let mut curr = 0;
        let mut old = 0;
        while !q.is_empty() {
            curr += 1;
            if curr / 1_000_000 != old {
                old += 1;
                raw_mode_print(&format!("{old}/101"));
            }

            let (curr_corner_perm_sym, curr_phase2_edge_perm) = q.pop_front().unwrap();
            let curr_depth = self.get_phase2_table(curr_corner_perm_sym, curr_phase2_edge_perm);
            max_depth = max(max_depth, curr_depth);
            for move_action in Move::G1PRESERVING {
                let (next_corner_perm_sym_class_idx, next_corner_perm_sym_idx) = move_table
                    .get_next_corner_perm_sym_coord(curr_corner_perm_sym, 0, move_action as u8);

                let next: (u16, u16) = (
                    next_corner_perm_sym_class_idx,
                    self.edge8_pos_conjugate_table.get_edge8_pos_conjugate(
                        move_table.get_next_phase2_edge_perm_coord(
                            curr_phase2_edge_perm,
                            move_action as u8,
                        ),
                        next_corner_perm_sym_idx,
                    ),
                );
                match self.get_phase2_table(next.0, next.1) == u8::MAX {
                    true => {
                        q.push_back(next);
                        distribution[curr_depth as usize + 1] += 1;
                        self.set_phase2_table(next.0, next.1, curr_depth + 1);

                        let sym_state = self.corner_perm_sym_table.get_sym_states(next.0);
                        if sym_state != 1 {
                            let mut sym = sym_state >> 1;
                            for sym_idx in 1..16 {
                                if sym & 1 == 1 {
                                    let alt_edge_perm = self
                                        .edge8_pos_conjugate_table
                                        .get_edge8_pos_conjugate(next.1, sym_idx);
                                    if self.get_phase2_table(next.0, alt_edge_perm) == u8::MAX {
                                        self.set_phase2_table(
                                            next.0,
                                            alt_edge_perm,
                                            curr_depth + 1,
                                        );
                                        distribution[curr_depth as usize + 1] += 1;
                                    }
                                }
                                sym >>= 1;
                            }
                        }
                    }
                    false => {}
                }
            }
        }
        // println!("Max depth: {max_depth}");
        // println!("Distribution: {distribution:?}");
    }
}
