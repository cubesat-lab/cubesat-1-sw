*** Variables ***
${SCRIPTS_FOLDER}    ./tools
${PYTHON}         python3    # Modify this if needed
${TARGET_FOLDER}    ./firmware/obc/cubesat-1-fw-obc/target
${QEMU_COMMAND}    qemu-system-arm -cpu cortex-m3 -machine stm32vldiscovery -serial pty -nographic -semihosting-config enable=on,target=native -kernel ${TARGET_FOLDER}/thumbv7m-none-eabi/debug/examples/serial_stm32vldiscovery
