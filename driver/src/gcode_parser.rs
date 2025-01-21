use core::str::from_utf8_unchecked;
use arrayvec::ArrayVec;
use library::{CanSend, GcodeCommand, CircularBuffer};
use crate::{pins::{write_uart, READER}, write_uart_u8};
use embedded_hal::serial::Read;

const INPUT_BUFFER_SIZE: usize = 200;
const RX_SIZE: usize = 100;
static mut RX_BUFFER: CircularBuffer<u8, RX_SIZE> = CircularBuffer::<u8, RX_SIZE>{data:[0u8; RX_SIZE], begin: 0, length: 0};

#[avr_device::interrupt(atmega2560)]
#[allow(static_mut_refs)]
fn USART0_RX() {
    if let Ok(b) = unsafe{READER.assume_init_mut().read()} {
        unsafe{RX_BUFFER.push(b)};
    }
}

#[allow(unused)]
pub struct Parser<F>
    where 
        F: CanSend<GcodeCommand>,
{
    send: F,
    to_send: Option<GcodeCommand>,
    input_bufer:ArrayVec<u8, INPUT_BUFFER_SIZE>,
}

impl<F> Parser<F>
    where
        F: CanSend<GcodeCommand>,
{
    pub fn new(send: F) -> Self {
        Self {
            send,
            input_bufer: ArrayVec::new(),
            to_send: None,
        }
    }

    #[allow(static_mut_refs)]
    pub fn read_serial(&mut self) {
        if self.input_bufer.remaining_capacity() > 0 && !unsafe{RX_BUFFER.is_empty()} {
            avr_device::interrupt::free(|_| {
                for b in unsafe{RX_BUFFER.consume()} {
                    let r = self.input_bufer.try_push(b);
                    if r.is_err() {
                        break;
                    }
                }
            });
        }
        if self.to_send.is_some() {
            self.to_send = self.send.send(self.to_send.take().unwrap()).map(|_| {
                write_uart("ok\r\n");
                None
            }).unwrap_or_else(|e| Some(e));
        }
    }

    pub fn parse_buffer(&mut self) {
        if self.to_send.is_some() || self.input_bufer.len() == 0 {
            return;
        }
        let nl_index = self.input_bufer.iter().position(|&c| c == b'\n');
        if let Some(nl_index) = nl_index {
            if nl_index > 0 {
                let to_parse = self.input_bufer.split_at(nl_index).0;
                let parsed = library::parse(unsafe{from_utf8_unchecked(to_parse)});
                self.to_send = parsed.map_err(|err| {
                    write_uart("unrecognized command: ");
                    write_uart(&err);
                    write_uart("\n");
                    write_uart_u8(&to_parse);
                }).ok();
            }
            self.input_bufer.drain(0..nl_index+1);
        }
        //if let Ok(parsed) = parsed {
            //ufmt::uwriteln!(&mut serial_writer, "parsed gcode {}.{}", parsed.command_id.major, parsed.command_id.minor).unwrap();
        //}
        //else if let Err(err) = parsed {
            //ufmt::uwriteln!(&mut serial_writer, "Parse error:").unwrap();
            //for &b in err.as_bytes() {
                //let _ = serial_writer.write(b);
            //}
        //}
    }
}
