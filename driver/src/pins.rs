use core::mem::MaybeUninit;
use arduino_hal::{clock::MHz16, hal::{port::{PE0, PE1}, Atmega}, pac::USART0, port::mode::{Input, Output}};
use avr_hal_generic::usart::{UsartReader, UsartWriter};
use embedded_hal::serial::Write;
use library::{StepDir, XYZId};

use crate::my_clock::clock_init;

/*
* Arduino mega ramps 1.4 pinout.
* X Pins
*   step    A0(PF0)
*   dir     A1(PF1)
*   enable  D38(PD7)
* Y Pins
*   step    A6(PF6)
*   dir     A7(PF7)
*   enable  A2(PF2)
* Z Pins
*   step    D46(PL3)
*   dir     D48(PL1)
*   enable  A8(PK0)
* E0 Pins
*   step    D26(PA4)
*   dir     D28(PA6)
*   enable  D24(PA2)
* E1 Pins
*   step    D36(PC1)
*   dir     D34(PC3)
*   enable  D30(PC7)
*/

// on my cnc I use the Z slot for X movement, and X for Z movement. Those are simply swapped.
// Y2 is currently driven from Y, but I would like to switch to use E0 so I can control skew.
pub static mut X_STEP: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PL3>> = MaybeUninit::uninit();
pub static mut X_DIR: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PL1>> = MaybeUninit::uninit();
pub static mut X_ENABLE: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PK0>> = MaybeUninit::uninit();
pub static mut Y_STEP: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PF6>> = MaybeUninit::uninit();
pub static mut Y_DIR: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PF7>> = MaybeUninit::uninit();
pub static mut Y_ENABLE: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PF2>> = MaybeUninit::uninit();
pub static mut E0_STEP: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PA4>> = MaybeUninit::uninit();
pub static mut E0_DIR: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PA6>> = MaybeUninit::uninit();
pub static mut E0_ENABLE: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PA2>> = MaybeUninit::uninit();
pub static mut E1_STEP: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PC1>> = MaybeUninit::uninit();
pub static mut E1_DIR: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PC3>> = MaybeUninit::uninit();
pub static mut E1_ENABLE: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PC7>> = MaybeUninit::uninit();
pub static mut Z_STEP: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PF0>> = MaybeUninit::uninit();
pub static mut Z_DIR: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PF1>> = MaybeUninit::uninit();
pub static mut Z_ENABLE: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PD7>> = MaybeUninit::uninit();

pub static mut LED: MaybeUninit<arduino_hal::port::Pin<Output, arduino_hal::hal::port::PB7>> = MaybeUninit::uninit();

pub static mut WRITER: MaybeUninit<UsartWriter<Atmega, USART0, arduino_hal::port::Pin<Input, PE0>, arduino_hal::port::Pin<Output, PE1>, MHz16>> = MaybeUninit::uninit();
pub static mut READER: MaybeUninit<UsartReader<Atmega, USART0, arduino_hal::port::Pin<Input, PE0>, arduino_hal::port::Pin<Output, PE1>, MHz16>> = MaybeUninit::uninit();

pub fn write_uart(source: &str) {
    write_uart_u8(source.as_bytes());
}
pub fn write_uart_u8(source: &[u8]) {
    #[allow(static_mut_refs)]
    let writer = unsafe{WRITER.assume_init_mut()};
    let mut to_send = source.iter();
    let mut n = to_send.next();
    while let Some(b) = n {
        let _ = writer.write(*b).map(|()| n = to_send.next());
    }
}

pub unsafe fn init_static_pins() {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    clock_init(dp.TC0, dp.TC2);
    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);
    serial.listen(avr_hal_generic::usart::Event::RxComplete);
    let (serial_reader, serial_writer) = serial.split();
    #[allow(static_mut_refs)]
    unsafe {
        WRITER.write(serial_writer);
        READER.write(serial_reader);
        LED.write(pins.d13.into_output());
        X_STEP.write(pins.d46.into_output());
        X_DIR.write(pins.d48.into_output());
        X_ENABLE.write(pins.a8.into_output());
        Y_STEP.write(pins.a6.into_output());
        Y_DIR.write(pins.a7.into_output());
        Y_ENABLE.write(pins.a2.into_output());
        E0_STEP.write(pins.d26.into_output());
        E0_DIR.write(pins.d28.into_output());
        E0_ENABLE.write(pins.d24.into_output());
        E1_STEP.write(pins.d36.into_output());
        E1_DIR.write(pins.d34.into_output());
        E1_ENABLE.write(pins.d30.into_output());
        Z_STEP.write(pins.a0.into_output());
        Z_DIR.write(pins.a1.into_output());
        Z_ENABLE.write(pins.d38.into_output());
    }
}

#[derive(Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum Pin {
    Led,
    XStep,
    YStep,
    ZStep,
    E0Step,
    E1Step,
    XDir,
    YDir,
    ZDir,
    E0Dir,
    E1Dir,
    XEnable,
    YEnable,
    ZEnable,
    E0Enable,
    E1Enable,
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

//pub fn pin_output(pin:Pin) -> bool {
    //match pin {
        //Pin::Led => unsafe { &mut LED.assume_init_mut() }.is_set_high(),
        //Pin::XStep => unsafe { &mut X_STEP.assume_init_mut() }.is_set_high(),
        //Pin::YStep => unsafe { &mut Y_STEP.assume_init_mut() }.is_set_high(),
        //Pin::ZStep => unsafe { &mut Z_STEP.assume_init_mut() }.is_set_high(),
        //Pin::XDir => unsafe { &mut X_DIR.assume_init_mut() }.is_set_high(),
        //Pin::YDir => unsafe { &mut Y_DIR.assume_init_mut() }.is_set_high(),
        //Pin::ZDir => unsafe { &mut Z_DIR.assume_init_mut() }.is_set_high(),
        //Pin::XEnable => unsafe { &mut X_ENABLE.assume_init_mut() }.is_set_high(),
        //Pin::YEnable => unsafe { &mut Y_ENABLE.assume_init_mut() }.is_set_high(),
        //Pin::ZEnable => unsafe { &mut Z_ENABLE.assume_init_mut() }.is_set_high(),
    //}
//}

#[allow(static_mut_refs)]
pub fn pin_write(pin:Pin, action:PinAction) {
    match pin {
        Pin::Led => translate_pin_set(action, unsafe { &mut LED.assume_init_mut() }),
        Pin::XStep => translate_pin_set(action, unsafe { &mut X_STEP.assume_init_mut() }),
        Pin::YStep => translate_pin_set(action, unsafe { &mut Y_STEP.assume_init_mut() }),
        Pin::ZStep => translate_pin_set(action, unsafe { &mut Z_STEP.assume_init_mut() }),
        Pin::E0Step => translate_pin_set(action, unsafe { &mut E0_STEP.assume_init_mut() }),
        Pin::E1Step => translate_pin_set(action, unsafe { &mut E1_STEP.assume_init_mut() }),
        Pin::XDir => translate_pin_set(action, unsafe { &mut X_DIR.assume_init_mut() }),
        Pin::YDir => translate_pin_set(action, unsafe { &mut Y_DIR.assume_init_mut() }),
        Pin::ZDir => translate_pin_set(action, unsafe { &mut Z_DIR.assume_init_mut() }),
        Pin::E0Dir => translate_pin_set(action, unsafe { &mut E0_DIR.assume_init_mut() }),
        Pin::E1Dir => translate_pin_set(action, unsafe { &mut E1_DIR.assume_init_mut() }),
        Pin::XEnable => translate_pin_set(action, unsafe { &mut X_ENABLE.assume_init_mut() }),
        Pin::YEnable => translate_pin_set(action, unsafe { &mut Y_ENABLE.assume_init_mut() }),
        Pin::ZEnable => translate_pin_set(action, unsafe { &mut Z_ENABLE.assume_init_mut() }),
        Pin::E0Enable => translate_pin_set(action, unsafe { &mut E0_ENABLE.assume_init_mut() }),
        Pin::E1Enable => translate_pin_set(action, unsafe { &mut E1_ENABLE.assume_init_mut() }),
    }
}

pub fn step(axis: XYZId) {
    match axis {
        XYZId::X => {pin_write(Pin::XStep, PinAction::Toggle)},
        XYZId::Y => {
            pin_write(Pin::YStep, PinAction::Toggle);
            pin_write(Pin::E0Step, PinAction::Toggle);
        },
        XYZId::Z => {pin_write(Pin::ZStep, PinAction::Toggle)},
    };
}

pub fn direction(axis: XYZId, state: bool) {
    match axis {
        XYZId::X => pin_write(Pin::XDir, state.into()),
        XYZId::Y => {
            pin_write(Pin::YDir, state.into());
            pin_write(Pin::E0Dir, state.into());
        },
        XYZId::Z => pin_write(Pin::ZDir, state.into()),
    };
}
//pub fn pin_output_state(axis: XYZId) -> bool{
    //match axis {
        //XYZId::X => pin_output(Pin::XDir),
        //XYZId::Y => pin_output(Pin::YDir),
        //XYZId::Z => pin_output(Pin::ZDir),
    //}
//}

#[derive(Clone, Copy)]
pub struct DriverStaticStepDir;
impl StepDir for DriverStaticStepDir {
    fn step(&mut self, axis: XYZId) { step(axis) }
    fn dir(&mut self, axis: XYZId, d: bool) { direction(axis, d) }
    //fn output(&self, axis: XYZId) -> bool { pin_output_state(axis) }
}
