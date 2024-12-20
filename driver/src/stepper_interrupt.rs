use library::XYZId;

use crate::my_clock::micros;

static mut TEST_COUNTER: u64 = 0;
pub fn stepper_interrupt_init(treg: arduino_hal::pac::TC1) {
    // micros registers
    treg.tccr1a.write(|w| w.wgm1().bits(2));
    treg.ocr1a.write(|w| unsafe { w.bits(u16::MAX) });
    treg.tccr1b.write(|w| w.cs1().direct());
    treg.timsk1.write(|w| w.ocie1a().set_bit());
    unsafe{TEST_COUNTER = 0;}
}

#[avr_device::interrupt(atmega2560)]
fn TIMER1_COMPA() {
    unsafe{TEST_COUNTER+=1};
}

pub fn counter() -> u64 {
    unsafe {TEST_COUNTER}
}
