use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unsupported opcode {0:#06X}")]
    Opcode(u16),
    #[error("Missing program argument. Usage: r8 program.ch8")]
    MissingArg,
    #[error("Could not initialize SDL2: {0}")]
    Sdl2Init(String),
    #[error("Could not initialize video: {0}")]
    Video(String),
    #[error("Could not initialize event pump: {0}")]
    EventPump(String),
    #[error(transparent)]
    WindowBuild(#[from] sdl2::video::WindowBuildError),
    #[error(transparent)]
    IntegerOrSdl(#[from] sdl2::IntegerOrSdlError),
    #[error("Could not fill rect: {0}")]
    FillRect(String),
}

pub type Result<T> = std::result::Result<T, Error>;
