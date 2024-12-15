use core::cell;

use crate::MACHINE;

// spec sheet: https://ww1.microchip.com/downloads/en/DeviceDoc/doc7799.pdf
// rust timer block: https://blog.rahix.de/005-avr-hal-millis/

//static X_TARGET: avr_device::interrupt::Mutex<cell::Cell<i32>> = avr_device::interrupt::Mutex::new(cell::Cell::new(0 as i32));
//static Y_TARGET: avr_device::interrupt::Mutex<cell::Cell<i32>> = avr_device::interrupt::Mutex::new(cell::Cell::new(0 as i32));
//static Z_TARGET: avr_device::interrupt::Mutex<cell::Cell<i32>> = avr_device::interrupt::Mutex::new(cell::Cell::new(0 as i32));

static MICROS_COUNTER: avr_device::interrupt::Mutex<cell::Cell<u64>> = avr_device::interrupt::Mutex::new(cell::Cell::new(0 as u64));

pub fn stepper_interrupt_init(treg: arduino_hal::pac::TC1) {
    treg.tccr1a.write(|w| w.wgm1().bits(1));
    treg.ocr1a.write(|w| unsafe { w.bits(125 as u16) });
    treg.tccr1b.write(|w| w.cs1().direct());
    treg.timsk1.write(|w| w.ocie1a().set_bit());

    // Reset the global millisecond counter
    avr_device::interrupt::free(|cs| {
        MICROS_COUNTER.borrow(cs).set(0);
    });
}

#[avr_device::interrupt(atmega2560)]
fn TIMER1_COMPA() {
    let mut counter = 0;
    avr_device::interrupt::free(|cs| {
        let counter_cell = MICROS_COUNTER.borrow(cs);
        counter = counter_cell.get();
        counter_cell.set(counter + 1 as u64);
    });
    if let Some(machine) = unsafe{MACHINE.as_mut()} {
        machine.step_monitor(counter);
    }
}

//const POST_SCALAR:f32 = 0.10/6.3;
pub fn micros() -> u64 {
    let counter = avr_device::interrupt::free(|cs| MICROS_COUNTER.borrow(cs).get());
    counter
}

