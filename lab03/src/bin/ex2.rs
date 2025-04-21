#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::adc::InterruptHandler;
use embassy_rp::pwm::Config as ConfigPwm;
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
    let peripherals = embassy_rp::init(Config::default());
    let mut config: ConfigPwm = Default::default();
    
    let mut led = Pwm::new_output_a(peripherals.PWM_SLICE1, peripherals.PIN_2, PwmConfig::default());

    let mut adc = Adc::new(peripherals.ADC, Irqs, AdcConfig::default());
    let mut potentiometer = AdcChannel::new_pin(peripherals.PIN_26, Pull::None);

    let mut min_value=0;
    let mut max_value=4095;

    loop {
        let value = adc.read(&mut potentiometer).await.unwrap();
        info!("Potentiometer value: {}", value);
        
        let min_value = min_value as u32;
        let max_value = max_value as u32;
        let value = value as u32;
    
        let duty_cycle = ((value - min_value) * 100) / (max_value/2 - min_value);
        
        led.set_duty_cycle_percent(duty_cycle as u8);
        Timer::after_millis(200).await;
    }
}