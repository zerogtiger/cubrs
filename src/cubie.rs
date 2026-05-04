use core::iter::Iterator;
use std::default;
use std::ops::Mul;
use crate::moves::{Move::*, B_MOVE, D_MOVE, F_MOVE, L_MOVE, R_MOVE, U_MOVE};
use crate::moves::Move;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        edge_orientation: [EdgeOrientation; 12]
    ) -> Self {
        Cubie {
            corner_permutation: corner_permutation.map(|c| c as u8),
            corner_orientation: corner_orientation.map(|c| c as u8),
            edge_permutation: edge_permutation.map(|c| c as u8),
            edge_orientation: edge_orientation.map(|c| c as u8)
        }
    }

    pub fn apply_move(self, move_action: Move) -> Self {
        let decomposed = match move_action {
            U => U_MOVE,
            U2 => U_MOVE * U_MOVE,
            U3 => U_MOVE * U_MOVE * U_MOVE,
            D => D_MOVE,
            D2 => D_MOVE * D_MOVE,
            D3 => D_MOVE * D_MOVE * D_MOVE,
            F => F_MOVE,
            F2 => F_MOVE * F_MOVE,
            F3 => F_MOVE * F_MOVE * F_MOVE,
            B => B_MOVE,
            B2 => B_MOVE * B_MOVE,
            B3 => B_MOVE * B_MOVE * B_MOVE,
            L => L_MOVE,
            L2 =>  L_MOVE * L_MOVE,
            L3 =>  L_MOVE * L_MOVE * L_MOVE,
            R => R_MOVE,
            R2 => R_MOVE * R_MOVE,
            R3 => R_MOVE * R_MOVE * R_MOVE,
        };
        self * decomposed
    }

    pub fn apply_moves(&self, moves: &[Move]) -> Self {
        moves.iter().fold(*self, |acc, &m|acc.apply_move(m))
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
            ret.corner_orientation[i] = (self.corner_orientation
                [rhs.corner_permutation[i] as usize]
                + rhs.corner_orientation[i])
                % 3;
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

