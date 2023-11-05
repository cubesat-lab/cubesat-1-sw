/* Linker script content taken from: https://github.com/japaric-archived/vl/blob/master/memory.x */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 128K
    RAM : ORIGIN = 0x20000000, LENGTH = 8K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
