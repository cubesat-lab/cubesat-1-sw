# cubesat-1-fw-obc

**CubeSat-1 Firmware for OBC** is an executable Rust project developed for OBC system of the [**CubeSat-1**](https://example.com/) project.

### Firmware Compiling

- Compile OBC Firmware

    ```bash
    cd ./firmware/obc/cubesat-1-fw-obc/
    cargo build
    ```

### Board Connection

- Connect the NUCLEO-F767ZI to your PC via USB cable
    ```
    +----------+           +----+-----------------+
    |          |    USB    |    |                 |
    |    PC    |<--------->|    |  NUCLEO-F767ZI  |
    |          |           |    |                 |
    +----------+           +----+-----------------+
    ```

### Firmware Flashing

- Flashing with `cargo flash`, without debugging
    ```bash
    cargo flash --chip STM32F767ZITx
    ```

- Flashing with vscode `launch.json`, with debugging - press key `F5`

### Miscellaneous

- OpenOCD (for manual connection with gdb)
    ```bash
    openocd -f openocd.cfg
    ```
