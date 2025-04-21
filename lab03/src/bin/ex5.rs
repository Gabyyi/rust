#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::adc::InterruptHandler;
use embassy_rp::config::Config;
use embassy_rp::pwm::SetDutyCycle;
use embassy_rp::{adc, bind_interrupts};
use embassy_rp::{
    adc::{Adc, Channel as AdcChannel, Config as AdcConfig},
    gpio, init, peripherals,
    pwm::{Config as PwmConfig, Pwm},
};
use embassy_time::{Duration, Timer};
use gpio::{Input, Pull};
use gpio::{Level, Output};
use fixed::traits::ToFixed;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals: embassy_rp::Peripherals = embassy_rp::init(Config::default());

    // let mut servo = Pwm::new_output_a(peripherals.PWM_SLICE1, peripherals.PIN_2, PwmConfig::default());

    let mut servo_config: PwmConfig = Default::default();
    servo_config.top = 0xB71A; 
    servo_config.divider = 64_i32.to_fixed();
    const PERIOD_US: usize = 20_000;
    const MIN_PULSE_US: usize = 500;
    const MAX_PULSE_US: usize = 2500;

    let min_pulse = (MIN_PULSE_US * servo_config.top as usize) / PERIOD_US;
    let max_pulse = (MAX_PULSE_US * servo_config.top as usize) / PERIOD_US;

    let mut servo = Pwm::new_output_a(
        peripherals.PWM_SLICE1, 
        peripherals.PIN_2, 
        servo_config.clone()
    );

    loop{
        servo.set_duty_cycle_percent(0);
        for i in (0..180){
            // servo.set_duty_cycle((min_pulse+(max_pulse-min_pulse)*i as usize / 180) as u16);
            // servo.set_config((min_pulse+(max_pulse-min_pulse)*i as usize / 180) as u16);
            servo.set_duty_cycle(max_pulse as u16);
            Timer::after(Duration::from_millis(20)).await;
        }
        for i in (0..180).rev(){
            servo.set_duty_cycle((min_pulse+(max_pulse-min_pulse)*i as usize / 180) as u16);
            servo.set_duty_cycle(min_pulse as u16);
            Timer::after(Duration::from_millis(20)).await;
        }
        servo.set_duty_cycle(max_pulse as u16);
        Timer::after(Duration::from_millis(1000)).await;
        servo.set_duty_cycle(min_pulse as u16);
        Timer::after(Duration::from_millis(1000)).await;
    }
}