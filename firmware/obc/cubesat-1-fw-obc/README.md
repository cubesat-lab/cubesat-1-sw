# cubesat-1-fw-obc

**CubeSat-1 Firmware for OBC** is an executable Rust project developed for OBC system of the [**CubeSat-1**](https://example.com/) project.

## Supported boards

- [NUCLEO-F767ZI](https://www.st.com/en/evaluation-tools/nucleo-f767zi.html) - main development board
- [NUCLEO-F446RE](https://www.st.com/en/evaluation-tools/nucleo-f446re.html) - alternative development board (cheaper, less pins, no sdmmc)
- [STM32VLDISCOVERY](https://www.st.com/en/evaluation-tools/stm32vldiscovery.html) - added for experimenting QEMU (only UART works so far)

## Getting started

While in development, this project will be subject to many changes, so this readme file might not reflect the latest state.

As a Rust project, this crate contains the main app, which resides in `src/main.rs` and some example mini-projects located in `examples` folder.

Because the development is done on multiple embedded targets simultaneosly and the compatibility is assured with a carefully crafted abstractions, it is important to be aware on which target you want to run the code.

Various targets/boards are defined in the `Cargo.toml` file as **features**. These are the possible board features:
- `nucleo-f446re-board`
- `nucleo-f767zi-board`
- `stm32vldiscovery-board`

So when building, flashing or using other cargo commands, it is important to pick the key parameters like `--target`, `--features`, `--example`, `--chip`, `--probe` and others when applicable. See examples below.

### Firmware compilation

First, you need to be able to run commands from the crate root directory:
```bash
cd ./firmware/obc/cubesat-1-fw-obc/
```

- Compile OBC Firmware
    ```bash
    # NUCLEO-F767ZI
    cargo build --features nucleo-f767zi-board

    # NUCLEO-F446RE
    cargo build --features nucleo-f446re-board

    # STM32VLDISCOVERY
    cargo build --target thumbv7m-none-eabi --features stm32vldiscovery-board
    ```

- Compile examples

    If you wish to compile an example project just add the `--example <example_filename>` parameter in the command line, like so:
    ```bash
    # Example file hello_cubesat.rs for NUCLEO-F767ZI
    cargo build --features nucleo-f767zi-board --example hello_cubesat
    ```
    Be careful, that specific examples are created for specific boards, consult the `Cargo.toml`, `[[example]]` sections.

### Board connection

- Connect the board to your PC via USB cable
    ```
    +----------+    USB    +----+-------------+
    |          |<--------->|    |             |
    |    PC    |           |    |    Board    |
    |          |           |    |             |
    +----------+           +----+-------------+
    ```

### Firmware flashing

When flashing the firmaware, make sure you have the compiled the firmware, connected the board/s and select the appropriate parameters.
If you have multiple boards connected to the PC, it's important to tell `cargo-flash` which specific board you want to flash.
For that, use the command:
```bash
probe-rs list
```
So whenever flashing a firmware, select the `--features`, `--target` (if needed), `--chip`, `--probe` (if you have multiple boards connected), `--connect-under-reset` (sometimes you might not be able to flash without this option) and `--example` (if you're using example).

- Flashing with `cargo flash`, without debugging
    ```bash
    # NUCLEO-F767ZI
    cargo flash --features nucleo-f767zi-board --chip STM32F767ZITx --probe 1234:5678:ABCDEF1234567890ABCD1234 --connect-under-reset

    # NUCLEO-F446RE
    cargo flash --features nucleo-f446re-board --chip STM32F446RETx --probe 1234:5678:ABCDEF1234567890ABCD1234 --connect-under-reset
    ```

- Flashing with vscode `launch.json`, with debugging - press key `F5`

### Miscellaneous

- OpenOCD (for manual connection with gdb)
    ```bash
    # In the first terminal
    openocd -f openocd_stm32f7xx.cfg

    # In the second terminal
    gdb-multiarch ./target/thumbv7em-none-eabihf/debug/cubesat-1-fw-obc -x openocd.gdb
    ```
    This example is for Linux (Ubuntu), debugging firmware on NUCLEO-F767ZI.

- QEMU (todo)
