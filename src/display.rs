use crate::{
    cubie::Cubie,
    facelet::{Color, Facelet},
};

pub struct CubeDisplay {
    colors: [&'static str; 6], // Color of U, R, F, D, L, B
    color_size: u8,
}

impl CubeDisplay {
    pub fn new(colors: [&'static str; 6], color_size: u8) -> Self {
        CubeDisplay { colors, color_size }
    }

    pub fn default_colors() -> Self {
        CubeDisplay {
            colors: [
                "\x1b[48;5;15m\x1b[38;5;0mW \x1b[0m",
                "\x1b[48;5;9m\x1b[38;5;0mR \x1b[0m",
                "\x1b[48;5;10m\x1b[38;5;0mG \x1b[0m",
                "\x1b[48;5;11m\x1b[38;5;0mY \x1b[0m",
                "\x1b[48;5;208m\x1b[38;5;0mO \x1b[0m",
                "\x1b[48;5;32m\x1b[38;5;0mB \x1b[0m",
            ],
            color_size: 2,
        }
    }

    fn realize_color(&self, color: Color) -> &str {
        &self.colors[match color {
            Color::U(_) => 0,
            Color::R(_) => 1,
            Color::F(_) => 2,
            Color::D(_) => 3,
            Color::L(_) => 4,
            Color::B(_) => 5,
        }]
    }

    pub fn display_cubie(&self, cubie: &Cubie) {
        self.display_facelet(&Facelet::from_cubie(&cubie));
    }

    pub fn display_facelet(&self, facelet: &Facelet) {
        let filler = " ".repeat((self.color_size * 3 + 1) as usize);
        for i in 0..3 {
            print!("{}", filler);
            for j in 0..3 {
                print!(
                    "{}",
                    self.realize_color(facelet.get_face(&Color::U(i * 3 + j)))
                );
            }
            println!();
        }
        for i in 0..3 {
            for j in 0..3 {
                print!(
                    "{}",
                    self.realize_color(facelet.get_face(&Color::L(i * 3 + j)))
                );
            }
            print!(" ");
            for j in 0..3 {
                print!(
                    "{}",
                    self.realize_color(facelet.get_face(&Color::F(i * 3 + j)))
                );
            }
            print!(" ");
            for j in 0..3 {
                print!(
                    "{}",
                    self.realize_color(facelet.get_face(&Color::R(i * 3 + j)))
                );
            }
            print!(" ");
            for j in 0..3 {
                print!(
                    "{}",
                    self.realize_color(facelet.get_face(&Color::B(i * 3 + j)))
                );
            }
            println!();
        }
        for i in 0..3 {
            print!("{}", filler);
            for j in 0..3 {
                print!(
                    "{}",
                    self.realize_color(facelet.get_face(&Color::D(i * 3 + j)))
                );
            }
            println!();
        }
    }
}
