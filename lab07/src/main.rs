#![no_std]
#![no_main]


use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_rp::{adc::{Adc, Channel as ChannelAdc, Config as ConfigAdc, InterruptHandler}, bind_interrupts, clocks::AdcClkSrc, gpio::{Input, Level, Output, Pull}, pac::spi, peripherals::{SPI0, SPI1}, pwm::{Config as ConfigPwm, Pwm, SetDutyCycle}, spi::{Async, Blocking, Config as ConfigSpi, Spi}, time_driver::init};
use embassy_time::{Duration, Timer, Delay};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::{raw::{NoopRawMutex, ThreadModeRawMutex}, NoopMutex};
use embassy_futures::select::{select, Either};
use defmt::info;
use st7735_lcd::{Orientation, ST7735};
use embassy_embedded_hal::{shared_bus::blocking::spi::SpiDevice};
use static_cell::StaticCell;
use embedded_graphics::{mono_font::{ascii::FONT_6X10, MonoTextStyle}, pixelcolor::Rgb565, prelude::*, text::Text};
use {defmt_rtt as _, panic_probe as _};

use itoa::{Buffer};

static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, SPI0, Blocking>>>> = StaticCell::new();
static CHANNEL_TEMP: Channel<ThreadModeRawMutex, u32, 64> = Channel::new();
static CHANNEL_PRESS: Channel<ThreadModeRawMutex, u32, 64> = Channel::new();

bind_interrupts!(struct Irqs {
});

#[embassy_executor::task]
async fn display_write(
    spi_bus: &'static NoopMutex<RefCell<Spi<'static, SPI0, Blocking>>>,
    mut cs: Output<'static>,
    mut ad: Output<'static>,
    mut reset: Output<'static>
) {
    let spi_dev = SpiDevice::new(spi_bus, cs);

    let mut display = ST7735::new(spi_dev, ad, core::prelude::v1::Some(reset), Default::default(), false, 130, 130);
    let mut delay = Delay;

    display.init(&mut delay).unwrap();
    display.set_orientation(&Orientation::LandscapeSwapped).unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::RED);

    // Draw text using embedded-graphics
    Text::new("Hello, World!", Point::new(20, 50), text_style)
        .draw(&mut display)
        .unwrap();

    info!("Display initialized and text written.");

    Timer::after_millis(1000).await;

    loop {
        display.clear(Rgb565::BLACK).unwrap();

        let temp_value = CHANNEL_TEMP.receive().await;
        let mut buffer1 = Buffer::new();
        let temp_str = buffer1.format(temp_value);
    
        Text::new("Temperature:", Point { x: 20, y: 20 }, text_style)
            .draw(&mut display)
            .unwrap();
        Text::new(temp_str, Point { x: 20, y: 40 }, text_style)
            .draw(&mut display)
            .unwrap();
        info!("Temp: {}", temp_str);
    
        let pres_value = CHANNEL_PRESS.receive().await;
        let mut buffer2 = Buffer::new();
        let pres_str = buffer2.format(pres_value);
        
        Text::new("Pressure:", Point { x: 20, y: 60 }, text_style)
            .draw(&mut display)
            .unwrap();
        Text::new(pres_str, Point { x: 20, y: 80 }, text_style)
            .draw(&mut display)
            .unwrap();
        info!("Pressure: {}", pres_str);
    
        Timer::after_millis(500).await;
    }    
}

#[embassy_executor::task]
async fn bmp_sensor(
    mut spi: Spi<'static, SPI1, Async>,
    mut cs: Output<'static>
) {
    cs.set_low();
    let mut id_buf = [0xD0 | 0x80, 0x00];
    let mut id_rx = [0u8; 2];
    spi.transfer(&mut id_rx, &id_buf).await.unwrap();
    cs.set_high();

    let sensor_id = id_rx[1];
    info!("Sensor ID: {:#X}", sensor_id);
    if sensor_id != 0x58 {
        info!("Error: Incorrect sensor ID! Check SPI wiring.");
        return;
    }


    cs.set_low();
    let write_buf = [0xF4, 0b01011011];
    spi.write(&write_buf).await.unwrap();
    cs.set_high();


    loop {

        cs.set_low();
        let mut status_buf = [0xF3 | 0x80, 0x00];
        let mut status_rx = [0u8; 2];
        spi.transfer(&mut status_rx, &status_buf).await.unwrap();
        cs.set_high();
        // info!("Status Register: {:#b}", status_rx[1]);

        let status = status_rx[1];
        if status & 0b00001000 != 0 {
            info!("Sensor is still measuring...");
            Timer::after_millis(10).await;
            continue;
        }


        cs.set_low();
        let mut temp_buf = [0xFA | 0x80, 0x00, 0x00, 0x00];
        let mut temp_rx = [0u8; 4];
        spi.transfer(&mut temp_rx, &temp_buf).await.unwrap();
        cs.set_high();

        let temp_msb = temp_rx[1];
        let temp_lsb = temp_rx[2];
        let temp_xlsb = temp_rx[3];

        let raw_temp = ((temp_msb as u32) << 12) | ((temp_lsb as u32) << 4) | ((temp_xlsb as u32) >> 4);
        CHANNEL_TEMP.send(raw_temp).await;


        cs.set_low();
        let mut pres_buf = [0xF7 | 0x80, 0x00, 0x00, 0x00];
        let mut pres_rx = [0u8; 4];
        spi.transfer(&mut pres_rx, &pres_buf).await.unwrap();
        cs.set_high();

        let pres_msb = pres_rx[1];
        let pres_lsb = pres_rx[2];
        let pres_xlsb = pres_rx[3];

        let raw_pres = ((pres_msb as u32) << 12) | ((pres_lsb as u32) << 4) | ((pres_xlsb as u32) >> 4);
        CHANNEL_PRESS.send(raw_pres).await;


        Timer::after_millis(500).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut spiconfig1 = ConfigSpi::default();
    spiconfig1.frequency = 16_000_000;

    let miso1 = p.PIN_16;
    let mosi1 = p.PIN_19;
    let clk1 = p.PIN_18;

    let mut spi1 = Spi::new_blocking(p.SPI0, clk1, mosi1, miso1, spiconfig1);
    let spi_bus = NoopMutex::new(RefCell::new(spi1));
    let spi_bus = SPI_BUS.init(spi_bus);

    let mut cs1 = Output::new(p.PIN_17, Level::High);
    let mut ad1 = Output::new(p.PIN_20, Level::Low);
    let mut reset1 = Output::new(p.PIN_21, Level::High);


    let mut spiconfig2 = ConfigSpi::default();
    spiconfig2.frequency = 2_000_000;

    let miso2 = p.PIN_12;
    let mosi2 = p.PIN_15;
    let clk2 = p.PIN_14;

    let mut spi2 = Spi::new(p.SPI1, clk2, mosi2, miso2, p.DMA_CH0, p.DMA_CH1, spiconfig2);

    let mut cs2 = Output::new(p.PIN_13, Level::High);

    spawner.spawn(display_write(spi_bus, cs1, ad1, reset1)).unwrap();
    spawner.spawn(bmp_sensor(spi2, cs2)).unwrap();
}

