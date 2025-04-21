#![no_std]
#![no_main]
#![allow(unused_imports)]

extern crate alloc;

use core::{net::Ipv4Addr, str};
use alloc_cortex_m::CortexMHeap;

use cyw43::JoinOptions;
use embassy_executor::Spawner;
use embassy_rp::{adc::{Adc, Channel as ChannelAdc, Config as ConfigAdc, InterruptHandler as AdcInterruptHandler}, bind_interrupts, clocks::{RoscRng, AdcClkSrc}, config, gpio::{Input, Level, Output, Pull}, peripherals::{DMA_CH0, PIO0, SPI0, SPI1}, pio::{InterruptHandler as PioInterruptHandler, Pio}, pwm::{Config as ConfigPwm, Pwm, SetDutyCycle}, spi::{Async, Blocking, Config as ConfigSpi, Spi}, time_driver::init};
use embassy_time::{Duration, Timer, Delay};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::{raw::{NoopRawMutex, ThreadModeRawMutex}, NoopMutex};
use embassy_futures::{join, select::{select, Either}};
use defmt::{info, unwrap};

use static_cell::StaticCell;
use rand::RngCore as _;

use {defmt_rtt as _, panic_probe as _};

use itoa::{Buffer};

use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use embassy_net::{Config as NetConfig, Ipv4Cidr, StackResources, StaticConfigV4};


bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();


#[embassy_executor::task]
async fn cyw43_task(runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let wifi_ssid = env!("WIFI_SSID");
    let wifi_password = env!("WIFI_PASSWORD");
    
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

    let config = NetConfig::dhcpv4(Default::default());
    // let config = NetConfig::ipv4_static(StaticConfigV4 {
    //     address: Ipv4Cidr::new(Ipv4Addr::new(192, 168, 69, 2), 24),
    //     dns_servers: heapless::Vec::<Ipv4Addr, 3>::new(),
    //     gateway: Some(Ipv4Addr::new(192, 168, 69, 1)),
    // });
   
    
    let seed = RoscRng.next_u64();

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(_net_device, config, RESOURCES.init(StackResources::new()), seed);

    unwrap!(spawner.spawn(net_task(runner)));

    loop {
        match control 
            // .join(wifi_ssid, JoinOptions::new_open())
            .join(wifi_ssid, JoinOptions::new(wifi_password.as_bytes()))
            .await
        {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }

    info!("waiting for dhcp");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up");

    if let Some(config) = stack.config_v4() {
        info!("IP Address: {}", config.address.address());
    } else {
        info!("Failed to retrieve IP address");
    }
}

