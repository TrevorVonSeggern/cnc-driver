use core::str::from_utf8_unchecked;
use arrayvec::ArrayVec;
use library::{CanSend, GcodeCommand};
use crate::pins::write_uart;

const INPUT_BUFFER_SIZE: usize = 200;

#[allow(unused)]
pub struct Parser<SR, F>
    where 
        F: CanSend<GcodeCommand>,
        SR: FnMut() -> Result<u8, ()>,
{
    send: F,
    serial_read: SR,
    to_send: Option<GcodeCommand>,
    input_bufer:ArrayVec<u8, INPUT_BUFFER_SIZE>,
}

impl<SR, F> Parser<SR, F>
    where
        F: CanSend<GcodeCommand>,
        SR: FnMut() -> Result<u8, ()>
{
    pub fn new(reader: SR, send: F) -> Self {
        Self {
            send,
            serial_read: reader,
            input_bufer: ArrayVec::new(),
            to_send: None,
        }
    }

    pub fn read_serial(&mut self) {
        if let Ok(read_byte) = (self.serial_read)() {
            self.input_bufer.push(read_byte);
        }
        if self.to_send.is_some() {
            self.to_send = self.send.send(self.to_send.take().unwrap()).map(|_| {
                write_uart("ok\n");
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
            if nl_index > 1 {
                let to_parse = self.input_bufer.split_at(nl_index).0;
                let parsed = library::parse(unsafe{from_utf8_unchecked(to_parse)});
                self.to_send = parsed.map_err(|err| {
                    write_uart("unrecognized command: ");
                    write_uart(&err);
                    write_uart("\n");
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
