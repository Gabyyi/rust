#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::config::Config;
use embassy_rp::pwm::SetDutyCycle;
use embassy_rp::{
    gpio, init, peripherals,
    pwm::{Config as PwmConfig, Pwm},
    spi::{self, Spi},
};
use embassy_time::{Duration, Timer};
use gpio::{Input, Pull};
use gpio::{Level, Output};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use panic_probe as _;

static SIG: Signal<CriticalSectionRawMutex, u8> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    // SPI configuration
    let mut config = spi::Config::default();
    config.frequency = 1_000_000; // 1 MHz for MPU-6500
    config.phase = spi::Phase::CaptureOnFirstTransition;
    config.polarity = spi::Polarity::IdleHigh;

    // default values are fine
    let mut config = spi::Config::default();

    let miso = peripherals.PIN_4;
    let mosi = peripherals.PIN_3;
    let clk = peripherals.PIN_2;

    let mut spi = Spi::new(peripherals.SPI0, clk, mosi, miso, peripherals.DMA_CH0, peripherals.DMA_CH1, config);

    // make sure to actually choose a pin
    let mut cs = Output::new(peripherals.PIN_5, Level::High);

    cs.set_low();
    const WHO_AM_I: u8 = 0x75;
    let tx_buffer = [WHO_AM_I | 0x80, 0x00];
    let mut rx_buffer = [0u8; 2];
    // spi.transfer(&mut rx_buffer, &tx_buffer).await.unwrap();
    // info!("TX buffer: {:?}", tx_buffer);
    // info!("RX buffer before transfer: {:?}", rx_buffer);
    spi.transfer(&mut rx_buffer, &tx_buffer).await.unwrap();
    // info!("RX buffer after transfer: {:?}", rx_buffer);
    cs.set_high();

    let who_am_i = rx_buffer[1];
    info!("WHO_AM_I register: {=u8}", who_am_i);

    if who_am_i == 0x70 {
        info!("WHO_AM_I matches the MPU-6500 datasheet!");
    } else {
        panic!("WHO_AM_I does not match the MPU-6500 datasheet!");
    }

    embassy_time::Timer::after(Duration::from_secs(1)).await;
}