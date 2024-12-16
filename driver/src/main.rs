#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod my_clock;
mod stepper;
mod machine;
mod gcode_parser;
mod pins;

use arduino_hal::delay_ms;
use my_clock::micros;
use pins::{init_static_pins, pin_output, pin_write, write_uart, Pin, PinAction, READER};
use stepper::{Speed, StepDir};
use core::panic::PanicInfo;
use library::{CommandArgument, GcodeCommand, XYZData, XYZId};
use machine::Machine;

use embedded_hal::serial::Read;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    write_uart("!panic handler!\n");
    let dp = unsafe{arduino_hal::Peripherals::steal()};
    let pins = arduino_hal::pins!(dp);
    let mut led = pins.d13.into_output();
    led.set_high();
    loop {
        for _ in 0..6 {
            led.toggle();
            delay_ms(200);
        }
        for _ in 0..6 {
            led.toggle();
            delay_ms(500);
        }
    }
}

fn step(axis: XYZId) {
    match axis {
        XYZId::X => {pin_write(Pin::XStep, PinAction::Toggle); pin_write(Pin::Led, PinAction::Toggle);},
        XYZId::Y => {pin_write(Pin::YStep, PinAction::Toggle)},
        XYZId::Z => {pin_write(Pin::ZStep, PinAction::Toggle)},
    }
}

fn direction(axis: XYZId, state: bool) {
    match axis {
        XYZId::X => pin_write(Pin::XDir, state.into()),
        XYZId::Y => pin_write(Pin::YDir, state.into()),
        XYZId::Z => pin_write(Pin::ZDir, state.into()),
    }
}
fn pin_output_state(axis: XYZId) -> bool{
    match axis {
        XYZId::X => pin_output(Pin::XDir),
        XYZId::Y => pin_output(Pin::YDir),
        XYZId::Z => pin_output(Pin::ZDir),
    }
}

#[derive(Clone, Copy)]
pub struct DriverStaticStepDir;
impl StepDir for DriverStaticStepDir {
    fn step(&self, axis: XYZId) { step(axis) }
    fn dir(&self, axis: XYZId, d: bool) { direction(axis, d) }
    fn output(&self, axis: XYZId) -> bool { pin_output_state(axis) }
}

static STEPPER_SPEED: Speed<f32> = Speed::<f32>{
    speed: 10000.0,
    acceleration: 2000,
    //speed: 18.0,
    //acceleration: 900,
};
static RESOLUTION:f32 = 1.0;
//static RESOLUTION:f32 = 637.0;

pub static mut HAS_GCODE: Option<GcodeCommand> = None;
pub fn add_gcode_buffer(c: GcodeCommand) -> Result<(), GcodeCommand> {
    let full = unsafe{HAS_GCODE.is_some()};
    if full {
        Err(c)
    }
    else {
        unsafe { HAS_GCODE = Some(c) };
        Ok(())
    }
}
pub fn recieve_gcode() -> Option<GcodeCommand>{
    unsafe{HAS_GCODE.take()}
}

pub static mut MACHINE: Option<Machine<DriverStaticStepDir>> = None;

#[arduino_hal::entry]
fn main() -> ! {
    unsafe{init_static_pins()};

    let mut parse_input = gcode_parser::Parser::new(move || unsafe{READER.assume_init_mut()}.read().or_else(|_| Err(())), add_gcode_buffer);
    unsafe{MACHINE = Some(Machine::new(DriverStaticStepDir{}, XYZData::from_clone(STEPPER_SPEED.clone()), XYZData::from_clone(RESOLUTION.clone())))};

    // command is g0 x100
    let mut parsed = GcodeCommand::default();
    let mut arg = CommandArgument::default();
    arg.value.major = 40000;
    parsed.arguments.push(arg);

    //let mut flipflip = false;
    //let mut next_command = parsed.clone();

    //let _ = add_gcode_buffer(parsed);
    //parse_input.poll_task();

    unsafe { avr_device::interrupt::enable(); }

    //let mut buffer: str_buf::StrBuf<100> = str_buf::StrBuf::new();
    delay_ms(1);
    let machine = unsafe{MACHINE.as_mut().unwrap()};
    let mut counter: u32 = 0;
    loop {
        counter += 1;
        parse_input.read_serial();
        if counter % 100 == 0 {
            parse_input.parse_buffer();
        }
        if counter % 10 == 0 {
            machine.poll_task();
        }
        let axis = match counter % 3 {
            0 => Some(XYZId::X),
            1 => Some(XYZId::Y),
            3 => Some(XYZId::Z),
            _ => None,
        };
        if let Some(axis) = axis {
            machine.step_monitor(micros(), axis);
        }
        //next_command = add_gcode_buffer(next_command).map(|()| {
            //write_uart("next command!\n");
            //let mut command = parsed.clone();
            //flipflip = !flipflip;
            //if flipflip {
                //command.arguments[0].value.major = 0;
            //}
            //command
        //}).unwrap_or_else(|c| c);
    }
}

