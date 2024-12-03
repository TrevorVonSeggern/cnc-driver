#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod my_clock;

use my_clock::{millis, millis_init};
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    ufmt::uwriteln!(&mut serial, "Startup").unwrap();
    //arduino_hal::delay_ms(1000);

    millis_init(dp.TC0);

    unsafe {
        avr_device::interrupt::enable();
    }


    //let mut led = pins.d13.into_output();

    loop {
        ufmt::uwriteln!(&mut serial, "Hello from Arduino! {}", millis()).unwrap();
        //let b = nb::block!(serial.read()).unwrap();
        //led.toggle();
        arduino_hal::delay_ms(100);
    }
}
