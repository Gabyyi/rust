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

    // MPU-6500 register addresses
    const PWR_MGMT_1: u8 = 0x6B;
    const ACCEL_FS_SEL: u8 = 0x1C;
    const GYRO_FS_SEL: u8 = 0x1B;
    const ACCEL_XOUT_H: u8 = 0x3B;
    const GYRO_XOUT_H: u8 = 0x43;

    const ACCEL_SENSITIVITY: f32 = 16384.0;
    const GYRO_SENSITIVITY: f32 = 32.8;

    cs.set_low();
    spi.write(&[PWR_MGMT_1, 0x00]).await.unwrap();
    cs.set_high();

    cs.set_low();
    spi.write(&[ACCEL_FS_SEL, 0x00]).await.unwrap();
    cs.set_high();

    cs.set_low();
    spi.write(&[GYRO_FS_SEL, 0x10]).await.unwrap();
    cs.set_high();

    loop {
        cs.set_low();
        let tx_buffer = [ACCEL_XOUT_H | 0x80];
        let mut rx_buffer = [0x00, 0x00];
        spi.transfer(&mut rx_buffer, &tx_buffer).await.unwrap();
        cs.set_high();

        let accel_x_raw = i16::from_be_bytes([rx_buffer[0], rx_buffer[1]]);
        let accel_x = (accel_x_raw as f32) / ACCEL_SENSITIVITY * 9.80665;
        info!("Acceleration X: {} m/s² ", accel_x);

        cs.set_low();
        let tx_buffer = [GYRO_XOUT_H | 0x80];
        let mut rx_buffer = [0x00, 0x00];
        spi.transfer(&mut rx_buffer, &tx_buffer).await.unwrap();
        cs.set_high();

        let gyro_x_raw = i16::from_be_bytes([rx_buffer[0], rx_buffer[1]]);
        let gyro_x = (gyro_x_raw as f32) / GYRO_SENSITIVITY;
        info!("Angular Velocity X: {} °/s", gyro_x);

        Timer::after(Duration::from_millis(500)).await;
    }
}