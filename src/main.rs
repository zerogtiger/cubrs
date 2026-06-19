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


use cube_timer::{
    cubie::Cubie, display::CubeDisplay, moves::Move, solver::Solver
};

fn timer() {
    println!("== Cubrs ==");
    // let mut cube = Cubie::default();
    // cube = cube.apply_moves(&Move::move_list_from_str("B' L D U' B F' D' L' F' D L' D B' U'").unwrap());

    let solver = Solver::new();
    let cube_display = CubeDisplay::default_colors();
    // cube_display.display_cubie(&cube);
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

fn debug() {
    let solver = Solver::new();
    let cube_display = CubeDisplay::default_colors();
    let mut cube = Cubie::from_string("YBYGWBOOGWOBWRYYRRWWRGGRBORWGGRYYYRBOWGOOYOGORWGBBYWBB").unwrap();
    let  moves = solver.solve(&cube, 21).unwrap();
    cube_display.display_cubie(&cube);
    cube = cube.apply_moves(&moves);
    cube_display.display_cubie(&cube);
    println!("{:?}", moves);
}

fn main() {
    timer();
    // debug();
}
