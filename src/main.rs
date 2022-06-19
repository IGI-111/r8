use crate::error::*;
use crate::machine::*;
use sdl2::{event::Event, pixels::Color};
use std::io::{stdout, Write};
use std::process::ExitCode;

mod error;
mod ins;
mod machine;

fn main() -> ExitCode {
    if let Err(e) = try_main() {
        stdout().flush().unwrap();
        eprintln!("{}", e);
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
    loop {
        m.step(&mut event_pump, &mut canvas)?;
        std::thread::sleep(std::time::Duration::from_micros(500)); // slow down a bit
        if let Some(Event::Quit { .. }) = event_pump.poll_event() {
            return Ok(());
        }
    }
}
