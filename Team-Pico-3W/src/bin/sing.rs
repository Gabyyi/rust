#![no_main]
#![no_std]

use core::array;
use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::pwm::{Config as ConfigPmw, Pwm};
use embassy_time::Timer;
use fixed::traits::ToFixed;

mod music;
use music::*;
const TEMPO: u64 = 100;
/// A whole note duration in milliseconds.
const WHOLE_NOTE: u64 = 4 * (60_000 / TEMPO);
/// The microcontroller clock frequency
const CLOCK_FREQ: u64 = 150_000_000;
const PWM_DIV: u64 = 64;
/// Game of Thrones Theme
pub const MELODY: [(f64, i16); 62] = [
    (REST, 2),
    (NOTE_D4, 4),
    (NOTE_G4, -4),
    (NOTE_AS4, 8),
    (NOTE_A4, 4),
    (NOTE_G4, 2),
    (NOTE_D5, 4),
    (NOTE_C5, -2),
    (NOTE_A4, -2),
    (NOTE_G4, -4),
    (NOTE_AS4, 8),
    (NOTE_A4, 4),
    (NOTE_F4, 2),
    (NOTE_GS4, 4),
    (NOTE_D4, -1),
    (NOTE_D4, 4),
    (NOTE_G4, -4),
    (NOTE_AS4, 8),
    (NOTE_A4, 4),
    (NOTE_G4, 2),
    (NOTE_D5, 4),
    (NOTE_F5, 2),
    (NOTE_E5, 4),
    (NOTE_DS5, 2),
    (NOTE_B4, 4),
    (NOTE_DS5, -4),
    (NOTE_D5, 8),
    (NOTE_CS5, 4),
    (NOTE_CS4, 2),
    (NOTE_B4, 4),
    (NOTE_G4, -1),
    (NOTE_AS4, 4),
    (NOTE_D5, 2),
    (NOTE_AS4, 4),
    (NOTE_D5, 2),
    (NOTE_AS4, 4),
    (NOTE_DS5, 2),
    (NOTE_D5, 4),
    (NOTE_CS5, 2),
    (NOTE_A4, 4),
    (NOTE_AS4, -4),
    (NOTE_D5, 8),
    (NOTE_CS5, 4),
    (NOTE_CS4, 2),
    (NOTE_D4, 4),
    (NOTE_D5, -1),
    (REST, 4),
    (NOTE_AS4, 4),
    (NOTE_D5, 2),
    (NOTE_AS4, 4),
    (NOTE_D5, 2),
    (NOTE_AS4, 4),
    (NOTE_F5, 2),
    (NOTE_E5, 4),
    (NOTE_DS5, 2),
    (NOTE_B4, 4),
    (NOTE_DS5, -4),
    (NOTE_D5, 8),
    (NOTE_CS5, 4),
    (NOTE_CS4, 2),
    (NOTE_AS4, 4),
    (NOTE_G4, -1),
];
use panic_probe as _;


fn get_note(note_frec: u64) -> u64 {
    ((CLOCK_FREQ / note_frec) / PWM_DIV) as u64
}


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let mut config: ConfigPmw = Default::default();
    config.divider = PWM_DIV.to_fixed();
    config.top = 0x9088;

    let mut pwm = Pwm::new_output_b(peripherals.PWM_SLICE1, peripherals.PIN_3, config.clone());
    let song = Song::new(200);
    loop {
        for note in MELODY {
            if note.0 == REST {
                let note_dur = song.calc_note_duration(note.1) as u64;
                Timer::after_millis((note_dur as f64 * 0.9) as u64).await;
                config.compare_b = 0;
                Timer::after_millis((note_dur as f64 * 0.1) as u64).await;
                continue;
            }

            config.top = (get_note(note.0 as u64)) as u16;
            config.compare_b = config.top / 2;
            info!("{}", note.0);
            pwm.set_config(&config);
            let note_dur = song.calc_note_duration(note.1) as u64;
            Timer::after_millis((note_dur as f64 * 0.9) as u64).await;
            config.compare_b = 0;
            Timer::after_millis((note_dur as f64 * 0.1) as u64).await;
        }
    }
}