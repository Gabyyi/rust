#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::config::Config;
use embassy_rp::pwm::SetDutyCycle;
use embassy_rp::{
    bind_interrupts, gpio, init, peripherals,
    pwm::{Config as PwmConfig, Pwm},
    spi::{self, Spi},
};
use embassy_time::{Duration, Timer};
use gpio::{Input, Pull};
use gpio::{Level, Output};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_rp::i2c::{I2c, InterruptHandler as I2CInterruptHandler, Config as I2cConfig};
use embedded_hal_async::i2c::{Error, I2c as _};
use embassy_rp::peripherals::I2C0;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => I2CInterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let sda = peripherals.PIN_4;
    let scl = peripherals.PIN_5;
    let mut i2c = I2c::new_async(peripherals.I2C0, scl, sda, Irqs, I2cConfig::default());

    const BMP280_ADDR: u8 = 0x76;
    const EEPROM_ADDR: u8 = 0x50;
    const EEPROM_TEMP_ADDR: u16 = 0xACDC;

    let mut eeprom_data = [0u8; 4];
    i2c.write_read(EEPROM_ADDR, &EEPROM_TEMP_ADDR.to_be_bytes(), &mut eeprom_data)
        .await
        .unwrap();
    let last_temp = i32::from_be_bytes(eeprom_data);
    info!("Last logged temperature: {}.{}°C", last_temp / 100, last_temp.abs() % 100);

    let ctrl_meas_value = 0b010_000_11;
    i2c.write(BMP280_ADDR, &[0xF4, ctrl_meas_value]).await.unwrap();
    info!("BMP280 configured.");

    let mut data = [0u8; 6];
    i2c.write_read(BMP280_ADDR, &[0x88], &mut data).await.unwrap();
    let dig_t1: u16 = ((data[1] as u16) << 8) | (data[0] as u16);
    let dig_t2: i16 = ((data[3] as i16) << 8) | (data[2] as i16);
    let dig_t3: i16 = ((data[5] as i16) << 8) | (data[4] as i16);
    info!(
        "Calibration data: dig_t1 = {}, dig_t2 = {}, dig_t3 = {}",
        dig_t1, dig_t2, dig_t3
    );

    loop{
        let mut temp_data = [0u8; 3];
        i2c.write_read(BMP280_ADDR, &[0xFA], &mut temp_data).await.unwrap();

        let temp_msb = temp_data[0] as i32;
        let temp_lsb = temp_data[1] as i32;
        let temp_xlsb = temp_data[2] as i32;
        let raw_temp: i32 = (temp_msb << 12) + (temp_lsb << 4) + (temp_xlsb >> 4);

        let var1 = (((raw_temp >> 3) - ((dig_t1 as i32) << 1)) * (dig_t2 as i32)) >> 11;
        let var2 = (((((raw_temp >> 4) - (dig_t1 as i32)) * ((raw_temp >> 4) - (dig_t1 as i32))) >> 12) * (dig_t3 as i32)) >> 14;
        let t_fine = var1 + var2;

        let actual_temp = (t_fine * 5 + 128) >> 8;

        let mem_buff: [u8; 2] = EEPROM_TEMP_ADDR.to_be_bytes();
        let temp_bytes: [u8; 4] = actual_temp.to_be_bytes();
        let mut tx_buf = [0x00; 6];
        tx_buf[..2].copy_from_slice(&mem_buff);
        tx_buf[2..].copy_from_slice(&temp_bytes);

        i2c.write(EEPROM_ADDR, &tx_buf).await.unwrap();

        info!(
            "Temperature {}.{}°C",
            actual_temp / 100,
            actual_temp.abs() % 100
        );
        Timer::after(Duration::from_secs(1)).await;
    }
}