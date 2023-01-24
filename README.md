# cubesat-1-sw

**CubeSat-1 Software** is a collection of software and tools used to pilot and operate the hardware of the [**CubeSat-1**](https://example.com/) project.

The firmware for each CubeSat's subsystem is written in [Rust](https://www.rust-lang.org/) language for Arm® Cortex®-M7 32-bit RISC based [STM32F767ZI](https://www.st.com/en/microcontrollers-microprocessors/stm32f767zi.html) microcontroller from STMicroelectronics.

## Setup

### Prerequisites

- Installed [Rust](https://www.rust-lang.org/learn/get-started)
- STM32F767ZI Board ([NUCLEO-F767ZI](https://www.st.com/en/evaluation-tools/nucleo-f767zi.html))
- VSCode (optionally)

### Compiling

- Compile OBC Firmware

    ```bash
    cd ./firmware/obc/cubesat-1-fw-obc/
    cargo build
    ```

### Board Connection

- todo

### Firmware Flashing

- todo

## Usage

- todo

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
