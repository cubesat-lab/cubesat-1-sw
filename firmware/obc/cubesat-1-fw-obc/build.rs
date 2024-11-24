use std::{
    env,
    fmt::{Debug, Formatter, Result as FormatterResult},
    fs::File,
    io::{self, prelude::*},
    path::PathBuf,
};

const CMD_EXAMPLES: [&str; 3] = [
    "cargo build --target thumbv7em-none-eabihf --features nucleo-f446re-board --example serial_nucleo_f446re",
    "cargo build --target thumbv7em-none-eabihf --features nucleo-f767zi-board --example hello_cubesat",
    "cargo build --target thumbv7m-none-eabi --features stm32vldiscovery-board --example hello_stm32vldiscovery",
];

fn main() -> Result<(), Error> {
    // Retrieve the TARGET environment variable
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    let board = Board::read()?;

    assert_target(board.reference_target(), target)?;
    copy_memory_config(board)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

/// Make `memory.x` available to dependent crates
fn copy_memory_config(board: Board) -> Result<(), Error> {
    let memory_x = match board {
        Board::NucleoF446re => include_bytes!("memory_stm32f4xx.x").as_ref(),
        Board::NucleoF767zi => include_bytes!("memory_stm32f7xx.x").as_ref(),
        Board::Stm32vldiscovery => include_bytes!("memory_stm32vldiscovery.x").as_ref(),
    };

    let out_dir = env::var("OUT_DIR")?;
    let out_dir = PathBuf::from(out_dir);

    File::create(out_dir.join("memory.x"))?.write_all(memory_x)?;

    // Tell Cargo where to find the file.
    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rerun-if-changed=memory_stm32f4xx.x");
    println!("cargo:rerun-if-changed=memory_stm32f7xx.x");
    println!("cargo:rerun-if-changed=memory_stm32vldiscovery.x");

    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum Board {
    NucleoF446re,
    NucleoF767zi,
    Stm32vldiscovery,
}

impl Board {
    fn read() -> Result<Self, Error> {
        if cfg!(feature = "nucleo-f767zi-board") {
            Ok(Board::NucleoF767zi)
        } else if cfg!(feature = "nucleo-f446re-board") {
            Ok(Board::NucleoF446re)
        } else if cfg!(feature = "stm32vldiscovery-board") {
            Ok(Board::Stm32vldiscovery)
        } else {
            Err(Error::UnspecifiedBoard)
        }
    }

    fn reference_target(&self) -> String {
        match self {
            Board::NucleoF446re => "thumbv7em-none-eabihf".to_string(),
            Board::NucleoF767zi => "thumbv7em-none-eabihf".to_string(),
            Board::Stm32vldiscovery => "thumbv7m-none-eabi".to_string(),
        }
    }
}

fn assert_target(reference_target: String, actual_target: String) -> Result<(), Error> {
    if reference_target != actual_target {
        return Err(Error::WrongTarget);
    }
    Ok(())
}

enum Error {
    Env(env::VarError),
    Io(io::Error),
    WrongTarget,
    UnspecifiedBoard,
}

// Manually implement Debug to avoid dead code warnings for unused fields
impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatterResult {
        match self {
            Error::Env(e) => write!(f, "Env Error: {:?}", e),
            Error::Io(e) => write!(f, "IO Error: {:?}", e),
            Error::WrongTarget => panic_with_suggestion(
                "Wrong target",
                "You must select a correct target arch for the selected board (--target option)",
            ),
            Error::UnspecifiedBoard => panic_with_suggestion(
                "Unspecified board",
                "You must select a valid board (--features option)",
            ),
        }
    }
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
    panic!("\n\n\n\n{}\n\n\n", message);
}

fn panic_with_suggestion(error_message: &str, suggestion: &str) -> ! {
    let examples = CMD_EXAMPLES
        .iter()
        .map(|x| format!("    `{x}`\n"))
        .collect::<Box<str>>();
    let message = format!(
        "Error:      {}\nSuggestion: {}\nExamples:\n{}",
        error_message, suggestion, examples
    );

    error(message.as_str());
}
