// +---------------------------------------------------------------------------+
// |                            Embassy March 2025                             |
// |                                   Demo                                    |
// +---------------------------------------------------------------------------+

//! By default, this app blinks and LED connected to the `GP4` pin.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::Output;
use embassy_time::{Duration, Timer};
// Use the `panic_probe` crate to provided the panic handler and the 
// `defmt_rtt` to import the runtime for defmt, to be able to use
// the print macros.
use {defmt_rtt as _, panic_probe as _};
// Use the logging macros provided by defmt.
#[allow(unused)]
use defmt::*;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Get a handle to the RP's peripherals.
    let p = embassy_rp::init(Default::default());

    // Configure pin 4 as output and use the initial level `High`. Keep in mind
    // that the on board LEDs have their Anode connected to +5V.
    let mut led = Output::new(p.PIN_4, embassy_rp::gpio::Level::High);

    let delay = Duration::from_secs(1);
    loop {
        led.toggle();
        // Wait one second.
        Timer::after(delay).await;
    }
}
