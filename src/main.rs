use crate::error::*;
use crate::machine::*;
use sdl2::{event::Event, pixels::Color};
use std::io::{stdout, Write};
use std::process::ExitCode;
use std::time::{Duration, Instant};

mod error;
mod ins;
mod machine;

const FREQUENCY: u64 = 500;

fn main() -> ExitCode {
    if let Err(e) = try_main() {
        stdout().flush().unwrap();
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn try_main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        return Err(Error::MissingArg);
    }

    let sdl_context = sdl2::init().map_err(Error::Sdl2Init)?;
    let video_subsystem = sdl_context.video().map_err(Error::Video)?;

    let window = video_subsystem
        .window(
            "r8",
            (DISPLAY_WIDTH * PIXEL_SIZE) as u32,
            (DISPLAY_HEIGHT * PIXEL_SIZE) as u32,
        )
        .position_centered()
        .opengl()
        .build()?;

    let mut m = Machine::new();

    let program = std::fs::read(&args[1]).unwrap();
    m.load(&program);

    let mut canvas = window.into_canvas().build()?;
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().map_err(Error::EventPump)?;

    let period = Duration::from_micros(1_000_000 / FREQUENCY);
    let mut period_start = Instant::now();
    loop {
        m.step(&mut event_pump, &mut canvas)?;
        if let Some(Event::Quit { .. }) = event_pump.poll_event() {
            return Ok(());
        }

        let elapsed = period_start.elapsed();
        if elapsed <= period {
            std::thread::sleep(period.saturating_sub(elapsed));
        }

        period_start = Instant::now();
    }
}
