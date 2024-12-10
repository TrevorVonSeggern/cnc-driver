#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod my_clock;
mod stepper;
mod machine;
mod gcode_parser;

use arduino_hal::{clock::MHz16, hal::{port::{PE0, PE1}, Atmega}, pac::USART0, port::mode::{Input, Output}};
//use avr_device::interrupt::Mutex;
use avr_hal_generic::{port::Pin, usart::UsartWriter, usart::UsartReader};
use panic_halt as _;
use stepper::{float_to_u32_fraction, Speed};
use core::{cell::RefCell, mem::MaybeUninit};
use library::GcodeCommand;
use machine::Machine;
use my_clock::millis_init;
use embedded_hal::{digital::v2::OutputPin, serial::Read, serial::Write};

static mut WRITER: MaybeUninit<UsartWriter<Atmega, USART0, Pin<Input, PE0>, Pin<Output, PE1>, MHz16>> = MaybeUninit::uninit();
static mut READER: MaybeUninit<UsartReader<Atmega, USART0, Pin<Input, PE0>, Pin<Output, PE1>, MHz16>> = MaybeUninit::uninit();

pub fn write_uart(source: &str) {
    let writer = unsafe{WRITER.assume_init_mut()};
    let mut to_send = source.as_bytes().iter();
    let mut n = to_send.next();
    while let Some(b) = n {
        let _ = writer.write(*b).map(|()| n = to_send.next());
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let (serial_reader, serial_writer) = serial.split();
    unsafe {WRITER.write(serial_writer)};
    unsafe {READER.write(serial_reader)};

    millis_init(dp.TC0);
    unsafe { avr_device::interrupt::enable(); }

    //serial_writer.write(j);

    //let serial_writer = RefCell::new(serial_writer);
    //let write = |source: &str| {
        //let mut writer = serial_writer.borrow_mut();
        //let mut to_send = source.as_bytes().iter();
        //let mut n = to_send.next();
        //while let Some(b) = n {
            //let _ = writer.write(*b).map(|()| n = to_send.next());
        //}
    //};

    write_uart("Startup\n");

    let led = RefCell::new(pins.d13.into_output());
    let x_step = RefCell::new(pins.a0.into_output());
    let x_dir = RefCell::new(pins.a1.into_output());
    let y_step = RefCell::new(pins.a6.into_output());
    let y_dir = RefCell::new(pins.a7.into_output());
    let z_step = RefCell::new(pins.d46.into_output());
    let z_dir = RefCell::new(pins.d48.into_output());

    let has_gcode = RefCell::<Option<GcodeCommand>>::default();
    let send_gcode = |c: GcodeCommand| -> Result<(), GcodeCommand> {
        let full = has_gcode.borrow().as_ref().is_some();
        if full { Err(c) }
        else { has_gcode.replace(Some(c)); Ok(()) }
    };
    let recieve_gcode = || has_gcode.take();
        //ufmt::uwriteln!(&mut serial_writer, "parsed gcode {}.{}", c.command_id.major, c.command_id.minor).unwrap();

    let step_fn = |axis| {
        match axis {
            machine::Axis::X => {led.borrow_mut().toggle(); x_step.borrow_mut().toggle()},
            machine::Axis::Y => y_step.borrow_mut().toggle(),
            machine::Axis::Z => z_step.borrow_mut().toggle(),
        };
    };
    let dir_fn = |axis, state: bool| { 
        let _ = match axis {
            machine::Axis::X => x_dir.borrow_mut().set_state(state.into()),
            machine::Axis::Y => y_dir.borrow_mut().set_state(state.into()),
            machine::Axis::Z => z_dir.borrow_mut().set_state(state.into()),
        };
    };
    let default_stepper_config = (Speed{
        speed: float_to_u32_fraction(100.0),
        acceleration: float_to_u32_fraction(5.0),
        decceleration: float_to_u32_fraction(5.0),
    }, 1);

    let mut parse_input = gcode_parser::Parser::new(move || unsafe{READER.assume_init_mut()}.read().or_else(|_| Err(())), send_gcode);
    let mut machine = Machine::new(recieve_gcode, step_fn, dir_fn, [
        default_stepper_config.clone(), // x
        default_stepper_config.clone(), // y
        default_stepper_config // z
    ]);

    parse_input.poll_task();
    loop {
        machine.poll_task();

        //if next_update.is_none() {
            //let step_calc = machine.x.next_step_time();
            //next_update = step_calc.0.map(|x| x as u64 + now);
            ////ufmt::uwriteln!(&mut serial, "{}: {}ms ({}). v:{}",machine.x.stats.position, now, (next_update.unwrap() - now),  vel as i32).unwrap();
        //}
        //if let Some(next_time) = next_update {
            //if next_time <= now {
                //machine.x.step();
                //led.toggle();
                //next_update = None; // triggers an update
            //}
            //if target == machine.x.stats.position {
                //// reset and ramp up again.
                //machine.x.set_position(0);
                //machine.x.set_target(target);
                ////machine.x.set_target(target).map_err(|e| {
                    ////ufmt::uwriteln!(&mut serial, "error: {}", e).unwrap();
                ////}).unwrap();
                //next_update = None;
                ////ufmt::uwriteln!(&mut serial, "reset position back to 0.").unwrap();
            //}
        //}
        //if now % 10 == 0 {
            ////if let Some(command) = parser.next() {
                ////let _ = command.line_number();
                ////match command.major_number() {
                    ////0 => { ufmt::uwriteln!(&mut serial, "g0 command").unwrap(); },
                    ////_ => {
                        ////ufmt::uwriteln!(&mut serial, "Unknown gcode command").unwrap();
                    ////},
                ////};
            ////}
        //}
        arduino_hal::delay_us(100);
    }
}

