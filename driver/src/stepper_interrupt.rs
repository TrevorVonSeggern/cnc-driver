use core::{cell, u64};

// spec sheet: https://ww1.microchip.com/downloads/en/DeviceDoc/doc7799.pdf
// rust timer block: https://blog.rahix.de/005-avr-hal-millis/

// ╔═══════════╦══════════════╦═══════════════════╗
// ║ PRESCALER ║ TIMER_COUNTS ║ Overflow Interval ║
// ╠═══════════╬══════════════╬═══════════════════╣
// ║        64 ║          250 ║              1 ms ║
// ║       256 ║          125 ║              2 ms ║
// ║       256 ║          250 ║              4 ms ║
// ║      1024 ║          125 ║              8 ms ║
// ║      1024 ║          250 ║             16 ms ║
// ╚═══════════╩══════════════╩═══════════════════╝
const PRESCALER: u64 = 64;
const TIMER_COUNTS: u64 = 250;
const MILLIS_INCREMENT: u64 = PRESCALER * TIMER_COUNTS / 16000;

//static X_TARGET: avr_device::interrupt::Mutex<cell::Cell<i32>> = avr_device::interrupt::Mutex::new(cell::Cell::new(0 as i32));
//static Y_TARGET: avr_device::interrupt::Mutex<cell::Cell<i32>> = avr_device::interrupt::Mutex::new(cell::Cell::new(0 as i32));
//static Z_TARGET: avr_device::interrupt::Mutex<cell::Cell<i32>> = avr_device::interrupt::Mutex::new(cell::Cell::new(0 as i32));

static MICROS_COUNTER: avr_device::interrupt::Mutex<cell::Cell<u64>> = avr_device::interrupt::Mutex::new(cell::Cell::new(0 as u64));

pub fn stepper_interrupt_init(treg: arduino_hal::pac::TC1) {
    treg.tccr1a.write(|w| w.wgm1().bits(0b100));
    treg.ocr1a.write(|w| unsafe { w.bits(TIMER_COUNTS as u16) });
    treg.tccr1b.write(|w| match PRESCALER {
        8 => w.cs1().prescale_8(),
        64 => w.cs1().prescale_64(),
        256 => w.cs1().prescale_256(),
        1024 => w.cs1().prescale_1024(),
        _ => panic!(),
    });
    treg.timsk1.write(|w| w.ocie1a().set_bit());

    // Reset the global millisecond counter
    avr_device::interrupt::free(|cs| {
        MICROS_COUNTER.borrow(cs).set(0);
    });
}

#[avr_device::interrupt(atmega2560)]
fn TIMER1_COMPA() {
    avr_device::interrupt::free(|cs| {
        let counter_cell = MICROS_COUNTER.borrow(cs);
        let counter = counter_cell.get();
        counter_cell.set(counter + MILLIS_INCREMENT as u64);
    })
}

pub fn micros() -> u64 {
    avr_device::interrupt::free(|cs| MICROS_COUNTER.borrow(cs).get())
}

