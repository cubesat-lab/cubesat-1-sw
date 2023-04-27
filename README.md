# cubesat-1-sw

**CubeSat-1 Software** is a collection of software and tools used to pilot and operate the hardware of the [**CubeSat-1**](https://github.com/cubesat-lab) project.

The firmware for each CubeSat's subsystem is written in [Rust](https://www.rust-lang.org/) language for Arm® Cortex®-M7 32-bit RISC based [STM32F767ZI](https://www.st.com/en/microcontrollers-microprocessors/stm32f767zi.html) microcontroller from STMicroelectronics.

## Setup

The steps below are heavily inspired from [The Embedded Rust Book](https://docs.rust-embedded.org/book/intro/install.html)

### Basic

#### Prerequisites
none

#### Scope
Build and Flash the Firmware

#### Steps
- Install Rust
    - Linux (Ubuntu 22.04)

        *Run the command below and follow the on-screen instructions*
        ```bash
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        sudo apt install libudev-dev libssl-dev
        ```
    - Windows

        Download and Install Rust from [here](https://www.rust-lang.org/tools/install)
    - macOS

        [Update Homebrew](https://docs.brew.sh/FAQ)
        ```bash
        brew update
        brew upgrade
        ```
        ```bash
        brew install rust
        ```
- Add Cortex-M7 with hardware floating point (ARMv7E-M architecture):
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
- ST-LINK USB driver (for Windows only)

    Download and install it from [here](https://www.st.com/en/development-tools/stsw-link009.html)

### Complete

#### Prerequisites
Finish [Basic](#basic) setup

#### Scope
Design, Development, Testing, Debugging, etc.

#### Steps
- [Git](https://git-scm.com/downloads)
- IDE
    - [VSCode](https://code.visualstudio.com/) (Recommended IDE)
        - Extensions:
            - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
            - [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml)
            - [Structurizr](https://marketplace.visualstudio.com/items?itemName=ciarant.vscode-structurizr)
- [Python](https://www.python.org/downloads/)
    - Packages:
        ```bash
        pip install pyserial
        ```
- [Docker](https://www.docker.com/)
    - Setup [GitHub Actions](https://docs.github.com/en/actions) locally with [act](https://github.com/nektos/act)
    - [Structurizr Lite](https://structurizr.com/share/76352/documentation) for visualizing SW Architecture diagrams
- `gcc` and `gdb` for ARM
    - Linux (Ubuntu 22.04)

        gcc-arm-embedded
        ```bash
        ARM_TOOLCHAIN_VERSION=$(curl -s https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads | grep -Po '<h4>Version \K.+(?=</h4>)')
        curl -Lo gcc-arm-none-eabi.tar.xz "https://developer.arm.com/-/media/Files/downloads/gnu/${ARM_TOOLCHAIN_VERSION}/binrel/arm-gnu-toolchain-${ARM_TOOLCHAIN_VERSION}-x86_64-arm-none-eabi.tar.xz"
        sudo mkdir /opt/gcc-arm-none-eabi
        sudo tar xf gcc-arm-none-eabi.tar.xz --strip-components=1 -C /opt/gcc-arm-none-eabi
        echo 'export PATH=$PATH:/opt/gcc-arm-none-eabi/bin' | sudo tee -a /etc/profile.d/gcc-arm-none-eabi.sh
        source /etc/profile
        ```
    - Windows
        <!-- - MSYS2 Approach
            - Install [MSYS2](https://www.msys2.org/)
            - arm-none-eabi-gdb
                ```bash
                pacman -S gcc
                pacman -S mingw-w64-x86_64-gdb
                pacman -S mingw-w64-x86_64-arm-none-eabi-gdb
                ``` -->
        Download and install **arm-none-eabi-gdb** from [here](https://developer.arm.com/downloads/-/gnu-rm)
    - macOS
        ```bash
        brew install --cask gcc-arm-embedded
        brew install --cask arm-none-eabi-gdb
        ```
- OpenOCD
    - Linux (Ubuntu 22.04)
        ```bash
        sudo apt install openocd
        ```
    - Windows
        <!-- - MSYS2 Approach
            ```bash
            pacman -S mingw-w64-x86_64-openocd
            ``` -->
        Download and install **OpenOCD** from [here](https://xpack.github.io/dev-tools/openocd/install/)
    - macOS
        ```bash
        brew install open-ocd
        ```
<!-- - QEMU
    - TBD -->

## Hardware

- STM32F767ZI Board ([NUCLEO-F767ZI](https://www.st.com/en/evaluation-tools/nucleo-f767zi.html))

<!-- ## Usage
- TODO -->

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
