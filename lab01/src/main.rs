#![no_std]
#![no_main]
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use rp235x_hal::block::ImageDef;
use core::panic::PanicInfo;
use defmt::{info,error};
use defmt_rtt as _; // global logger

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();
#[entry]
fn main() -> ! {
    info!("Hello, world!");
    panic!("Device has started");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic occurred: {:?}", info);
    loop {}
}

