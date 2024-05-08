/*
    Linker script that makes sure all of our compiled code goes into the right
    place, so the RP2040 knows how to deal with it.

    The comments below refer to the following documentation:

    - Raspberry Pi Pico Datasheet:
      https://datasheets.raspberrypi.com/pico/pico-datasheet.pdf
    - RP2040 Datasheet:
      https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf
    - ARMv6-M Architecture Reference Manual:
      https://developer.arm.com/documentation/ddi0419/e/?lang=en


    === WARNING ===

    This is just a very minimal linker script, specifically tailored to support
    the minimal Rust application in the same directory. It doesn't have
    everything that a full-blown linker script needs. It isn't even capable of
    running all Rust code (specifically, mutable `statics`s are not supported).

    If you're looking for a production-ready linker script for Embedded Rust
    code, check out `cortex-m-rt`:
    https://crates.io/crates/cortex-m-rt
*/

/*
    Specify the memory regions available on the Raspberry Pi Pico
*/
MEMORY {
    /*
        The portion at the start of the flash memory where the second-stage
        bootloader resides.

        The starting address of flash memory is documented in the RP2040
        Datasheet. The 256 byte size (0x100 in hexadecimal) is also documented
        there.
    */
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100

    /*
        The rest of the flash memory. This is where our compiled code goes.

        It starts right after the BOOT2 region. The total flash memory is 2MB,
        but we have to subtract the portion occupied by the second-stage
        bootloader.

        The RP2040 uses external flash memory, meaning this is a separate chip
        on the Pi Pico. Thereforce, we can get the 2M number from the Raspberry
        Pi Pico Datasheet.
    */
    FLASH : ORIGIN = 0x10000100, LENGTH = 2M - 0x100

    /*
        Location and size of main memory. These are available from the RP2040
        Datasheet.
    */
    RAM : ORIGIN = 0x20000000, LENGTH = 264K
}

/*
    Now that we have declared what memory we have available, we can start
    defining what specifically goes where.
*/
SECTIONS {
    /*
        First, the memory section where the bootloader goes. We call that memory
        section `.boot2`, declare that it is located at the beginning of the
        BOOT2 memory region, and instruct the linker to put the `.boot2` section
        that the Rust compiler generated in there.
    */
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2

    /*
        Next up, the vector table. This is a data structure that tells the
        microcontroller where to find code when it executes some. There's a
        section on that in the ARMv6-M Architecture Reference Manual, although
        we only handle a small subset of that here.
    */
    .vector_table ORIGIN(FLASH) :
    {
        /*
            The vector table starts with the initial stack pointer. The stack
            grows downwards (from higher to lower addresses), so we just
            initialize it with the highest RAM address.

            The `LONG` command just tells the linker to put a 32-bit number at
            the current position.
        */
        LONG(ORIGIN(RAM) + LENGTH(RAM));

        /*
            The only other thing we initialize is the reset vector. It's a
            pointer to the code that the hardware executes when it's started, so
            basically our main function.

            We define this `.vector_table.reset_vector` section in our Rust
            code, and we tell the linker to put it here.
        */
        KEEP(*(.vector_table.reset_vector));
    } > FLASH

    /*
        Next, we need to make sure that our compiled code itself ends up in the
        binary. The compiler puts those into sections starting with `.text`, and
        here we make sure these end up in the final binary, right after the
        vector table.

        I don't know why we don't need the `KEEP` command here, when we need it
        for all the other sections. But we don't, and the other sections won't
        work without it, and that's how it is for some reason.
    */
    .text ADDR(.vector_table) + SIZEOF(.vector_table) :
    {
        *(.text .text.*);
    } > FLASH

    /*
        Finally, we need to tell the linker to discard some extra information
        that would otherwise end up in the binary. Not sure why, but without
        this, I get error messages about overlapping sections.
    */
    /DISCARD/ :
    {
        *(.ARM.exidx.*);
    }
}

/*
    That's it, but please heed the warning above: This is not a production-ready
    linker script! It is just a very, very minimal example, for educational
    purposes. Don't expect it to work in any other situation.
*/
