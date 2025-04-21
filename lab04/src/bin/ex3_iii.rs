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
};
use embassy_time::{Duration, Timer};
use gpio::{Input, Pull};
use gpio::{Level, Output};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use panic_probe as _;

static SIG: Signal<CriticalSectionRawMutex, u8> = Signal::new();

#[embassy_executor::task]
async fn button_task(button1: Input<'static>, button2: Input<'static>){
    let mut intensity=50;
    loop{
        if button1.is_low(){
            if(intensity<100){
                intensity+=10;
            }
            SIG.signal(intensity);
            Timer::after(Duration::from_millis(200)).await;
        }
        if button2.is_low(){
            if(intensity>0){
                intensity-=10;
            }
            SIG.signal(intensity);
            Timer::after(Duration::from_millis(200)).await;
        }
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Config::default());

    let mut red_led = Pwm::new_output_a(peripherals.PWM_SLICE1, peripherals.PIN_2, PwmConfig::default());
    let mut button1 = Input::new(peripherals.PIN_3, Pull::Up);
    let mut button2 = Input::new(peripherals.PIN_4, Pull::Up);

    spawner.spawn(button_task(button1,button2)).unwrap();

    loop{
        let intensity=SIG.wait().await;
        red_led.set_duty_cycle_percent(intensity);
        Timer::after_millis(100).await;
    }
}