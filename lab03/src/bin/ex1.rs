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
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Config::default());

    let mut led = Pwm::new_output_a(peripherals.PWM_SLICE1, peripherals.PIN_2, PwmConfig::default());

    loop{
        led.set_duty_cycle_percent(75);
        Timer::after_secs(2).await;
        led.set_duty_cycle_percent(100);
        Timer::after_secs(2).await;
        led.set_duty_cycle_percent(0);
        for i in (0..100).rev().step_by(10){
            led.set_duty_cycle_percent(i);
            Timer::after_secs(1).await;
        }
        for i in (0..100).step_by(10){
            led.set_duty_cycle_percent(i);
            Timer::after_secs(1).await;
        }
        Timer::after_secs(2).await;
    }
}