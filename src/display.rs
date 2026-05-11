use crate::facelet::{Color, Facelet};

pub struct CubeDisplay {
    colors: [char; 6], // Color of U, R, F, D, L, B
}

impl CubeDisplay {
    pub fn new(colors: [char; 6]) -> Self {
        CubeDisplay { colors }
    }

    fn realize_color(&self, color: Color) -> char {
        self.colors[match color {
            Color::U(_) => 0,
            Color::R(_) => 1,
            Color::F(_) => 2,
            Color::D(_) => 3,
            Color::L(_) => 4,
            Color::B(_) => 5,
        }]
    }

    pub fn display_cube(&self, facelet: &Facelet) {
        for i in 0..3 {
            print!(". . . ");
            for j in 0..3 {
                print!(
                    "{} ",
                    self.realize_color(facelet.get_face(&Color::U(i * 3 + j)))
                );
            }
            println!();
        }
        for i in 0..3 {
            for j in 0..3 {
                print!(
                    "{} ",
                    self.realize_color(facelet.get_face(&Color::L(i * 3 + j)))
                );
            }
            for j in 0..3 {
                print!(
                    "{} ",
                    self.realize_color(facelet.get_face(&Color::F(i * 3 + j)))
                );
            }
            for j in 0..3 {
                print!(
                    "{} ",
                    self.realize_color(facelet.get_face(&Color::R(i * 3 + j)))
                );
            }
            for j in 0..3 {
                print!(
                    "{} ",
                    self.realize_color(facelet.get_face(&Color::B(i * 3 + j)))
                );
            }
            println!();
        }
        for i in 0..3 {
            print!(". . . ");
            for j in 0..3 {
                print!(
                    "{} ",
                    self.realize_color(facelet.get_face(&Color::D(i * 3 + j)))
                );
            }
            println!();
        }
    }
}
