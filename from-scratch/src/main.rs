// Rust's standard library expects to run on top of an operating system, and we
// don't have that here. Let's tell the compiler that we're not using the
// standard library, which still leaves us with the core library.
#![no_std]
// Similarly, the usual Rust runtime expects there to operating system
// infrastructure that calls our `main` function. We'll be making our own
// arrangement, so here we let the Rust compiler know not to worry about it.
#![no_main]

// The RP2040 has a second stage bootloader, to initialize the external flash
// memory. We have a pre-compiled version of that in this repository, and we
// just need to make sure to include it in the binary, at the right place.
//
// The `.boot2` section we put this in gets picked up by the linker script,
// which takes care of the rest. The `#[used]` makes sure the compiler doesn't
// just throw this away as unused when compiling in release mode.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = *include_bytes!("../boot2_w25q080.padded.bin");

// After the second stage boot loader did its thing, the hardware will read the
// vector table to figure out where to continue. The following `static`, in
// combination with the linker script, makes sure that our `main` function gets
// called.
//
// Again, the `#[used]` makes sure the compiler doesn't just throw this away as
// unused when compiling in release mode.
#[link_section = ".vector_table.reset_vector"]
#[used]
pub static RESET_VECTOR: extern "C" fn() -> ! = main;

// And here's our `main` function, though it's not the usual.
//
// First, we have to tell the compiler to use C calling convention, to prevent
// it from doing any Rust-specific magic that the hardware can't deal it.
//
// Second, a normal `main` function returns at some point, but what would happen
// if we did that here? We don't have an operating system to go back to, and the
// `!` return value lets the compiler know that this is a function that never
// returns.
extern "C" fn main() -> ! {
    // This is the simplest Embedded Rust program I could think of: It turns on
    // an LED, and then just loops for the rest of time.

    clear_io_bank_0_reset();
    select_sio_for_pin25();
    set_pin25_to_output();

    loop {}
}

fn clear_io_bank_0_reset() {
    // I/O Bank 0 (also called GPIO, for "general-purpose I/O) is what lets us
    // control the RP2040's pins, which is what we need to turn on the LED. When
    // the microcontroller starts, GPIO is not enabled yet ("held in reset", we
    // could call it). Enabling it (clearing the reset) is what this function
    // does.

    // Like many microcontrollers, the RP2040 is using a technique called
    // memory-mapped I/O (MMIO). What this means is, we're writing to and
    // reading from what looks like normal memory addresses. But those don't
    // actually point to memory. Instead, it's special hardware listening on the
    // other end, and we used those memory reads and writes to communicate with
    // it.
    //
    // Such a fake memory location is called a "register", and for now we need
    // two of those, both belonging to the reset controller (`RESETS`). The
    // first one (`RESET`) controls the reset state of various hardware
    // peripherals. The second one (`RESET_DONE`) lets us know which hardware
    // peripherals are currently held in reset.
    //
    // We're going to use both of those further down.
    const RESETS_RESET: *mut u32 = 0x4000_c000 as *mut _;
    const RESETS_RESET_DONE: *mut u32 = 0x4000_c008 as *mut _;

    // Both of those registers work similarly, in that there's a bit for each
    // hardware peripheral. GPIO is represented by bit 5 in both registers. Here
    // we create a mask that we can use for some bit fiddling.
    const RESETS_IO_BANK0: u32 = 0x1 << 5;

    // To clear the reset, we need to clear the respective bit in the `RESET`
    // register. We do that here, by reading the register, clearing the bit,
    // then writing the new version back.
    //
    // Since we're executing from external flash, the QSPI peripheral is already
    // enabled, so we can't just write over the whole register.
    let mut reset = unsafe { RESETS_RESET.read_volatile() };
    reset &= !RESETS_IO_BANK0;
    unsafe {
        RESETS_RESET.write_volatile(reset);
    }

    // For some peripherals, it might take some time until they are initialized.
    // Not sure about GPIO, but let's be on the safe side and wait until the
    // `RESET_DONE` register lets us know it's ready.
    while unsafe { RESETS_RESET_DONE.read_volatile() } & RESETS_IO_BANK0 == 0 {}
}

fn select_sio_for_pin25() {
    // Pins on microcontrollers can be used by all kinds of hardware
    // peripherals. The purpose of this function is to configure the pin, so we
    // can control it directly.

    // Here's another register we're going to need. It's the control register
    // for pin 25. Pin 25 is what's connected to the LED on the Pi Pico.
    const GPIO25_CTRL: *mut u32 = 0x4001_40cc as *mut _;

    // Within the control register, there's a field that lets us select which
    // function controls the pin (FUNCSEL). The function that let's us control
    // the pin directly is called "SIO", and this is the value for selecting it.
    const GPIO_CTRL_FUNCSEL_SIO: u32 = 0x5;

    // Select SIO for pin 25, allowing us to control its output.
    //
    // There are more fields that FUNCSEL in the register, but we don't care
    // about any of them, and all their reset values are `0` anyway. We can just
    // write over all of it without having to take any special care.
    unsafe {
        GPIO25_CTRL.write_volatile(GPIO_CTRL_FUNCSEL_SIO);
    }
}

fn set_pin25_to_output() {
    // Finally, here's the function that sets the output pin to HIGH, resulting
    // in the LED to light up.

    // You know the drill by now: We need to write to some registers to control
    // the hardware. Here we define two more: `GPIO_OUT_SET` lets us control the
    // output of the pin (HIGH or LOW), which `GPIO_OE_SET` lets us put the pin
    // into output mode.
    const SIO_GPIO_OUT_SET: *mut u32 = 0xd000_0014 as *mut _;
    const SIO_GPIO_OE_SET: *mut u32 = 0xd000_0024 as *mut _;

    // Both of those registers have the same layout, with one bit per pin. Bit
    // 25 is the one for pin 25.
    const PIN25: u32 = 0x1 << 25;

    // Configure pin 25's output to be HIGH, then put it into output mode. We
    // could do it the other way around, but then the pin output might end up in
    // an undesired state, depending on what the value in the register was
    // before.
    //
    // The way those registers work, we're only changing anything for the one
    // pin whose bit we have set to `1`. The bits we leave at `0` don't cause
    // any change, so if we had configured other pins, we wouldn't overwrite
    // them here.
    unsafe {
        SIO_GPIO_OUT_SET.write_volatile(PIN25);
        SIO_GPIO_OE_SET.write_volatile(PIN25);
    }
}

// When a Rust program panics, usually the runtime prints out an error message,
// possibly with a stack trace. Again, we don't have an operating system, so we
// can't just print stuff!
//
// There are other things we can do, but let's keep it simple for now and just
// do nothing.
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
