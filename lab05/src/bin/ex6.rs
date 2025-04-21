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
use micromath::F32Ext; // For atan2 and sqrt
use embedded_graphics::primitives::Circle;
use embedded_graphics::primitives::PrimitiveStyle;

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

    // Sensor configuration
    let imu_cs = Output::new(peripherals.PIN_5, Level::High);
    let imu_config = SpiConfig::default();
    let mut imu_spi = SpiDeviceWithConfig::new(&spi_bus, imu_cs, imu_config);


    const PWR_MGMT_1: u8 = 0x6B;
    const ACCEL_FS_SEL: u8 = 0x1C;
    const GYRO_FS_SEL: u8 = 0x1B;
    const ACCEL_XOUT_H: u8 = 0x3B;
    const GYRO_XOUT_H: u8 = 0x43;

    const ACCEL_SENSITIVITY: f32 = 16384.0;
    const GYRO_SENSITIVITY: f32 = 32.8;

    imu_spi.write(&[PWR_MGMT_1, 0x00]).unwrap();

    imu_spi.write(&[ACCEL_FS_SEL, 0x00]).unwrap();

    const SCREEN_WIDTH: i32 = 160;
    const SCREEN_HEIGHT: i32 = 128;

    let mut ball_x = SCREEN_WIDTH / 2;
    let mut ball_y = SCREEN_HEIGHT / 2;
    const BALL_RADIUS: i32 = 5;
    let ball_radius = 10;

    loop {
        screen.clear(Rgb565::BLACK).unwrap();
        let mut buffer = [ACCEL_XOUT_H | 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut read_buffer = [0u8; 7];
        imu_spi.transfer(&mut buffer, &mut read_buffer).unwrap();

        let acc_x_raw = i16::from_be_bytes([buffer[1], buffer[2]]);
        let acc_y_raw = i16::from_be_bytes([buffer[3], buffer[4]]);
        let acc_z_raw = i16::from_be_bytes([buffer[5], buffer[6]]);

        let acc_x = (acc_x_raw as f32) / ACCEL_SENSITIVITY;
        let acc_y = (acc_y_raw as f32) / ACCEL_SENSITIVITY;
        let acc_z = (acc_z_raw as f32) / ACCEL_SENSITIVITY;

        let pitch = f32::atan2(acc_y, f32::sqrt(acc_x * acc_x + acc_z * acc_z)).to_degrees();
        let roll = f32::atan2(-acc_x, f32::sqrt(acc_y * acc_y + acc_z * acc_z)).to_degrees();



        let mut new_ball_x = (ball_x as f32 + roll / 10.0) as i32;
        let mut new_ball_y = (ball_y as f32 - pitch / 10.0) as i32;

        // ball_x = new_ball_x.clamp(BALL_RADIUS, SCREEN_WIDTH - BALL_RADIUS);
        // ball_y = new_ball_y.clamp(BALL_RADIUS, SCREEN_HEIGHT - BALL_RADIUS);

        if new_ball_x < BALL_RADIUS {
            new_ball_x = BALL_RADIUS;
        } else if new_ball_x > SCREEN_WIDTH - BALL_RADIUS {
            new_ball_x = SCREEN_WIDTH - BALL_RADIUS;
        }
        
        if new_ball_y < BALL_RADIUS {
            new_ball_y = BALL_RADIUS;
        } else if new_ball_y > SCREEN_HEIGHT - BALL_RADIUS {
            new_ball_y = SCREEN_HEIGHT - BALL_RADIUS;
        }

        Circle::new(Point::new(ball_x, ball_y), BALL_RADIUS as u32)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut screen)
            .unwrap();

        Circle::new(Point::new(new_ball_x, new_ball_y), BALL_RADIUS as u32)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(&mut screen)
            .unwrap();

        ball_x = new_ball_x;
        ball_y = new_ball_y;

        Timer::after(Duration::from_millis(50)).await;
    }
}