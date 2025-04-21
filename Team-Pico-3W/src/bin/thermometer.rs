#![no_main]
#![no_std]


use bmp280_ehal::{self, BMP280, Control, Oversampling, PowerMode};
use core::array;
use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::I2C1;
use embassy_rp::pwm::{Config as ConfigPmw, Pwm};
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;
use fixed::traits::ToFixed;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let mut configred: ConfigPmw = Default::default();
    configred.top = 0x9088;
    configred.compare_b = configred.top;

    let mut configgreenblue: ConfigPmw = Default::default();
    configgreenblue.top = 0x9088;
    configgreenblue.compare_b = 0;
    configgreenblue.compare_a = 0;

    let mut red = Pwm::new_output_b(peripherals.PWM_SLICE1, peripherals.PIN_3, configred.clone());
    let mut greenblue = Pwm::new_output_ab(
        peripherals.PWM_SLICE2,
        peripherals.PIN_4,
        peripherals.PIN_5,
        configgreenblue.clone(),
    );
    greenblue.set_config(&configgreenblue);

    let sda = peripherals.PIN_14;
    let scl = peripherals.PIN_15;

    let i2c = i2c::I2c::new_blocking(peripherals.I2C1, scl, sda, Config::default());
    let mut bmp = bmp280_ehal::BMP280::new(i2c).unwrap();
    bmp.reset();
    bmp.set_control(Control {
        osrs_t: Oversampling::x2,
        osrs_p: Oversampling::x2,
        mode: PowerMode::Normal,
    });
    loop {
        info!("{}", bmp.temp());
        Timer::after_millis(1000).await;
    }
}

/// Sets an RGB value on the LED
async fn set_rgb(
    red: &mut Pwm<'_>,
    greenblue: &mut Pwm<'_>,
    red_val: u16,
    green_val: u16,
    blue_val: u16,
) {
    let mut config_red: ConfigPmw = Default::default();
    config_red.top = 0x9088;
    config_red.compare_b = red_val;

    let mut config_greenblue: ConfigPmw = Default::default();
    config_greenblue.top = 0x9088;
    config_greenblue.compare_a = green_val;
    config_greenblue.compare_b = blue_val;

    red.set_config(&config_red);
    greenblue.set_config(&config_greenblue);
    Timer::after_millis(10).await; // Optional delay if needed
}