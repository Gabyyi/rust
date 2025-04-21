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
    let peripherals = embassy_rp::init(Default::default());

    exercise_1();
    //exercise_2();
    //exercise_3();
    //exercise_4();
    //exercise_5();
    //exercise_6();
    //exercise_7();
    //exercise_8();
    //exercise_9i();
    //exercise_9ii();

    loop{}

}

//Exercise 1
fn exercise_1(){
    info!("Device started");
}

//Exercise 2
fn exercise_2() {
    let peripherals = embassy_rp::init(Default::default());
    let mut led = Output::new(peripherals.PIN_2, Level::Low);
    led.set_high();
    info!("LED on GPIO pin 2 is set to HIGH");
}


//Exercise 3
async fn exercise_3(){
    let peripherals = embassy_rp::init(Default::default());
    let mut led = Output::new(peripherals.PIN_2, Level::Low);
    loop{
        led.set_high();
        Timer::after_millis(300).await;
        led.set_low();
        Timer::after_millis(300).await;
    }
}

//Exercise 4
async fn exercise_4(){
    let peripherals = embassy_rp::init(Default::default());
    let button = Input::new(peripherals.PIN_2, Pull::Up);
    loop{
        if button.is_low(){
            info!("The button was pressed");
        }
    }
}

//Exercise 5
async fn exercise_5(){
    let peripherals = embassy_rp::init(Default::default());
    let mut led = Output::new(peripherals.PIN_2, Level::Low);
    let button = Input::new(peripherals.PIN_3, Pull::Up);
    led.set_low();
    loop{
        if button.is_low(){
            led.set_high();
            info!("BUTTON pressed LED on");
        }else{
            led.set_low();
            info!("BUTTON pressed LED off");
        }
    }
}

//Exercise 6
async fn exercise_6(){
    let peripherals = embassy_rp::init(Default::default());
    let mut led = Output::new(peripherals.PIN_2, Level::Low);
    let mut button = Input::new(peripherals.PIN_3, Pull::Up);
    
    loop{
        button.wait_for_falling_edge().await;
        info!("The button was pressed");
        if led.is_set_high() {
            led.set_low();
            info!("BUTTON pressed LED on");
        } else {
            led.set_high();
            info!("BUTTON pressed LED off");
        }
        button.wait_for_rising_edge().await;
        }
}

//Exercise 7
async fn exercise_7(){
    let peripherals = embassy_rp::init(Default::default());
    let mut red=Output::new(peripherals.PIN_2, Level::Low);
    let mut green=Output::new(peripherals.PIN_3, Level::Low);
    let mut yellow=Output::new(peripherals.PIN_4, Level::Low);
    loop{
        red.set_high();
        Timer::after_secs(3).await;
        red.set_low();
        green.set_high();
        Timer::after_secs(3).await;
        green.set_low();
        yellow.set_high();
        Timer::after_secs(1).await;
        yellow.set_low();
    }
}

//Exercise 8
async fn exercise_8(){
    let peripherals = embassy_rp::init(Default::default());
    let mut red=Output::new(peripherals.PIN_2, Level::Low);
    let mut green=Output::new(peripherals.PIN_3, Level::Low);
    let mut yellow=Output::new(peripherals.PIN_4, Level::Low);
    let pedestrian_button = Input::new(peripherals.PIN_5, Pull::Up);
    let mut blue = Output::new(peripherals.PIN_6, Level::Low);
    green.set_high();
    loop{
        if pedestrian_button.is_low(){
            green.set_low();
            yellow.set_high();
            Timer::after_secs(1).await;
            yellow.set_low();
            
            red.set_high();
            for _ in 0..10 {
                blue.toggle();
                Timer::after_millis(500).await;
            }
            blue.set_low();
            
            red.set_low();
            green.set_high();
        }
    }
}

//Exercise 9 i
async fn exercise_9i(){
    let peripherals = embassy_rp::init(Default::default());
    let mut led1=Output::new(peripherals.PIN_2, Level::Low);
    let mut led2=Output::new(peripherals.PIN_3, Level::Low);
    let mut led3=Output::new(peripherals.PIN_4, Level::Low);
    let mut leds = [led1, led2, led3];
    display_morse_code('A', &mut leds).await;
}

//Exercise 9 ii
async fn exercise_9ii(){
    let peripherals = embassy_rp::init(Default::default());
    let mut led1=Output::new(peripherals.PIN_2, Level::Low);
    let mut led2=Output::new(peripherals.PIN_3, Level::Low);
    let mut led3=Output::new(peripherals.PIN_4, Level::Low);
    let mut leds = [led1, led2, led3];
    display_text_in_morse_code("Hello World", &mut leds).await;
}

async fn display_text_in_morse_code(text: &str, leds: &mut [Output<'_>; 3]) {
    for character in text.chars() {
        display_morse_code(character, leds).await;
        Timer::after_millis(500).await; // Gap between characters
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
                leds[1].set_high();
                Timer::after_millis(200).await;
                leds[1].set_low();
            }
            '-' => {
                for led in leds.iter_mut() {
                    led.set_high();
                }
                Timer::after_millis(200).await;
                for led in leds.iter_mut() {
                    led.set_low();
                }
            }
            _ => {}
        }
        Timer::after_millis(200).await; // Gap between symbols
    }
}
