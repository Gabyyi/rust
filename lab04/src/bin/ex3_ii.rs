#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
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
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Config::default());

    let mut red_led = Pwm::new_output_a(peripherals.PWM_SLICE1, peripherals.PIN_2, PwmConfig::default());
    let mut button1 = Input::new(peripherals.PIN_3, Pull::Up);
    let mut button2 = Input::new(peripherals.PIN_4, Pull::Up);

    let mut intensity = 50;
    red_led.set_duty_cycle_percent(intensity);

    loop {
        let button1_increase = button1.wait_for_falling_edge();
        let button2_decrease = button2.wait_for_falling_edge();
        match select(button1_increase, button2_decrease).await {
            Either::First(_) => {
                if intensity < 100 {
                    intensity += 10;
                }
                red_led.set_duty_cycle_percent(intensity);
                Timer::after_millis(100).await;
            }
            Either::Second(_) => {
                if intensity > 0 {
                    intensity -= 10;
                }
                red_led.set_duty_cycle_percent(intensity);
                Timer::after_millis(100).await;
            }
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}
