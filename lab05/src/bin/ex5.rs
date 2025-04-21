#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, panic};
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_rp::config::Config;
use embassy_rp::pwm::SetDutyCycle;
use embassy_rp::{
    gpio, init, peripherals,
    pwm::{Config as PwmConfig, Pwm},
    spi::{self, Spi, Config as SpiConfig},
};
use embassy_time::{Duration, Timer, Delay};
use gpio::{Input, Pull};
use gpio::{Level, Output};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use panic_probe as _;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_sync::blocking_mutex::{Mutex, raw::NoopRawMutex};
use core::cell::RefCell;
use display_interface_spi::SPIInterface;
use mipidsi::models::ST7735s;
use mipidsi::options::{Orientation, Rotation};
use embedded_hal_1::spi::SpiDevice as _;
use heapless::String;
use core::fmt::Write;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    // SPI configuration
    // let mut config = spi::Config::default();
    // config.frequency = 1_000_000; // 1 MHz for MPU-6500
    // config.phase = spi::Phase::CaptureOnFirstTransition;
    // config.polarity = spi::Polarity::IdleHigh;

    // // default values are fine
    // let mut config = spi::Config::default();

    let miso = peripherals.PIN_4;
    let mosi = peripherals.PIN_3;
    let clk = peripherals.PIN_2;

    // let spi = Spi::new(peripherals.SPI0, clk, mosi, miso, peripherals.DMA_CH0, peripherals.DMA_CH1, config);

    // make sure to actually choose a pin
    // let mut cs = Output::new(peripherals.PIN_5, Level::High);



    let mut screen_config = embassy_rp::spi::Config::default();
    screen_config.frequency = 32_000_000u32;
    screen_config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    screen_config.polarity = embassy_rp::spi::Polarity::IdleHigh;

    let screen_rst = Output::new(peripherals.PIN_6, Level::Low);
    let screen_dc = Output::new(peripherals.PIN_7, Level::Low);
    let screen_cs = Output::new(peripherals.PIN_9, Level::High);

    let spi = Spi::new_blocking(peripherals.SPI0, clk, mosi, miso, screen_config);
    let spi_bus: Mutex<NoopRawMutex, _> = Mutex::new(RefCell::new(spi));
    let display_spi = SpiDevice::new(&spi_bus, screen_cs);

    let di = SPIInterface::new(display_spi, screen_dc);
    let mut screen = mipidsi::Builder::new(ST7735s, di)
        .reset_pin(screen_rst)
        .orientation(Orientation::new().rotate(Rotation::Deg180))
        .init(&mut Delay)
        .unwrap();

    let imu_cs = Output::new(peripherals.PIN_5, Level::High);
    let imu_config = SpiConfig::default();
    let mut imu_spi = SpiDeviceWithConfig::new(&spi_bus, imu_cs, imu_config);



    // MPU-6500 register addresses
    const PWR_MGMT_1: u8 = 0x6B;
    const ACCEL_FS_SEL: u8 = 0x1C;
    const GYRO_FS_SEL: u8 = 0x1B;
    const ACCEL_XOUT_H: u8 = 0x3B;
    const GYRO_XOUT_H: u8 = 0x43;

    // Sensitivity values for full-scale configurations
    const ACCEL_SENSITIVITY: f32 = 16384.0;
    const GYRO_SENSITIVITY: f32 = 32.8;

    imu_spi.write(&[PWR_MGMT_1, 0x00]).unwrap();

    imu_spi.write(&[ACCEL_FS_SEL, 0x00]).unwrap();

    imu_spi.write(&[GYRO_FS_SEL, 0x10]).unwrap();

    loop {
        // imu_cs.set_low();
        let tx_buffer = [ACCEL_XOUT_H | 0x80];
        let mut rx_buffer = [0x00, 0x00];
        imu_spi.transfer(&mut rx_buffer, &tx_buffer).unwrap();
        // imu_cs.set_high();

        let accel_x_raw = i16::from_be_bytes([rx_buffer[0], rx_buffer[1]]);
        let accel_x = (accel_x_raw as f32) / ACCEL_SENSITIVITY * 9.80665;
        info!("Acceleration X: {} m/s²", accel_x);

        // imu_cs.set_low();
        let tx_buffer = [GYRO_XOUT_H | 0x80];
        let mut rx_buffer = [0x00, 0x00];
        imu_spi.transfer(&mut rx_buffer, &tx_buffer).unwrap();
        // imu_cs.set_high();

        let gyro_x_raw = i16::from_be_bytes([rx_buffer[0], rx_buffer[1]]);
        let gyro_x = (gyro_x_raw as f32) / GYRO_SENSITIVITY;
        info!("Angular Velocity X: {} °/s", gyro_x);

        let mut buf = String::<64>::new();
        write!(
            &mut buf,
            "Accel X: {}.{}\nGyro X: {}.{}",
            accel_x as u32,
            (accel_x * 1000.0) as u32 % 1000,
            gyro_x as u32,
            (gyro_x * 1000.0) as u32 % 1000
        )
        .unwrap();

        screen.clear(Rgb565::BLACK).unwrap();
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        Text::new(&buf, Point::new(10, 20), text_style)
            .draw(&mut screen)
            .unwrap();

        Timer::after(Duration::from_millis(500)).await;
    }
}