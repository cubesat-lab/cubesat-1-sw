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
- Linux (Ubuntu 22.04)

    - Install Rust using command below and follow the on-screen instructions
        ```bash
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        ```
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
    - cargo-flash
        ```bash
        sudo apt install libudev-dev
        cargo install cargo-flash
        ```
    - gcc-arm-embedded
        ```bash
        ARM_TOOLCHAIN_VERSION=$(curl -s https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads | grep -Po '<h4>Version \K.+(?=</h4>)')
        curl -Lo gcc-arm-none-eabi.tar.xz "https://developer.arm.com/-/media/Files/downloads/gnu/${ARM_TOOLCHAIN_VERSION}/binrel/arm-gnu-toolchain-${ARM_TOOLCHAIN_VERSION}-x86_64-arm-none-eabi.tar.xz"
        sudo mkdir /opt/gcc-arm-none-eabi
        sudo tar xf gcc-arm-none-eabi.tar.xz --strip-components=1 -C /opt/gcc-arm-none-eabi
        echo 'export PATH=$PATH:/opt/gcc-arm-none-eabi/bin' | sudo tee -a /etc/profile.d/gcc-arm-none-eabi.sh
        source /etc/profile
        ```
    - OpenOCD
        ```bash
        sudo apt install openocd
        ```
    
- macOS
    - Install [VSCode](https://code.visualstudio.com/) (optionally)
    - Update Homebrew
        ```bash
        brew update
        brew upgrade
        ```
    - Install Rust
        ```bash
        brew install rust
        ```
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
    - cargo-flash
        ```bash
        cargo install cargo-flash
        ```
    - gcc-arm-embedded
        ```bash
        brew install --cask gcc-arm-embedded 
        ```
    - arm-none-eabi-gdb
        ```bash
        brew install --cask arm-none-eabi-gdb
        ```
    - OpenOCD
        ```bash
        brew install open-ocd
        ```

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
