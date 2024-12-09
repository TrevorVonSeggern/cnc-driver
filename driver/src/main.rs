#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod my_clock;
mod stepper;
mod machine;
mod gcode_parser;

use panic_halt as _;
use machine::Machine;
use my_clock::{millis, millis_init};
use stepper::float_to_u32_fraction;
use embedded_hal::serial::{Read, Write};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    millis_init(dp.TC0);
    unsafe { avr_device::interrupt::enable(); }

    let (mut serial_read, mut serial_writer) = arduino_hal::default_serial!(dp, pins, 57600).split();
    ufmt::uwriteln!(&mut serial_writer, "Startup").unwrap();

    let mut led = pins.d13.into_output();
    let mut machine = Machine::new();
    machine.x.config.steps_per_mm = 1.0;
    machine.x.config.max_acceleration = 5.0;
    machine.x.config.max_feed_rate = f32::MAX;
    machine.x.stats.max_feed_rate = float_to_u32_fraction(10.0);
    machine.x.stats.acceleration = float_to_u32_fraction(machine.x.config.max_acceleration);
    machine.x.stats.position = 0;

    let parsed = library::parse("G0 X1 Y2 Z3\n");
    if let Ok(parsed) = parsed {
        ufmt::uwriteln!(&mut serial_writer, "parsed gcode {}.{}", parsed.command_id.major, parsed.command_id.minor).unwrap();
    }
    else if let Err(err) = parsed {
        ufmt::uwriteln!(&mut serial_writer, "Parse error:").unwrap();
        for &b in err.as_bytes() {
            let _ = serial_writer.write(b);
        }
    }

    //ufmt::uwriteln!(&mut serial, "a: {}", (machine.x.stats.acceleration * 1000.0) as i32).unwrap();
    let target = 20;
    //ufmt::uwriteln!(&mut serial, "Pre set target").unwrap();
    machine.x.set_target(target);
    //machine.x.set_target(target).map_err(|e| {
        //ufmt::uwriteln!(&mut serial, "error: {}", e).unwrap();
    //}).unwrap();
    //ufmt::uwriteln!(&mut serial, "Set target").unwrap();
    let mut next_update = None;
    loop {
        let now = millis() as u64;
        if let Ok(b) = serial_read.read() {
            let _ = serial_writer.write(b).unwrap();
            let _ = serial_writer.write(b'\n').unwrap();
            let _ = serial_writer.flush().unwrap();
        }

        if next_update.is_none() {
            let step_calc = machine.x.next_step_time();
            next_update = step_calc.0.map(|x| x as u64 + now);
            //ufmt::uwriteln!(&mut serial, "{}: {}ms ({}). v:{}",machine.x.stats.position, now, (next_update.unwrap() - now),  vel as i32).unwrap();
        }
        if let Some(next_time) = next_update {
            if next_time <= now {
                machine.x.step();
                led.toggle();
                next_update = None; // triggers an update
            }
            if target == machine.x.stats.position {
                // reset and ramp up again.
                machine.x.set_position(0);
                machine.x.set_target(target);
                //machine.x.set_target(target).map_err(|e| {
                    //ufmt::uwriteln!(&mut serial, "error: {}", e).unwrap();
                //}).unwrap();
                next_update = None;
                //ufmt::uwriteln!(&mut serial, "reset position back to 0.").unwrap();
            }
        }
        if now % 10 == 0 {
            //if let Some(command) = parser.next() {
                //let _ = command.line_number();
                //match command.major_number() {
                    //0 => { ufmt::uwriteln!(&mut serial, "g0 command").unwrap(); },
                    //_ => {
                        //ufmt::uwriteln!(&mut serial, "Unknown gcode command").unwrap();
                    //},
                //};
            //}
        }
        arduino_hal::delay_us(100);
    }
}

