// Possible Values for ms:
// ╔═══════════╦══════════════╦═══════════════════╗
// ║ PRESCALER ║ TIMER_COUNTS ║ Overflow Interval ║
// ╠═══════════╬══════════════╬═══════════════════╣
// ║        64 ║          250 ║              1 ms ║
// ║       256 ║          125 ║              2 ms ║
// ║       256 ║          250 ║              4 ms ║
// ║      1024 ║          125 ║              8 ms ║
// ║      1024 ║          250 ║             16 ms ║
// ╚═══════════╩══════════════╩═══════════════════╝
const PRESCALER: u32 = 64;
const TIMER_COUNTS: u32 = 250;

#[allow(non_camel_case_types)]
type ms = u32;

const MILLIS_INCREMENT: ms = PRESCALER as ms * TIMER_COUNTS as ms / 16000;

static mut MICROS_COUNTER: u32 = 0;
static mut MILLIS_COUNTER: ms = 0;

pub fn clock_init(tc0: arduino_hal::pac::TC0, tc2: arduino_hal::pac::TC2) {
    //arduino_hal::pac::TC Configure the timer for the above interval (in CTC mode)
    tc0.tccr0a.write(|w| w.wgm0().ctc());
    tc0.ocr0a.write(|w| unsafe { w.bits(TIMER_COUNTS as u8) });
    tc0.tccr0b.write(|w| match PRESCALER {
        8 => w.cs0().prescale_8(),
        64 => w.cs0().prescale_64(),
        256 => w.cs0().prescale_256(),
        1024 => w.cs0().prescale_1024(),
        _ => panic!(),
    });
    tc0.timsk0.write(|w| w.ocie0a().set_bit());

    // micros registers
    tc2.tccr2a.write(|w| w.wgm2().ctc());
    tc2.ocr2a.write(|w| unsafe { w.bits(125) });
    tc2.tccr2b.write(|w| w.cs2().direct());
    tc2.timsk2.write(|w| w.ocie2a().set_bit());

    //tc1.tccr1a.write(|w| w.wgm1().bits(1));
    //tc1.ocr1a.write(|w| unsafe { w.bits(125 as u16) });
    //tc1.tccr1b.write(|w| w.cs1().direct());
    //tc1.timsk1.write(|w| w.ocie1a().set_bit());
    reset_time();
}

pub fn reset_time() {
    unsafe{MILLIS_COUNTER = 0};
    unsafe{MICROS_COUNTER = 0;}
}

#[avr_device::interrupt(atmega2560)]
fn TIMER0_COMPA() {
    unsafe{MILLIS_COUNTER+=MILLIS_INCREMENT};
}

#[avr_device::interrupt(atmega2560)]
fn TIMER2_COMPA() {
    unsafe{MICROS_COUNTER+=1};
}

#[allow(unused)]
pub fn millis() -> u32 {
    (unsafe {MILLIS_COUNTER}) as u32
}

//const POST_SCALAR:f32 = 16.0 * 0.9921;
pub fn micros() -> u64 {
    let counter = unsafe {MICROS_COUNTER};
    //(counter as f32 * POST_SCALAR) as u64
    counter as u64 * 1000  / 128
}
