pub mod cubie;
pub mod display;
pub mod facelet;
pub mod moves;
pub mod movetable;
pub mod solver;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::{
    io::{Write, stdout},
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

use crate::{
    display::CubeDisplay,
    solver::Solver,
};

fn main() {
    println!("== Cubrs ==");
    let solver = Solver::new();
    let cube_display = CubeDisplay::default_colors();
    loop {
        let (cube, to_cube) = solver.generate_scramble(21);
        for move_action in to_cube {
            print!("{move_action} ");
        }
        println!("");
        cube_display.display_cubie(&cube);

        enable_raw_mode().unwrap();
        let mut start = false;
        while !start {
            execute!(
                stdout(),
                cursor::MoveToColumn(0),
                Clear(ClearType::CurrentLine)
            )
            .unwrap();
            print!("\"q\" to quit; spacebar to ready");
            stdout().flush().unwrap();
            if let Event::Key(event) = event::read().unwrap() {
                if event.code == KeyCode::Char('q') {
                    disable_raw_mode().unwrap();
                    return;
                }

                if event.code == KeyCode::Char(' ') && event.kind == KeyEventKind::Press {
                    execute!(
                        stdout(),
                        cursor::MoveToColumn(0),
                        Clear(ClearType::CurrentLine)
                    )
                    .unwrap();
                    print!("Ready: \"q\" to cancel; spacebar to start");
                    stdout().flush().unwrap();
                    loop {
                        if let Event::Key(event) = event::read().unwrap() {
                            if event.code == KeyCode::Char('q') {
                                break;
                            }

                            if event.code == KeyCode::Char(' ') && event.kind == KeyEventKind::Press
                            {
                                execute!(
                                    stdout(),
                                    cursor::MoveToColumn(0),
                                    Clear(ClearType::CurrentLine)
                                )
                                .unwrap();
                                print!("Solve");
                                stdout().flush().unwrap();
                                start = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        let start = Instant::now();

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let timer_thread = std::thread::spawn(move || {
            while running_clone.load(std::sync::atomic::Ordering::Relaxed) {
                let elapsed = start.elapsed().as_secs_f64();
                execute!(
                    stdout(),
                    cursor::MoveToColumn(0),
                    Clear(ClearType::CurrentLine)
                )
                .unwrap();
                print!("{:.4}s", elapsed);
                stdout().flush().unwrap();
                std::thread::sleep(Duration::from_millis(10));
            }
        });
        loop {
            if let Event::Key(e) = event::read().unwrap() {
                if e.kind == KeyEventKind::Press {
                    let elapsed = start.elapsed().as_secs_f64();
                    execute!(
                        stdout(),
                        cursor::MoveToColumn(0),
                        Clear(ClearType::CurrentLine)
                    )
                    .unwrap();
                    running.store(false, std::sync::atomic::Ordering::Relaxed);
                    let _ = timer_thread.join();
                    println!("{:.4}s\n", elapsed);
                    break;
                }
            }
        }
        execute!(
            stdout(),
            cursor::MoveToColumn(0),
            Clear(ClearType::CurrentLine)
        )
        .unwrap();
        disable_raw_mode().unwrap();
    }
}
