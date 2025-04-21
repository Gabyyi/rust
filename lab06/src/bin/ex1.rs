#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::config::Config;
use embassy_rp::pwm::SetDutyCycle;
use embassy_rp::{
    bind_interrupts, gpio, init, peripherals,
    pwm::{Config as PwmConfig, Pwm},
    spi::{self, Spi},
};
use embassy_time::{Duration, Timer};
use gpio::{Input, Pull};
use gpio::{Level, Output};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_rp::i2c::{I2c, InterruptHandler as I2CInterruptHandler, Config as I2cConfig};
use embedded_hal_async::i2c::{Error, I2c as _};
use embassy_rp::peripherals::I2C0;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let sda = peripherals.PIN_4;
    let scl = peripherals.PIN_5;
    let mut i2c = I2c::new_async(peripherals.I2C0, scl, sda, Irqs, I2cConfig::default());
    
    info!("Starting I2C scan...");
    for address in 0x08..=0x77 {
        let result = i2c.write_async(address as u16, [0x01]).await;
        if result.is_ok() {
            info!("Device found at address: 0x{:02X}", address);
        }
        Timer::after(Duration::from_millis(10)).await;
    }
    info!("I2C scan complete.");
}