use std::{
    env,
    fs::File,
    io::{self, prelude::*},
    path::PathBuf,
};

fn main() -> Result<(), Error> {
    let target = Target::read();

    copy_memory_config(target)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

/// Make `memory.x` available to dependent crates
fn copy_memory_config(target: Target) -> Result<(), Error> {
    let memory_x = match target.board {
        Board::NucleoF767zi => include_bytes!("memory_stm32f7xx.x").as_ref(),
        Board::Stm32vldiscovery => include_bytes!("memory_stm32vldiscovery.x").as_ref(),
    };

    let out_dir = env::var("OUT_DIR")?;
    let out_dir = PathBuf::from(out_dir);

    File::create(out_dir.join("memory.x"))?.write_all(memory_x)?;

    // Tell Cargo where to find the file.
    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rerun-if-changed=memory_stm32f7xx.x");
    println!("cargo:rerun-if-changed=memory_stm32vldiscovery.x");

    Ok(())
}

#[derive(Clone, Copy)]
struct Target {
    board: Board,
}

impl Target {
    fn read() -> Self {
        let board = Board::read();
        Self { board }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Board {
    NucleoF767zi,
    Stm32vldiscovery,
}

impl Board {
    fn read() -> Self {
        if cfg!(feature = "nucleo-f767zi-board") {
            Board::NucleoF767zi
        } else if cfg!(feature = "stm32vldiscovery") {
            Board::Stm32vldiscovery
        } else {
            error("
ERROR: You must select a target board (--features option), see examples below:
    `cargo build --features nucleo-f767zi-board`
    `cargo build --target thumbv7m-none-eabi --features stm32vldiscovery --example hello_stm32vldiscovery`
");
        }
    }
}

#[derive(Debug)]
enum Error {
    Env(env::VarError),
    Io(io::Error),
}

impl From<env::VarError> for Error {
    fn from(error: env::VarError) -> Self {
        Self::Env(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

fn error(message: &str) -> ! {
    panic!("\n\n\n{}\n\n\n", message);
}
