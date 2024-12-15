use core::mem::MaybeUninit;
use arduino_hal::{clock::MHz16, hal::{port::{PE0, PE1}, Atmega}, pac::USART0, port::mode::{Input, Output}};
use avr_hal_generic::usart::{UsartReader, UsartWriter};
use embedded_hal::serial::Write;

use crate::{my_clock::millis_init, stepper_interrupt::stepper_interrupt_init};

pub static mut LED: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PB7>> = MaybeUninit::uninit();
pub static mut X_STEP: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PA4>> = MaybeUninit::uninit();
pub static mut X_DIR: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PA6>> = MaybeUninit::uninit();
pub static mut X_ENABLE: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PA2>> = MaybeUninit::uninit();
pub static mut Y_STEP: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PF6>> = MaybeUninit::uninit();
pub static mut Y_DIR: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PF7>> = MaybeUninit::uninit();
pub static mut Y_ENABLE: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PF2>> = MaybeUninit::uninit();
pub static mut Z_STEP: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PL3>> = MaybeUninit::uninit();
pub static mut Z_DIR: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PL1>> = MaybeUninit::uninit();
pub static mut Z_ENABLE: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PK0>> = MaybeUninit::uninit();

static mut WRITER: MaybeUninit<UsartWriter<Atmega, USART0, arduino_hal::port::Pin<Input, PE0>, arduino_hal::port::Pin<Output, PE1>, MHz16>> = MaybeUninit::uninit();
static mut READER: MaybeUninit<UsartReader<Atmega, USART0, arduino_hal::port::Pin<Input, PE0>, arduino_hal::port::Pin<Output, PE1>, MHz16>> = MaybeUninit::uninit();

pub fn write_uart(source: &str) {
    let writer = unsafe{WRITER.assume_init_mut()};
    let mut to_send = source.as_bytes().iter();
    let mut n = to_send.next();
    while let Some(b) = n {
        let _ = writer.write(*b).map(|()| n = to_send.next());
    }
}

pub unsafe fn init_static_pins() {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    millis_init(dp.TC0);
    stepper_interrupt_init(dp.TC1);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let (serial_reader, serial_writer) = serial.split();
    unsafe {
        WRITER.write(serial_writer);
        READER.write(serial_reader);
        LED.write(pins.d13.into_output());
        X_STEP.write(pins.d26.into_output());
        X_DIR.write(pins.d28.into_output());
        X_ENABLE.write(pins.d24.into_output());
        Y_STEP.write(pins.a6.into_output());
        Y_DIR.write(pins.a7.into_output());
        Y_ENABLE.write(pins.a2.into_output());
        Z_STEP.write(pins.d46.into_output());
        Z_DIR.write(pins.d48.into_output());
        Z_ENABLE.write(pins.a8.into_output());
    }
}

#[derive(Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum Pin {
    Led,
    XStep,
    YStep,
    ZStep,
    XDir,
    YDir,
    ZDir,
    XEnable,
    YEnable,
    ZEnable,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PinAction {
    Low,
    High,
    Toggle,
}

impl From<bool> for PinAction {
    fn from(value: bool) -> Self {
        match value {
            true => PinAction::High,
            false => PinAction::Low,
        }
    }
}

pub fn translate_pin_set<T: avr_hal_generic::port::PinOps>(action: PinAction, b_pin: &mut avr_hal_generic::port::Pin<arduino_hal::port::mode::Output, T>) {
    match action {
        PinAction::Low => b_pin.set_low(),
        PinAction::High => b_pin.set_high(),
        PinAction::Toggle => b_pin.toggle(),
    }
}

pub fn pin_output(pin:Pin) -> bool {
    match pin {
        Pin::Led => unsafe { &mut LED.assume_init_mut() }.is_set_high(),
        Pin::XStep => unsafe { &mut X_STEP.assume_init_mut() }.is_set_high(),
        Pin::YStep => unsafe { &mut Y_STEP.assume_init_mut() }.is_set_high(),
        Pin::ZStep => unsafe { &mut Z_STEP.assume_init_mut() }.is_set_high(),
        Pin::XDir => unsafe { &mut X_DIR.assume_init_mut() }.is_set_high(),
        Pin::YDir => unsafe { &mut Y_DIR.assume_init_mut() }.is_set_high(),
        Pin::ZDir => unsafe { &mut Z_DIR.assume_init_mut() }.is_set_high(),
        Pin::XEnable => unsafe { &mut X_ENABLE.assume_init_mut() }.is_set_high(),
        Pin::YEnable => unsafe { &mut Y_ENABLE.assume_init_mut() }.is_set_high(),
        Pin::ZEnable => unsafe { &mut Z_ENABLE.assume_init_mut() }.is_set_high(),
    }
}

pub fn pin_write(pin:Pin, action:PinAction) {
    match pin {
        Pin::Led => translate_pin_set(action, unsafe { &mut LED.assume_init_mut() }),
        Pin::XStep => translate_pin_set(action, unsafe { &mut X_STEP.assume_init_mut() }),
        Pin::YStep => translate_pin_set(action, unsafe { &mut Y_STEP.assume_init_mut() }),
        Pin::ZStep => translate_pin_set(action, unsafe { &mut Z_STEP.assume_init_mut() }),
        Pin::XDir => translate_pin_set(action, unsafe { &mut X_DIR.assume_init_mut() }),
        Pin::YDir => translate_pin_set(action, unsafe { &mut Y_DIR.assume_init_mut() }),
        Pin::ZDir => translate_pin_set(action, unsafe { &mut Z_DIR.assume_init_mut() }),
        Pin::XEnable => translate_pin_set(action, unsafe { &mut X_ENABLE.assume_init_mut() }),
        Pin::YEnable => translate_pin_set(action, unsafe { &mut Y_ENABLE.assume_init_mut() }),
        Pin::ZEnable => translate_pin_set(action, unsafe { &mut Z_ENABLE.assume_init_mut() }),
    }
}
