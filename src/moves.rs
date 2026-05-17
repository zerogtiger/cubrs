use core::assert;
use core::fmt;
use std::str::FromStr;

use crate::cubie::Corner::*;
use crate::cubie::CornerOrientation::*;
use crate::cubie::Edge::*;
use crate::cubie::EdgeOrientation::*;
use crate::cubie::*;
use Move::*;
use SymMove::*;

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

impl FromStr for Move {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "R" => Ok(R),
            "R'" => Ok(R3),
            "R3" => Ok(R3),
            "R2" => Ok(R2),
            "L" => Ok(L),
            "L'" => Ok(L3),
            "L3" => Ok(L3),
            "L2" => Ok(L2),
            "U" => Ok(U),
            "U'" => Ok(U3),
            "U3" => Ok(U3),
            "U2" => Ok(U2),
            "D" => Ok(D),
            "D'" => Ok(D3),
            "D3" => Ok(D3),
            "D2" => Ok(D2),
            "F" => Ok(F),
            "F'" => Ok(F3),
            "F3" => Ok(F3),
            "F2" => Ok(F2),
            "B" => Ok(B),
            "B'" => Ok(B3),
            "B3" => Ok(B3),
            "B2" => Ok(B2),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            U3 => write!(f, "U'"),
            D3 => write!(f, "D'"),
            R3 => write!(f, "R'"),
            L3 => write!(f, "L'"),
            F3 => write!(f, "F'"),
            B3 => write!(f, "B'"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Move {
    pub const ALL: [Move; 18] = [
        U, U2, U3, D, D2, D3, F, F2, F3, B, B2, B3, L, L2, L3, R, R2, R3,
    ];
    pub const ALL_UNIQUE: [Move; 6] = [U, D, F, B, L, R];
    pub const G1PRESERVING: [Move; 10] = [U, U2, U3, D, D2, D3, R2, L2, F2, B2];

    pub fn move_action_to_move_cubie(move_action: Move) -> Cubie {
        match move_action as Move {
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
            L2 => L_MOVE * L_MOVE,
            L3 => L_MOVE * L_MOVE * L_MOVE,
            R => R_MOVE,
            R2 => R_MOVE * R_MOVE,
            R3 => R_MOVE * R_MOVE * R_MOVE,
        }
    }

    pub fn inverse_move(&self) -> Move {
        match self {
            U => U3,
            U3 => U,
            D => D3,
            D3 => D,
            R => R3,
            R3 => R,
            L => L3,
            L3 => L,
            F => F3,
            F3 => F,
            B => B3,
            B3 => B,
            _ => *self,
        }
    }

    pub fn invert_moves(move_list: &mut Vec<Move>) {
        move_list.reverse();
        move_list.iter_mut().for_each(|m| *m = m.inverse_move());
    }

    pub fn move_cubie_to_move_action(move_cubie: &Cubie) -> Result<Move, ()> {
        for move_action in Self::ALL {
            if Self::move_action_to_move_cubie(move_action) == *move_cubie {
                return Ok(move_action);
            }
        }
        Err(())
    }

    pub fn is_same_class(move1: u8, move2: u8) -> bool {
        move1 / 3 == move2 / 3
    }

    pub fn move_list_from_str(moves: &str) -> Result<Vec<Move>, ()> {
        moves
            .split_whitespace()
            .map(|word| Move::from_str(word))
            .collect()
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymMove {
    S_F2 = 0,
    S_U4,
    S_LR2,
}

impl SymMove {
    pub const ALL: [SymMove; 3] = [S_F2, S_U4, S_LR2];

    pub fn sym_index_to_cubie_move(mut sym_idx: u8) -> Cubie {
        assert!(sym_idx < 16);
        let mut ret = Cubie::default();
        for _ in 0..sym_idx % 2 {
            ret = ret * S_F2_MOVE;
        }
        sym_idx /= 2;
        for _ in 0..sym_idx % 4 {
            ret = ret * S_U4_MOVE;
        }
        sym_idx /= 4;
        for _ in 0..sym_idx {
            ret = ret * S_LR2_MOVE;
        }
        ret
    }

    pub fn sym_index_to_inverse_cubie_move(mut sym_idx: u8) -> Cubie {
        assert!(sym_idx < 16);
        let mut ret = Cubie::default();
        let f2 = sym_idx % 2;
        sym_idx /= 2;
        let u4 = sym_idx % 4;
        let lr = sym_idx / 4;
        for _ in 0..lr {
            ret = ret * S_LR2_MOVE;
        }
        for _ in 0..(4 - u4) % 4 {
            ret = ret * S_U4_MOVE;
        }
        for _ in 0..f2 {
            ret = ret * S_F2_MOVE;
        }
        ret
    }

    pub fn sym_action_to_sym_index(sym_action: &Cubie) -> Result<u8, ()> {
        for sym_idx in 0..SYM_COUNT {
            if Self::sym_index_to_cubie_move(sym_idx) == *sym_action {
                return Ok(sym_idx);
            }
        }
        Err(())
    }
}

pub const S_F2_MOVE: Cubie = Cubie {
    corner_permutation: map_to_u8![DLF, DRF, DRB, DLB, ULF, URF, URB, ULB],
    edge_permutation: map_to_u8![DL, DF, DR, DB, UL, UF, UR, UB, FL, FR, BR, BL],
    corner_orientation: [0, 0, 0, 0, 0, 0, 0, 0],
    edge_orientation: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub const S_U4_MOVE: Cubie = Cubie {
    corner_permutation: map_to_u8![URB, URF, ULF, ULB, DRB, DRF, DLF, DLB],
    edge_permutation: map_to_u8![UB, UR, UF, UL, DB, DR, DF, DL, BR, FR, FL, BL],
    corner_orientation: [0, 0, 0, 0, 0, 0, 0, 0],
    edge_orientation: [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
};

pub const S_LR2_MOVE: Cubie = Cubie {
    corner_permutation: map_to_u8![ULF, URF, URB, ULB, DLF, DRF, DRB, DLB],
    edge_permutation: map_to_u8![UL, UF, UR, UB, DL, DF, DR, DB, FL, FR, BR, BL],
    corner_orientation: [3, 3, 3, 3, 3, 3, 3, 3],
    edge_orientation: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};
