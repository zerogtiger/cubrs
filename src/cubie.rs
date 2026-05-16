use crate::moves::Move;
use crate::moves::{B_MOVE, D_MOVE, F_MOVE, L_MOVE, Move::*, R_MOVE, U_MOVE};
use core::iter::Iterator;
use core::{assert, todo};
use std::default;
use std::ops::Mul;

pub const FACTORIAL: [u32; 13] = [
    1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800, 479001600,
];

pub const CORNER_ORIENTATION_COUNT: u16 = 2187;
pub const EDGE_ORIENTATION_COUNT: u16 = 2048;
pub const CORNER_PERMUTATION_COUNT: u16 = 40320;
pub const EDGE_PERMUTATION_COUNT: u32 = 479001600;
pub const UD_SLICE_COUNT: u16 = 495;
pub const PHASE2_EDGE_PERMUTATION_COUNT: u16 = 40320;
pub const PHASE2_UD_SLICE_COUNT: u8 = 24;
pub const FLIP_UD_SLICE_COUNT: u16 = 64430;
pub const CORNER_PERMUTATION_SYM_COUNT: u16 = 2768;

pub const SYM_COUNT: u8 = 16;
pub const MOVE_COUNT: u8 = 18;

pub fn binomial_coefficient(n: i8, k: i8) -> u32 {
    assert!(n <= 12);
    assert!(k <= 12);
    if k < 0 {
        return 0;
    }
    match k > n {
        true => 0,
        false => FACTORIAL[n as usize] / (FACTORIAL[k as usize] * FACTORIAL[(n - k) as usize]),
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Corner {
    URF = 0,
    ULF,
    ULB,
    URB,
    DRF,
    DLF,
    DLB,
    DRB,
}

impl TryFrom<u8> for Corner {
    type Error = ();
    fn try_from(x: u8) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(Corner::URF),
            1 => Ok(Corner::ULF),
            2 => Ok(Corner::ULB),
            3 => Ok(Corner::URB),
            4 => Ok(Corner::DRF),
            5 => Ok(Corner::DLF),
            6 => Ok(Corner::DLB),
            7 => Ok(Corner::DRB),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Edge {
    UR = 0,
    UF,
    UL,
    UB,
    DR,
    DF,
    DL,
    DB,
    FR,
    FL,
    BL,
    BR,
}

impl TryFrom<u8> for Edge {
    type Error = ();
    fn try_from(x: u8) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(Edge::UR),
            1 => Ok(Edge::UF),
            2 => Ok(Edge::UL),
            3 => Ok(Edge::UB),
            4 => Ok(Edge::DR),
            5 => Ok(Edge::DF),
            6 => Ok(Edge::DL),
            7 => Ok(Edge::DB),
            8 => Ok(Edge::FR),
            9 => Ok(Edge::FL),
            10 => Ok(Edge::BL),
            11 => Ok(Edge::BR),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CornerOrientation {
    No = 0,
    CW = 1,
    CCW = 2,
}

impl TryFrom<u8> for CornerOrientation {
    type Error = ();
    fn try_from(x: u8) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(CornerOrientation::No),
            1 => Ok(CornerOrientation::CW),
            2 => Ok(CornerOrientation::CCW),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeOrientation {
    Normal = 0,
    Flipped = 1,
}

impl TryFrom<u8> for EdgeOrientation {
    type Error = ();
    fn try_from(x: u8) -> Result<Self, Self::Error> {
        match x {
            0 => Ok(EdgeOrientation::Normal),
            1 => Ok(EdgeOrientation::Flipped),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cubie {
    pub corner_permutation: [u8; 8],
    pub edge_permutation: [u8; 12],
    pub corner_orientation: [u8; 8],
    pub edge_orientation: [u8; 12],
}

impl Cubie {
    pub fn new(
        corner_permutation: [Corner; 8],
        corner_orientation: [CornerOrientation; 8],
        edge_permutation: [Edge; 12],
        edge_orientation: [EdgeOrientation; 12],
    ) -> Self {
        Cubie {
            corner_permutation: corner_permutation.map(|c| c as u8),
            corner_orientation: corner_orientation.map(|c| c as u8),
            edge_permutation: edge_permutation.map(|c| c as u8),
            edge_orientation: edge_orientation.map(|c| c as u8),
        }
    }

    pub fn apply_move(self, move_action: Move) -> Self {
        self * Move::move_action_to_move_cubie(move_action)
    }

    pub fn apply_moves(&self, moves: &[Move]) -> Self {
        moves.iter().fold(*self, |acc, &m| acc.apply_move(m))
    }

    pub fn corner_perm_parity(&self) -> u8 {
        let mut ret = 0;
        for i in (0..8).rev() {
            for j in (0..i).rev() {
                if self.corner_permutation[j] > self.corner_permutation[i] {
                    ret += 1;
                }
            }
        }
        ret % 2
    }

    pub fn edge_perm_parity(&self) -> u8 {
        let mut ret = 0;
        for i in (0..12).rev() {
            for j in (0..i).rev() {
                if self.edge_permutation[j] > self.edge_permutation[i] {
                    ret += 1;
                }
            }
        }
        ret % 2
    }

    pub fn corner_orient_parity(&self) -> u8 {
        let mut ret = 0;
        for i in 0..8 {
            ret += self.corner_orientation[i];
        }
        ret % 3
    }

    pub fn edge_orient_parity(&self) -> u8 {
        let mut ret = 0;
        for i in 0..12 {
            ret += self.edge_orientation[i];
        }
        ret % 2
    }

    pub fn is_solvable(&self) -> bool {
        self.edge_perm_parity() == self.corner_perm_parity()
            && self.corner_orient_parity() == 0
            && self.edge_orient_parity() == 0
    }

    pub fn corner_orientation_coord(&self) -> u16 {
        let mut ret: u16 = 0;
        if let Some((last, rest)) = self.corner_orientation.split_last() {
            for &orientation in rest {
                ret = ret * 3 + (orientation % 3) as u16;
            }
        }
        ret
    }

    pub fn edge_orientation_coord(&self) -> u16 {
        let mut ret: u16 = 0;
        if let Some((last, rest)) = self.edge_orientation.split_last() {
            for &orientation in rest {
                ret = ret * 2 + orientation as u16;
            }
        }
        ret
    }

    pub fn corner_permutation_coord(&self) -> u16 {
        let mut ret: u16 = 0;
        for idx in 1..8 {
            let mut accum = 0;
            for curr in 0..idx {
                accum += match self.corner_permutation[curr] > self.corner_permutation[idx] as u8 {
                    true => 1,
                    false => 0,
                };
            }
            ret += accum * FACTORIAL[idx] as u16;
        }
        ret
    }

    pub fn edge_permutation_coord(&self) -> u32 {
        let mut ret: u32 = 0;
        let mut factorial: u32 = 1;
        for idx in 1..12 {
            factorial *= idx as u32;
            let mut accum = 0;
            for curr in 0..idx {
                accum += match self.edge_permutation[curr] > self.edge_permutation[idx] as u8 {
                    true => 1,
                    false => 0,
                };
            }
            ret += accum * factorial;
        }
        ret
    }

    pub fn ud_slice_coord(&self) -> u16 {
        let mut occupied: [u16; 12] = [0; 12];
        for (edge_idx, &edge) in self.edge_permutation.iter().enumerate() {
            occupied[edge_idx] = match edge >= 8 {
                true => 1,
                false => 0,
            }
        }
        let mut ret: u16 = 0;
        let mut cnt: i16 = -1;
        for (idx, if_occupied) in occupied.iter().enumerate() {
            ret += match if_occupied {
                0 => {
                    match cnt {
                        -1 => 0,
                        _ => {
                            // max value is 11C3 = 462 << u16MAX
                            binomial_coefficient(idx as i8, cnt as i8) as u16
                        }
                    }
                }
                1 => {
                    cnt += 1;
                    0 as u16
                }
                _ => 0,
            }
        }
        ret
    }

    pub fn phase2_edge_permutation_coord(&self) -> u16 {
        // TODO: asserts cube is in G1
        let mut ret: u16 = 0;
        let mut factorial: u32 = 1;
        for idx in 1..8 {
            factorial *= idx as u32;
            let mut accum = 0;
            for curr in 0..idx {
                accum += match self.edge_permutation[curr] > self.edge_permutation[idx] as u8 {
                    true => 1,
                    false => 0,
                };
            }
            ret += accum * factorial as u16;
        }
        ret
    }

    pub fn phase2_ud_slice_coord(&self) -> u8 {
        // TODO: asserts cube is in G1
        let mut ret: u8 = 0;
        let mut factorial: u16 = 1;
        for idx in 1..4 {
            factorial *= idx as u16;
            let mut accum = 0;
            for curr in 8..(8 + idx) {
                accum += match self.edge_permutation[curr] > self.edge_permutation[8 + idx] as u8 {
                    true => 1,
                    false => 0,
                };
            }
            ret += accum * factorial as u8;
        }
        ret
    }

    pub fn set_corner_orientation_coord(&mut self, mut corner_orient_coord: u16) {
        let mut accum = 0;
        for idx in (0..7).rev() {
            self.corner_orientation[idx] = (corner_orient_coord % 3) as u8;
            corner_orient_coord /= 3;
            accum = (accum + self.corner_orientation[idx]) % 3;
        }
        self.corner_orientation[7] = (3 - accum) % 3;
    }

    pub fn set_edge_orientation_coord(&mut self, mut edge_orient_coord: u16) {
        let mut accum = 0;
        for idx in (0..11).rev() {
            self.edge_orientation[idx] = (edge_orient_coord % 2) as u8;
            edge_orient_coord /= 2;
            accum = (accum + self.edge_orientation[idx]) % 2;
        }
        self.edge_orientation[11] = accum;
    }

    pub fn set_corner_permutation_coord(&mut self, mut corner_perm_coord: u16) {
        let mut items = vec![0, 1, 2, 3, 4, 5, 6, 7];
        for idx in (0..8).rev() {
            self.corner_permutation[idx] =
                items.remove(idx - (corner_perm_coord / FACTORIAL[idx] as u16) as usize);
            corner_perm_coord %= FACTORIAL[idx] as u16;
        }
    }

    pub fn set_edge_permutation_coord(&mut self, mut edge_perm_coord: u32) {
        let mut items = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        for idx in (0..12).rev() {
            self.edge_permutation[idx] =
                items.remove(idx - (edge_perm_coord / FACTORIAL[idx]) as usize);
            edge_perm_coord %= FACTORIAL[idx];
        }
    }

    pub fn set_ud_slice_coord(&mut self, mut ud_slice_coord: u16) {
        let mut items = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let mut k = 3;
        for n in (0..12).rev() {
            match ud_slice_coord >= (binomial_coefficient(n, k) as u16) {
                true => {
                    ud_slice_coord -= binomial_coefficient(n, k) as u16;
                    self.edge_permutation[n as usize] = items.remove(0);
                }
                false => {
                    k -= 1;
                    self.edge_permutation[n as usize] = items.remove(items.len() - 1);
                }
            }
        }
    }

    pub fn set_phase2_edge_permutation_coord(&mut self, mut phase2_edge_perm_coord: u16) {
        // TODO: assert cube in G1
        let mut items = vec![0, 1, 2, 3, 4, 5, 6, 7];
        for idx in (0..8).rev() {
            self.edge_permutation[idx] =
                items.remove(idx - (phase2_edge_perm_coord / FACTORIAL[idx] as u16) as usize);
            phase2_edge_perm_coord %= FACTORIAL[idx] as u16;
        }
    }

    pub fn set_phase2_ud_slice_coord(&mut self, mut phase2_ud_slice_coord: u8) {
        // TODO: assert cube in G1
        let mut items = vec![8, 9, 10, 11];
        for idx in (0..4).rev() {
            self.edge_permutation[8 + idx] =
                items.remove(idx - (phase2_ud_slice_coord as u32 / FACTORIAL[idx]) as usize);
            phase2_ud_slice_coord %= FACTORIAL[idx] as u8;
        }
    }
}

impl Default for Cubie {
    fn default() -> Self {
        Cubie {
            corner_permutation: [0, 1, 2, 3, 4, 5, 6, 7],
            edge_permutation: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            corner_orientation: [0, 0, 0, 0, 0, 0, 0, 0],
            edge_orientation: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}

impl Mul for Cubie {
    type Output = Cubie;

    fn mul(self, rhs: Self) -> Self::Output {
        // (A*B)(x).c=A(B(x).c).c
        // (A*B)(x).o=A(B(x).c).o+B(x).o
        let mut ret: Cubie = Default::default();
        for i in 0..8 {
            ret.corner_permutation[i] = self.corner_permutation[rhs.corner_permutation[i] as usize];

            let ori_a = self.corner_orientation[rhs.corner_permutation[i] as usize];
            let ori_b = rhs.corner_orientation[i];
            ret.corner_orientation[i] = if ori_a < 3 && ori_b < 3 {
                match ori_a + ori_b >= 3 {
                    true => ori_a + ori_b - 3,
                    false => ori_a + ori_b,
                }
            } else if ori_a < 3 && ori_b >= 3 {
                match ori_a + ori_b >= 6 {
                    true => ori_a + ori_b - 3,
                    false => ori_a + ori_b,
                }
            } else if ori_a >= 3 && ori_b < 3 {
                match ori_a - ori_b < 3 {
                    true => 3 + ori_a - ori_b,
                    false => ori_a - ori_b,
                }
            } else {
                match (ori_a as i8 - ori_b as i8) < 0 {
                    true => 3 + ori_a - ori_b,
                    false => ori_a - ori_b,
                }
            }
        }
        for i in 0..12 {
            ret.edge_permutation[i] = self.edge_permutation[rhs.edge_permutation[i] as usize];
            ret.edge_orientation[i] = (self.edge_orientation[rhs.edge_permutation[i] as usize]
                + rhs.edge_orientation[i])
                % 2;
        }
        ret
    }
}
