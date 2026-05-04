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
