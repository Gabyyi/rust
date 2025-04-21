#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{init, gpio};
use gpio::{Level, Output};
use gpio::{Input, Pull};
use embassy_time::{Timer, Duration};
use panic_probe as _;


#[embassy_executor::main]
async fn main(_spawner: Spawner){
    //Exercise 1
    // info!("Device started");

    //Exercise 2
    // let peripherals = embassy_rp::init(Default::default());
    // let mut led = Output::new(peripherals.PIN_2, Level::Low);
    // led.set_high();
    // info!("LED on GPIO pin 2 is set to HIGH");
    
    //Exercise 3
    // let peripherals = embassy_rp::init(Default::default());
    // let mut led = Output::new(peripherals.PIN_2, Level::Low);
    // loop {
    //     led.set_high();
    //     Timer::after(Duration::from_millis(300)).await;
    //     led.set_low();
    //     Timer::after(Duration::from_millis(300)).await;
    // }

    //Exercise 4
    // let peripherals = embassy_rp::init(Default::default());
    // let button = Input::new(peripherals.PIN_2, Pull::Up);
    // loop{
    //     if button.is_low(){
    //         info!("The button was pressed");
    //         Timer::after(Duration::from_millis(300)).await;
    //     }
    // }

    //Exercise 5
    // let peripherals = embassy_rp::init(Default::default());
    // let mut led = Output::new(peripherals.PIN_3, Level::Low);
    // let button = Input::new(peripherals.PIN_2, Pull::Up);
    // loop{
    //     if button.is_low(){
    //         led.toggle();
    //         info!("BUTTON pressed LED on");
    //     }else{
    //         info!("BUTTON pressed LED off");
    //     }
    // }

    //Exercise 6
    // let peripherals = embassy_rp::init(Default::default());
    // let mut led = Output::new(peripherals.PIN_3, Level::Low);
    // let mut button = Input::new(peripherals.PIN_2, Pull::Up);
    // loop {
    //     button.wait_for_falling_edge().await;
    //     info!("The button was pressed");

    //     if led.is_set_high() {
    //         led.set_low();
    //         info!("LED turned off");
    //     } else {
    //         led.set_high();
    //         info!("LED turned on");
    //     }
    //     button.wait_for_rising_edge().await;
    // }


    //Exercise 7
    // let peripherals = embassy_rp::init(Default::default());
    // let mut red=Output::new(peripherals.PIN_2, Level::High);
    // let mut green=Output::new(peripherals.PIN_3, Level::High);
    // let mut yellow=Output::new(peripherals.PIN_4, Level::High);

    // loop{
    //     red.set_low();
    //     green.set_high();
    //     yellow.set_high();
    //     Timer::after_secs(3).await;
    //     red.set_high();
    //     yellow.set_high();
    //     green.set_low();
    //     Timer::after_secs(3).await;
    //     green.set_high();
    //     red.set_high();
    //     yellow.set_low();
    //     Timer::after_secs(1).await;
    // }

    //Exercise 8
    // let peripherals = embassy_rp::init(Default::default());
    // let mut red=Output::new(peripherals.PIN_2, Level::High);
    // let mut green=Output::new(peripherals.PIN_3, Level::High);
    // let mut yellow=Output::new(peripherals.PIN_4, Level::High);
    // let mut blue = Output::new(peripherals.PIN_5, Level::High);
    // let pedestrian_button = Input::new(peripherals.PIN_6, Pull::Up);
    // green.set_low();
    // loop{
    //     if pedestrian_button.is_low(){
    //         green.set_high();
    //         red.set_high();
    //         yellow.set_low();
    //         Timer::after_secs(1).await;
    //         yellow.set_high();
            
    //         red.set_low();
    //         let start = embassy_time::Instant::now();
    //         while embassy_time::Instant::now() - start < Duration::from_secs(5) {
    //             blue.toggle();
    //             Timer::after(Duration::from_millis(500)).await;
    //         }
    //         blue.set_high();
            
    //         red.set_high();
    //         green.set_low();
    //     }
    // }

    //Exercise 9 i
    // let peripherals = embassy_rp::init(Default::default());
    // let mut led1=Output::new(peripherals.PIN_2, Level::High);
    // let mut led2=Output::new(peripherals.PIN_3, Level::High);
    // let mut led3=Output::new(peripherals.PIN_4, Level::High);
    // let mut leds = [led1, led2, led3];
    // loop{
    //     display_morse_code('A', &mut leds).await;
    //     Timer::after_secs(1).await;
    // }
    

    //Exercise 9 ii
    let peripherals = embassy_rp::init(Default::default());
    let mut led1=Output::new(peripherals.PIN_2, Level::High);
    let mut led2=Output::new(peripherals.PIN_3, Level::High);
    let mut led3=Output::new(peripherals.PIN_4, Level::High);
    let mut leds = [led1, led2, led3];
    loop{
        display_text_in_morse_code("Rust", &mut leds).await;
        Timer::after_secs(5).await;
    }

}

async fn display_text_in_morse_code(text: &str, leds: &mut [Output<'_>; 3]) {
    for character in text.chars() {
        display_morse_code(character, leds).await;
        Timer::after(Duration::from_millis(500)).await;
    }
}


async fn display_morse_code(character: char, leds: &mut [Output<'_>; 3]) {
    let morse_code = match character {
        'A' | 'a' => ".-",
        'B' | 'b' => "-...",
        'C' | 'c' => "-.-.",
        'D' | 'd' => "-..",
        'E' | 'e' => ".",
        'F' | 'f' => "..-.",
        'G' | 'g' => "--.",
        'H' | 'h' => "....",
        'I' | 'i' => "..",
        'J' | 'j' => ".---",
        'K' | 'k' => "-.-",
        'L' | 'l' => ".-..",
        'M' | 'm' => "--",
        'N' | 'n' => "-.",
        'O' | 'o' => "---",
        'P' | 'p' => ".--.",
        'Q' | 'q' => "--.-",
        'R' | 'r' => ".-.",
        'S' | 's' => "...",
        'T' | 't' => "-",
        'U' | 'u' => "..-",
        'V' | 'v' => "...-",
        'W' | 'w' => ".--",
        'X' | 'x' => "-..-",
        'Y' | 'y' => "-.--",
        'Z' | 'z' => "--..",
        '1' => ".----",
        '2' => "..---",
        '3' => "...--",
        '4' => "....-",
        '5' => ".....",
        '6' => "-....",
        '7' => "--...",
        '8' => "---..",
        '9' => "----.",
        '0' => "-----",
        _ => "",
    };

    for symbol in morse_code.chars() {
        match symbol {
            '.' => {
                leds[1].set_low();
                Timer::after(Duration::from_millis(200)).await;
                leds[1].set_high();
            }
            '-' => {
                for led in leds.iter_mut() {
                    led.set_low();
                }
                Timer::after(Duration::from_millis(200)).await;
                for led in leds.iter_mut() {
                    led.set_high();
                }
            }
            _ => {}
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}

