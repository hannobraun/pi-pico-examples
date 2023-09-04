#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Timer};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("Initializing...");

    let peripherals = embassy_rp::init(Default::default());
    let mut led = Output::new(peripherals.PIN_25, Level::Low);

    defmt::info!("Initialized.");

    loop {
        defmt::info!("LED on!");
        led.set_high();
        Timer::after(Duration::from_millis(50)).await;

        defmt::info!("LED off!");
        led.set_low();
        Timer::after(Duration::from_millis(950)).await;
    }
}
