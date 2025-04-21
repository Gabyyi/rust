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
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use gpio::{Input, Pull};
use gpio::{Level, Output};
use panic_probe as _;

static CHANNEL: Channel<ThreadModeRawMutex, State, 64> = Channel::new();

enum State{
    Increase,
    Decrease,
}

#[embassy_executor::task]
async fn button1_increase(button1: Input<'static>){
    loop{
        if button1.is_low(){
            CHANNEL.send(State::Increase).await;
            Timer::after(Duration::from_millis(200)).await;
        }
        Timer::after_millis(100).await;
    }
}

#[embassy_executor::task]
async fn button2_decrease(button2: Input<'static>){
    loop{
        if button2.is_low(){
            CHANNEL.send(State::Decrease).await;
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

    spawner.spawn(button1_increase(button1)).unwrap();
    spawner.spawn(button2_decrease(button2)).unwrap();

    let mut intensity=50;
    red_led.set_duty_cycle_percent(intensity);
    loop{
        let value = CHANNEL.receive().await;
        match value {
            State::Increase => {
                if intensity<100{
                    intensity+=10;
                    red_led.set_duty_cycle_percent(intensity);
                }
            }
            State::Decrease => {
                if intensity>0{
                    intensity-=10;
                    red_led.set_duty_cycle_percent(intensity);
                }
            }
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}