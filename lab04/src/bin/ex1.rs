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
use embassy_time::{Duration, Instant, Timer};
use gpio::{Input, Pull};
use gpio::{Level, Output};
use panic_probe as _;

#[embassy_executor::task(pool_size = 2)]
async fn blink_led(mut led: Pwm<'static>, time_interval: u64){
    loop{
        led.set_duty_cycle_percent(0);
        let start_time=Instant::now();
        while start_time.elapsed().as_millis() < time_interval {}

        led.set_duty_cycle_percent(100);
        let start_time=Instant::now();
        while start_time.elapsed().as_millis() < time_interval {}

        Timer::after_millis(100).await;
    }
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Config::default());

    let mut red_led = Pwm::new_output_a(peripherals.PWM_SLICE1, peripherals.PIN_2, PwmConfig::default());
    let mut blue_led = Pwm::new_output_a(peripherals.PWM_SLICE2, peripherals.PIN_4, PwmConfig::default());

    spawner.spawn(blink_led(red_led,1000)).unwrap();
    spawner.spawn(blink_led(blue_led,1000)).unwrap();
}