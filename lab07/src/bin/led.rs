#![no_std]
#![no_main]
#![allow(unused_imports)]

extern crate alloc;

use core::{cell::RefCell, net::Ipv4Addr, str::from_utf8};
use alloc::rc::Rc;
use alloc_cortex_m::CortexMHeap;

use cyw43::JoinOptions;
use embassy_executor::Spawner;
use embassy_rp::{adc::{Adc, Channel as ChannelAdc, Config as ConfigAdc, InterruptHandler as AdcInterruptHandler}, bind_interrupts, clocks::{RoscRng, AdcClkSrc}, config, gpio::{Input, Level, Output, Pull}, peripherals::{DMA_CH0, PIO0, SPI0, SPI1}, pio::{InterruptHandler as PioInterruptHandler, Pio}, pwm::{Config as ConfigPwm, Pwm, SetDutyCycle}, spi::{Async, Blocking, Config as ConfigSpi, Spi}, time_driver::init};
use embassy_time::{Duration, Timer, Delay};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::{raw::{NoopRawMutex, ThreadModeRawMutex}, NoopMutex};
use embassy_futures::{join, select::{select, Either}};
use defmt::{info, unwrap, warn};

use static_cell::StaticCell;
use rand::RngCore as _;

use {defmt_rtt as _, panic_probe as _};

use itoa::{Buffer};

use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use embassy_net::{tcp::TcpSocket, Config as NetConfig, Ipv4Cidr, StackResources, StaticConfigV4};


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

    let firmware = include_bytes!("/home/andrei/Documents/School/Microprocessors/embassy/cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("/home/andrei/Documents/School/Microprocessors/embassy/cyw43-firmware/43439A0_clm.bin");

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

    let mut l1 = Output::new(p.PIN_16, Level::Low);
    let mut l2 = Output::new(p.PIN_17, Level::Low);
    let mut l3 = Output::new(p.PIN_18, Level::Low);

    let config = NetConfig::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Addr::new(169, 42, 1, 1), 24),
        dns_servers: heapless::Vec::<Ipv4Addr, 3>::new(),
        gateway: None,
    });
   
    let seed = RoscRng.next_u64();

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(_net_device, config, RESOURCES.init(StackResources::new()), seed);

    unwrap!(spawner.spawn(net_task(runner)));

    let pico_ssid = "jones";
    let pico_password = "jack1234";

    control.start_ap_wpa2(pico_ssid, pico_password, 6).await;

    let mut rx_buf = [0; 4096];
    let mut tx_buf = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack,  &mut rx_buf, &mut tx_buf);
        socket.set_timeout(Some(Duration::from_secs(10)));

        control.gpio_set(0, false).await;
        info!("Listening on TCP: 6000");
        if let Err(e) = socket.accept(6000).await {
            warn!("Accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());
        control.gpio_set(0, true).await;

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("Read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("Read error: {:?}", e);
                    break;
                }
            };

            match from_utf8(&buf[..n]).unwrap().trim() {
                "red:on" => l1.set_high(),
                "red:off" => l1.set_low(),
                "green:on" => l2.set_high(),
                "green:off" => l2.set_low(),
                "blue:on" => l3.set_high(),
                "blue:off" => l3.set_low(),
                _ => warn!("Unknown command: {:?}", from_utf8(&buf[..n]).unwrap()),
            }

            Timer::after(Duration::from_millis(100)).await;
        }
    }
    
}

