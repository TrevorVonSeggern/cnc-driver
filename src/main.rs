#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod my_clock;

use panic_halt as _;
use arduino_hal::prelude::*;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut led = pins.d13.into_output();

    loop {
        ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").unwrap();
        //let b = nb::block!(serial.read()).unwrap();
        //led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
