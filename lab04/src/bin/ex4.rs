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

enum TrafficLightState {
    Green,
    Yellow,
    Red,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Config::default());

    let mut green_led = Pwm::new_output_a(peripherals.PWM_SLICE1, peripherals.PIN_2, PwmConfig::default());
    let mut yellow_led = Pwm::new_output_a(peripherals.PWM_SLICE2, peripherals.PIN_4, PwmConfig::default());
    let mut red_led = Pwm::new_output_a(peripherals.PWM_SLICE3, peripherals.PIN_6, PwmConfig::default());
    let mut button = Input::new(peripherals.PIN_5, Pull::Up);

    let mut state = TrafficLightState::Green;

    loop {
        match state {
            TrafficLightState::Green => {
                green_led.set_duty_cycle_percent(0);
                yellow_led.set_duty_cycle_percent(100);
                red_led.set_duty_cycle_percent(100);

                let timer = Timer::after(Duration::from_secs(5));
                let button_future = async {
                    while button.is_high() {
                        Timer::after(Duration::from_millis(10)).await;
                    }
                };

                match select(timer, button_future).await {
                    Either::First(_) => {
                        state = TrafficLightState::Yellow;
                    }
                    Either::Second(_) => {
                        state = TrafficLightState::Red;
                    }
                }
            }

            TrafficLightState::Yellow => {
                green_led.set_duty_cycle_percent(100);
                red_led.set_duty_cycle_percent(100);
                for _ in 0..4 {
                    yellow_led.set_duty_cycle_percent(0);
                    Timer::after(Duration::from_millis(125)).await;
                    yellow_led.set_duty_cycle_percent(100);
                    Timer::after(Duration::from_millis(125)).await;
                }

                let timer = Timer::after(Duration::from_secs(1));
                let button_future = async {
                    while button.is_high() {
                        Timer::after(Duration::from_millis(10)).await;
                    }
                };

                match select(timer, button_future).await {
                    Either::First(_) => {
                        state = TrafficLightState::Red;
                    }
                    Either::Second(_) => {
                        state = TrafficLightState::Red;
                    }
                }
            }

            TrafficLightState::Red => {
                green_led.set_duty_cycle_percent(100);
                yellow_led.set_duty_cycle_percent(100);
                red_led.set_duty_cycle_percent(0);

                let timer = Timer::after(Duration::from_secs(2));
                let button_future = async {
                    while button.is_high() {
                        Timer::after(Duration::from_millis(10)).await;
                    }
                };

                match select(timer, button_future).await {
                    Either::First(_) => {
                        state = TrafficLightState::Green;
                    }
                    Either::Second(_) => {
                        state = TrafficLightState::Red;
                    }
                }
            }
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}
