pub mod cubie;
pub mod moves;
pub mod display;
pub mod facelet;

use core::default::Default;
use moves::Move::*;

use crate::{cubie::Cubie, display::CubeDisplay};

fn main() {
    let mut cube: Cubie = Default::default();
    let moves = [U, D, L3, B2];
    cube = cube.apply_moves(&moves);
    let cube_display = CubeDisplay::new(['W', 'R', 'G', 'Y', 'O', 'B']);
    cube_display.display_cube(&facelet::Facelet::from_cubie(&cube));
}
