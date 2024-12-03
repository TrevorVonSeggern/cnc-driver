#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod my_clock;
mod stepper;
mod machine;

use machine::Machine;
use my_clock::{millis, millis_init};
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    millis_init(dp.TC0);
    unsafe { avr_device::interrupt::enable(); }

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    ufmt::uwriteln!(&mut serial, "Startup").unwrap();

    let mut led = pins.d13.into_output();
    let mut machine = Machine::default();
    machine.x.config.steps_per_mm = 1.0;
    machine.x.config.max_acceleration = 0.010;
    machine.x.config.max_feed_rate = 1000.0;
    machine.x.stats.max_feed_rate = machine.x.config.max_feed_rate;
    machine.x.stats.acceleration = machine.x.config.max_acceleration;

        //let b = nb::block!(serial.read()).unwrap();
    let mut now = 0;
    loop {
        let next_x_step_time = machine.x.next_step_time(100).unwrap_or(1000);
        let mut d = (next_x_step_time - now) as u16;
        if d >= 100 {
            d = 100;
        }

        arduino_hal::delay_ms(d as u16 + 1);
        now = millis() as u64;

        led.toggle();
        machine.x.step(now);
        ufmt::uwriteln!(&mut serial, "Hello from Arduino! {}, {}", now, d).unwrap();
    }
}
