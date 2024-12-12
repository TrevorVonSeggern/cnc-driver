use library::GcodeCommand;

pub struct Parser<SR, F>
    where 
        F: Fn(GcodeCommand) -> Result<(), GcodeCommand>,
        SR: FnMut() -> Result<u8, ()>,
{
    send: F,
    serial_read: SR,
}

impl<SR, F> Parser<SR, F>
    where
        F: Fn(GcodeCommand) -> Result<(), GcodeCommand>,
        SR: FnMut() -> Result<u8, ()>
{
    pub fn new(reader: SR, send: F) -> Self {
        Self {
            send,
            serial_read: reader,
        }
    }

    pub fn poll_task(&mut self) {
        let parsed = library::parse("G0 X10 Y30");
        if let Ok(cmd) = parsed {
            let _ = (self.send)(cmd);
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
