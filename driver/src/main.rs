#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod my_clock;
mod stepper;
mod machine;
mod gcode_parser;
mod pins;

use arduino_hal::delay_ms;
use my_clock::*;
use pins::*;
use stepper::*;
use library::*;
use machine::*;
use core::panic::PanicInfo;

use embedded_hal::serial::Read;

static STEPPER_SPEED: Speed<f32> = Speed::<f32>{
    //speed: 10000.0,
    //acceleration: 2000,
    speed: 18.0,
    acceleration: 90,
};
static RESOLUTION:f32 = 1.0;
//static RESOLUTION:f32 = 637.0;

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

struct PollCounter {
    counter: u8,
    target: u8,
}
impl PollCounter {
    fn new(target: u8) -> Self {
        Self { target, counter: 0 }
    }
    fn poll_check(&mut self) -> Option<u8> {
        let c = self.counter;
        self.counter += 1;
        if c == self.target { Some(c) }
        else { None }
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    unsafe{init_static_pins()};

    let reciever = SplitChannel::new(library::Channel::<GcodeCommand, 3>::default());
    let sender = reciever.create_sender();

    let mut parse_input = gcode_parser::Parser::new(move || unsafe{READER.assume_init_mut()}.read().or_else(|_| Err(())), sender);
    let mut machine = Machine::new(DriverStaticStepDir{}, XYZData::from_clone(STEPPER_SPEED.clone()), XYZData::from_clone(RESOLUTION.clone()));

    // command is g0 x100
    //let mut parsed = GcodeCommand::default();
    //let mut arg = CommandArgument::default();
    //arg.value.major = 100;
    //parsed.arguments.push(arg);
    //let mut flipflip = false;
    //let mut next_command = parsed.clone();
    //let sender2 = reciever.create_sender();

    unsafe { avr_device::interrupt::enable(); }

    //let mut buffer: str_buf::StrBuf<100> = str_buf::StrBuf::new();
    delay_ms(1);
    let mut task_serial = PollCounter::new(5);
    let mut task_parse = PollCounter::new(51);
    let mut task_calc = PollCounter::new(51);
    let mut task_step_counter = 0u8;
    loop {
        if let Some(_) = task_serial.poll_check() {
            parse_input.read_serial();
        }
        if let Some(_) = task_parse.poll_check() {
            parse_input.parse_buffer();
        }
        if let Some(_) = task_calc.poll_check() {
            machine.poll_task(&reciever);
        }
        let tsc = task_step_counter;
        task_step_counter += 1;
        let axis = match tsc {
            1 => Some(XYZId::X),
            2 => Some(XYZId::Y),
            3 => {task_step_counter = 0; Some(XYZId::Z)},
            _ => {task_step_counter = 0; None},
        };
        if let Some(axis) = axis {
            machine.step_monitor(micros(), axis);
        }
        //next_command = sender2.send(next_command).map(|()| {
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

