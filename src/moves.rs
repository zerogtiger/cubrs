use crate::cubie::Corner::*;
use crate::cubie::CornerOrientation::*;
use crate::cubie::Edge::*;
use crate::cubie::EdgeOrientation::*;
use crate::cubie::*;
use Move::*;

macro_rules! map_to_u8 {
    ($($c:ident),*) => {
        [$($c as u8), *]
    };
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    U = 0,
    U2,
    U3,
    D,
    D2,
    D3,
    F,
    F2,
    F3,
    B,
    B2,
    B3,
    L,
    L2,
    L3,
    R,
    R2,
    R3,
}

impl Move {
    pub const ALL: [Move; 18] = [
        U, U2, U3, D, D2, D3, F, F2, F3, B, B2, B3, L, L2, L3, R, R2, R3
    ];
    pub const ALL_UNIQUE: [Move; 6] = [
        U, D, F, B, L, R
    ];
}

pub const U_MOVE: Cubie = Cubie {
                                // URF, ULF, ULB, URB, DRF, DLF, DLB, DRB,
    corner_permutation: map_to_u8![URB, URF, ULF, ULB, DRF, DLF, DLB, DRB],
    corner_orientation: [0, 0, 0, 0, 0, 0, 0, 0],
                              // UR, UF, UL, UB, DR, DF, DL, DB, FR, FL, BL, BR,
    edge_permutation: map_to_u8![UB, UR, UF, UL, DR, DF, DL, DB, FR, FL, BL, BR],
    edge_orientation: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub const D_MOVE: Cubie = Cubie {
    corner_permutation: map_to_u8![URF, ULF, ULB, URB, DLF, DLB, DRB, DRF],
    corner_orientation: [0, 0, 0, 0, 0, 0, 0, 0],
    edge_permutation: map_to_u8![UR, UF, UL, UB, DF, DL, DB, DR, FR, FL, BL, BR],
    edge_orientation: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub const R_MOVE: Cubie = Cubie {
    corner_permutation: map_to_u8![DRF, ULF, ULB, URF, DRB, DLF, DLB, URB],
    corner_orientation: map_to_u8![CCW, No, No, CW, CW, No, No, CCW],
    edge_permutation: map_to_u8![FR, UF, UL, UB, BR, DF, DL, DB, DR, FL, BL, UR],
    edge_orientation: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub const L_MOVE: Cubie = Cubie {
    corner_permutation: map_to_u8![URF, ULB, DLB, URB, DRF, ULF, DLF, DRB],
    corner_orientation: map_to_u8![No, CW, CCW, No, No, CCW, CW, No],
    edge_permutation: map_to_u8![UR, UF, BL, UB, DR, DF, FL, DB, FR, UL, DL, BR],
    edge_orientation: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub const F_MOVE: Cubie = Cubie {
    corner_permutation: map_to_u8![ULF, DLF, ULB, URB, URF, DRF, DLB, DRB],
    corner_orientation: map_to_u8![CW, CCW, No, No, CCW, CW, No, No],
    edge_permutation: map_to_u8![UR, FL, UL, UB, DR, FR, DL, DB, UF, DF, BL, BR],
    edge_orientation: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0],
};

pub const B_MOVE: Cubie = Cubie {
    corner_permutation: map_to_u8![URF, ULF, URB, DRB, DRF, DLF, ULB, DLB],
    corner_orientation: map_to_u8![No, No, CW, CCW, No, No, CCW, CW],
    edge_permutation: map_to_u8![UR, UF, UL, BR, DR, DF, DL, BL, FR, FL, UB, DB],
    edge_orientation: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
};

