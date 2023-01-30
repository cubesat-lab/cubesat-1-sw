# cubesat-1-sw

**CubeSat-1 Software** is a collection of software and tools used to pilot and operate the hardware of the [**CubeSat-1**](https://example.com/) project.

The firmware for each CubeSat's subsystem is written in [Rust](https://www.rust-lang.org/) language for Arm® Cortex®-M7 32-bit RISC based [STM32F767ZI](https://www.st.com/en/microcontrollers-microprocessors/stm32f767zi.html) microcontroller from STMicroelectronics.

## Setup

The steps below are heavily inspired from [The Embedded Rust Book](https://docs.rust-embedded.org/book/intro/install.html)


- Windows
    - Install [VSCode](https://code.visualstudio.com/) (optionally)
    - Install [Rust](https://www.rust-lang.org/learn/get-started)
    - Install [MSYS2](https://www.msys2.org/)
    - Cortex-M7F with hardware floating point (ARMv7E-M architecture):
        ```bash
        rustup target add thumbv7em-none-eabihf
        ```
    - cargo-binutils
        ```bash
        cargo install cargo-binutils
        rustup component add llvm-tools-preview
        ```
    - cargo-generate
        ```bash
        cargo install cargo-generate
        ```
    - arm-none-eabi-gdb
        ```bash
        pacman -S gcc
        pacman -S mingw-w64-x86_64-gdb
        pacman -S mingw-w64-x86_64-arm-none-eabi-gdb
        ```
        or download [here](https://developer.arm.com/downloads/-/gnu-rm) and install it
    - OpenOCD
        ```bash
        pacman -S mingw-w64-x86_64-openocd
        ```
- Linux

    - todo

- macOS

    - todo

### Hardware

- STM32F767ZI Board ([NUCLEO-F767ZI](https://www.st.com/en/evaluation-tools/nucleo-f767zi.html))

## Usage

- todo

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
