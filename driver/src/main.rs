#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod my_clock;
mod stepper;
mod machine;


use micromath::F32Ext;
use panic_halt as _;
use machine::Machine;
use my_clock::{millis, millis_init};

use ufmt_float::uFmt_f32;



#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    millis_init(dp.TC0);
    unsafe { avr_device::interrupt::enable(); }

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    ufmt::uwriteln!(&mut serial, "Startup").unwrap();

    let test_data = [
        (1.121948751, 1.113, 0.008948751058),
        //(3.0, 0.0, 1.732050808),
        //(4.0, 0.0, 2.0),
        //(5.0, 0.0, 2.236067977),
        //(6.0, 0.0, 2.449489743),
    ];
    for (a, b, expected) in test_data {
        let r = b - a;
        ufmt::uwriteln!(&mut serial, "{} {} => {} / {}", uFmt_f32::Two(a), uFmt_f32::Two(b), uFmt_f32::Four(r), uFmt_f32::Four(expected)).unwrap();
    }

    let mut led = pins.d13.into_output();
    let mut machine = Machine::default();
    machine.x.config.steps_per_mm = 1.0;
    machine.x.config.max_acceleration = 0.01;
    machine.x.config.max_feed_rate = f32::MAX;
    machine.x.stats.max_feed_rate = machine.x.config.max_feed_rate;
    machine.x.stats.acceleration = machine.x.config.max_acceleration;
    machine.x.stats.instant_feed_rate = 0.0;
    machine.x.stats.position = 0;

        //let b = nb::block!(serial.read()).unwrap();
    //ufmt::uwriteln!(&mut serial, "a: {}", (machine.x.stats.acceleration * 1000.0) as i32).unwrap();
    let target = 100;
    let mut next_update = None;
    let mut next_v = None;
    loop {
        let now = millis() as u64;
        if next_update.is_none() {
            let step_calc = machine.x.next_step_time();
            machine.x.stats.instant_feed_rate = step_calc.map(|f| f.1).unwrap_or(0.0);
            next_update = step_calc.map(|x| x.0 + now);
            next_v = step_calc.map(|x| x.1);
            ufmt::uwriteln!(&mut serial, "{}: {}ms ({}). v:{}",machine.x.stats.position, now, step_calc.unwrap().0,  (next_v.unwrap() * 1000.0) as i32).unwrap();
        }
        if let Some(next_time) = next_update {
            if next_time <= now {
                machine.x.step(now, next_v.unwrap_or(0.0));
                led.toggle();
                next_update = None;
                next_v = None;
            }
            if target == machine.x.stats.position {
                // reset and ramp up again.
                machine.x.stats.position = 0;
                machine.x.stats.instant_feed_rate = 0.0;
                machine.x.stats.last_update = now;
                next_update = None;
                next_v = None;
                ufmt::uwriteln!(&mut serial, "reset position back to 0.").unwrap();
            }
        }
        arduino_hal::delay_us(100);
    }
}

