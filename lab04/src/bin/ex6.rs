#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::{adc::{Adc, Async, Channel as ChannelAdc, Config as ConfigAdc, InterruptHandler}, bind_interrupts, config, gpio::{Input, Level, Output, Pull}, pwm::{Config as ConfigPwm, Pwm, SetDutyCycle}, Peripheral, PeripheralRef};
use embassy_time::{Duration, Timer, Instant};
use embassy_sync::{channel::Channel, pubsub::{PubSubChannel, Publisher, Subscriber, WaitResult::{Message as wrm, Lagged}}};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_futures::{select::{select, Either}, join::join};
use defmt::info;
use fixed::traits::ToFixed;
use {defmt_rtt as _, panic_probe as _};
use embassy_rp::gpio::AnyPin;

static CHANNEL: PubSubChannel<ThreadModeRawMutex, TrafficLight, 64, 4, 1> = PubSubChannel::new();

#[derive(Clone, Copy, PartialEq)]
enum TrafficLight {
    Green,
    Yellow,
    Red,
}

#[embassy_executor::task]
async fn traffic_light(
    mut red: PeripheralRef<'static, AnyPin>,
    mut yellow: PeripheralRef<'static, AnyPin>,
    mut green: PeripheralRef<'static, AnyPin>,
    mut button1: Input<'static>,
    mut button2: Input<'static>
) {
    let mut red = Output::new(&mut red, Level::High);
    let mut yellow = Output::new(&mut yellow, Level::High);
    let mut green = Output::new(&mut green, Level::High);

    let mut state = TrafficLight::Green;

    let mut publ = CHANNEL.publisher().unwrap();
    publ.publish(state).await;

    loop {
        match state {
            TrafficLight::Green => {
                green.set_low();
                yellow.set_high();
                red.set_high();

                match select(Timer::after_secs(5), join(button1.wait_for_falling_edge(), button2.wait_for_falling_edge())).await {
                    Either::First(_) | Either::Second(_) => {
                        info!("Green -> Yellow");
                        state = TrafficLight::Yellow;
                        publ.publish(state).await;
                    }
                }
            }
            TrafficLight::Yellow => {
                green.set_high();
                red.set_high();

                let mut elapsed = 0;

                while elapsed < 2000 {
                    if elapsed % 250 == 0 {
                        yellow.toggle();
                    }
                    match select(Timer::after(Duration::from_millis(50)), join(button1.wait_for_falling_edge(), button2.wait_for_falling_edge())).await {
                        Either::First(_) => {
                            elapsed += 50;
                        }
                        Either::Second(_) => {
                            info!("Buttons pressed during Yellow.");
                            break;
                        }
                    }
                }
                yellow.set_low();
                info!("Yellow -> Red");
                state = TrafficLight::Red;
                publ.publish(state).await;
            }
            TrafficLight::Red => {
                green.set_high();
                yellow.set_high();
                red.set_low();

                match select(Timer::after_secs(2), join(button1.wait_for_falling_edge(), button2.wait_for_falling_edge())).await {
                    Either::First(_) => {
                        info!("Red -> Green");
                        state = TrafficLight::Green;
                        publ.publish(state).await;
                    }
                    Either::Second(_) => {
                        info!("Buttons pressed. Red timer reset.");
                    }
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn buzz(
    mut buzzer: Pwm<'static>,
    mut config1: ConfigPwm, // Configuration for green/yellow (200Hz)
    mut config2: ConfigPwm  // Configuration for red (400Hz)
) {
    let mut subs = CHANNEL.subscriber().unwrap();

    loop {
        match subs.next_message().await {
            wrm(TrafficLight::Green) => {
                info!("Green buzzer.");
                buzzer.set_config(&config1);
                buzzer.set_duty_cycle(config1.top / 2); // 50% duty cycle
            }
            wrm(TrafficLight::Yellow) => {
                info!("Yellow buzzer.");
                buzzer.set_config(&config1);
                buzzer.set_duty_cycle(config1.top / 2); // 50% duty cycle
            }
            wrm(TrafficLight::Red) => {
                info!("Red buzzer.");
                buzzer.set_config(&config2);

                loop {
                    buzzer.set_duty_cycle(config2.top / 2); // ON
                    Timer::after(Duration::from_millis(500)).await;
                    buzzer.set_duty_cycle(0); // OFF
                    Timer::after(Duration::from_millis(500)).await;

                    if let Some(wrm(new_state)) = subs.try_next_message() {
                        if new_state != TrafficLight::Red {
                            // Interrupt the loop and handle the new state
                            if new_state == TrafficLight::Green {
                                info!("Green buzzer.");
                                buzzer.set_config(&config1);
                                buzzer.set_duty_cycle(config1.top / 2); // 50% duty cycle for green
                            }
                            break;
                        }
                    }
                }
            }
            Lagged(_) => {}
        }
    }
}

#[embassy_executor::task]
async fn serv (
    mut servo: Pwm<'static>,
    min_pulse: u16,
    max_pulse: u16
) {
    let mut subs = CHANNEL.subscriber().unwrap();
    servo.set_duty_cycle(min_pulse);

    loop {
        match subs.next_message().await {
            wrm(TrafficLight::Green) => {
                info!("Green servo.");
                let _ = servo.set_duty_cycle(max_pulse);
            } 
            wrm(TrafficLight::Yellow) => {
                info!("Yellow servo.");
                let _ = servo.set_duty_cycle(max_pulse / 2);
            }
            wrm(TrafficLight::Red) => {
                info!("Red servo.");
                let _ = servo.set_duty_cycle(min_pulse);
            }
            Lagged(_) => {}
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let red = AnyPin::from(p.PIN_6).into_ref();
    let yellow = AnyPin::from(p.PIN_4).into_ref();
    let green = AnyPin::from(p.PIN_2).into_ref();

    let button1 = Input::new(p.PIN_3, Pull::Up);
    let button2 = Input::new(p.PIN_5, Pull::Up);

        // Configure PWM for 200Hz frequency
    let mut config1: ConfigPwm = Default::default();
    config1.top = 2500; // Limits `top` within `u16`
    config1.divider = 250_i32.to_fixed(); // Set clock divider for 500 Hz
    config1.compare_b = config1.top / 2; // 50% duty cycle
    
    let mut config2: ConfigPwm = Default::default();
    config2.top = 1250;
    config2.divider = 250_i32.to_fixed(); // Set clock divider for 1 kHz
    config2.compare_a = config2.top / 2; // 50% duty cycle

    let buzzer = Pwm::new_output_b(p.PWM_SLICE7, p.PIN_15, ConfigPwm::default());

        // Configure PWM for servo control
    let mut servo_config: ConfigPwm = Default::default();

        // Set the calculated TOP value for 50 Hz PWM
    servo_config.top = 0xB71A; 
    
        // Set the clock divider to 64
    servo_config.divider = 64_i32.to_fixed(); // Clock divider = 64
    
        // Servo timing constants
    const PERIOD_US: usize = 20_000; // 20 ms period for 50 Hz
    const MIN_PULSE_US: usize = 500; // 0.5 ms pulse for 0 degrees
    const MAX_PULSE_US: usize = 2500; // 2.5 ms pulse for 180 degrees
    
        // Calculate the PWM compare values for minimum and maximum pulse widths
    let min_pulse = ((MIN_PULSE_US * servo_config.top as usize) / PERIOD_US) as u16;
    let max_pulse = ((MAX_PULSE_US * servo_config.top as usize) / PERIOD_US) as u16;
    
    let servo = Pwm::new_output_a(p.PWM_SLICE0, p.PIN_16, servo_config.clone());

    info!("Start");

    spawner.spawn(traffic_light(red, yellow, green, button1, button2)).unwrap();
    spawner.spawn(buzz(buzzer, config1, config2)).unwrap();
    spawner.spawn(serv(servo, min_pulse, max_pulse)).unwrap();
}

