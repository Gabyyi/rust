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
    
    let mut adc = Adc::new(peripherals.ADC, Irqs, AdcConfig::default());
    let mut photoresistor = AdcChannel::new_pin(peripherals.PIN_26, Pull::None);
    
    loop{
        let light_intensity = adc.read(&mut photoresistor).await.unwrap();
        info!("Light intensity: {}", light_intensity);
        if light_intensity<1365{
            red.set_duty_cycle_percent(0);
            yellow.set_duty_cycle_percent(100);
            blue.set_duty_cycle_percent(100);
        }
        else if light_intensity>=1365 && light_intensity<2730{
            red.set_duty_cycle_percent(100);
            yellow.set_duty_cycle_percent(0);
            blue.set_duty_cycle_percent(100);
        }
        else{
            red.set_duty_cycle_percent(100);
            yellow.set_duty_cycle_percent(100);
            blue.set_duty_cycle_percent(0);
        }
        Timer::after(Duration::from_millis(300)).await;
    }
}