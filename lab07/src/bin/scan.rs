#![no_std]
#![no_main]
#![allow(unused_imports)]

use core::str;

use embassy_executor::Spawner;
use embassy_rp::{adc::{Adc, Channel as ChannelAdc, Config as ConfigAdc, InterruptHandler as AdcInterruptHandler}, bind_interrupts, clocks::AdcClkSrc, gpio::{Input, Level, Output, Pull}, peripherals::{DMA_CH0, PIO0, SPI0, SPI1}, pio::{InterruptHandler as PioInterruptHandler, Pio}, pwm::{Config as ConfigPwm, Pwm, SetDutyCycle}, spi::{Async, Blocking, Config as ConfigSpi, Spi}, time_driver::init};
use embassy_time::{Duration, Timer, Delay};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::{raw::{NoopRawMutex, ThreadModeRawMutex}, NoopMutex};
use embassy_futures::select::{select, Either};
use defmt::{info, unwrap};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use itoa::{Buffer};

use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};


bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let firmware = include_bytes!("/home/gabi/rust/embassy/cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("/home/gabi/rust/embassy/cyw43-firmware/43439A0_clm.bin");

    // Initialize the WiFi module
    let power = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);

    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0, 
        DEFAULT_CLOCK_DIVIDER, 
        pio.irq0, 
        cs, 
        p.PIN_24, 
        p.PIN_29, 
        p.DMA_CH0,
    );

    info!("Initializing WiFi module");

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, power, spi, firmware).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::Performance)
        .await;

    info!("WiFi module initialized");
    Timer::after(Duration::from_millis(500)).await;

    let mut scanner = control.scan(Default::default()).await;

    while let Some(bss) = scanner.next().await {
        if let Ok(ssid_str) = str::from_utf8(&bss.ssid) {
            info!("scanned {} == {:x}", ssid_str, bss.bssid);
        }
    }
}

