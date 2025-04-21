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
use panic_probe as _;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals: embassy_rp::Peripherals = embassy_rp::init(Config::default());

    let mut red = Pwm::new_output_a(peripherals.PWM_SLICE1, peripherals.PIN_2, PwmConfig::default());
    let mut yellow = Pwm::new_output_a(peripherals.PWM_SLICE2, peripherals.PIN_4, PwmConfig::default());
    let mut blue = Pwm::new_output_a(peripherals.PWM_SLICE3, peripherals.PIN_6, PwmConfig::default());
    let button = Input::new(peripherals.PIN_3, Pull::Up);
    let mut state=0;
    loop{
        if button.is_low(){
            info!("The button was pressed. State: {}", state);
            match state{
                0 =>{
                    red.set_duty_cycle_percent(0);
                    yellow.set_duty_cycle_percent(100);
                    blue.set_duty_cycle_percent(100);
                }
                1 =>{
                    red.set_duty_cycle_percent(100);
                    yellow.set_duty_cycle_percent(0);
                    blue.set_duty_cycle_percent(100);
                }
                2 =>{
                    red.set_duty_cycle_percent(100);
                    yellow.set_duty_cycle_percent(100);
                    blue.set_duty_cycle_percent(0);
                }
                _ =>{}
            }
            state=(state+1)%3;
            Timer::after(Duration::from_millis(300)).await;
        }
    }
}