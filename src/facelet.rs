use core::{iter::{zip, Iterator}};

use Color::*;

use crate::cubie::Cubie;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    U(u8),
    R(u8),
    F(u8),
    D(u8),
    L(u8),
    B(u8),
}

impl From<Color> for u8 {
    fn from(color: Color) -> Self {
        match color {
            U(i) => i,
            R(i) => 9 + i,
            F(i) => 9 * 2 + i,
            D(i) => 9 * 3 + i,
            L(i) => 9 * 4 + i,
            B(i) => 9 * 5 + i,
        }
    }
}

impl TryFrom<char> for Color {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'U' => Ok(U(0)),
            'R' => Ok(R(0)),
            'F' => Ok(F(0)),
            'D' => Ok(D(0)),
            'L' => Ok(L(0)),
            'B' => Ok(B(0)),
            _ => Err(()),
        }
    }
}

pub struct Facelet {
    // . . . |U0U1U2|
    // . . . |U3U4U5|
    // . . . |U6U7U8|
    //|L0L1L2|F0F1F2|R0R1R2|B0B1B2|
    //|L3L4L5|F3F4F5|R3R4R5|B3B4B5|
    //|L6L7L8|F6F7F8|R6R7R8|B6B7B8|
    // . . . |D0D1D2|
    // . . . |D3D4D5|
    // . . . |D6D7D8|
    //
    // U, R, F, D, L, B
    pub faces: [Color; 54],
}

impl Facelet {
    pub fn new() -> Self {
        return Self {
            faces: [
                U(0), U(1), U(2), U(3), U(4), U(5), U(6), U(7), U(8),
                R(0), R(1), R(2), R(3), R(4), R(5), R(6), R(7), R(8),
                F(0), F(1), F(2), F(3), F(4), F(5), F(6), F(7), F(8),
                D(0), D(1), D(2), D(3), D(4), D(5), D(6), D(7), D(8),
                L(0), L(1), L(2), L(3), L(4), L(5), L(6), L(7), L(8),
                B(0), B(1), B(2), B(3), B(4), B(5), B(6), B(7), B(8),
            ],
        };
    }

    pub fn from_cubie(cubie: &Cubie) -> Self {
        let mut ret = Self::new();
        for (original_idx, (corner, orientation)) in zip(cubie.corner_permutation, cubie.corner_orientation).enumerate() {
            for (face_idx, color) in CORNER_FACES[original_idx].iter().enumerate() {
                ret.faces[u8::from(*color) as usize] = CORNER_FACES[corner as usize][(face_idx + 3 - orientation as usize)%3];
            }
        }
        for (original_idx, (edge, orientation)) in zip(cubie.edge_permutation, cubie.edge_orientation).enumerate() {
            for (face_idx, color) in EDGE_FACES[original_idx].iter().enumerate() {
                ret.faces[u8::from(*color) as usize] = EDGE_FACES[edge as usize][(face_idx + orientation as usize)%2];
            }
        }
        ret
    }
    
    pub fn get_face(&self, color: &Color) -> Color {
        self.faces[u8::from(*color) as usize]
    }
}

pub const CORNER_FACES: [[Color; 3]; 8] = [
    /*URF=*/ [U(8), R(0), F(2)],
    /*ULF=*/ [U(6), F(0), L(2)],
    /*ULB=*/ [U(0), L(0), B(2)],
    /*URB=*/ [U(2), B(0), R(2)],
    /*DRF=*/ [D(2), F(8), R(6)],
    /*DLF=*/ [D(0), L(8), F(6)],
    /*DLB=*/ [D(6), B(8), L(6)],
    /*DRB=*/ [D(8), R(8), B(6)],
];

/// Map the edge positions to facelet positions.
pub const EDGE_FACES: [[Color; 2]; 12] = [
    /*UR=*/ [U(5), R(1)],
    /*UF=*/ [U(7), F(1)],
    /*UL=*/ [U(3), L(1)],
    /*UB=*/ [U(1), B(1)],
    /*DR=*/ [D(5), R(7)],
    /*DF=*/ [D(1), F(7)],
    /*DL=*/ [D(3), L(7)],
    /*DB=*/ [D(7), B(7)],
    /*FR=*/ [F(5), R(3)],
    /*FL=*/ [F(3), L(5)],
    /*BL=*/ [B(5), L(3)],
    /*BR=*/ [B(3), R(5)],
];
