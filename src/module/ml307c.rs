use std::error::Error;
use std::io;
use std::io::Write;

pub struct Controller {
    name: String,
    baud: u32,
    timeout: u64,
    serial: Option<Box<dyn serialport::SerialPort>>,
}
impl Controller {
    pub fn new(name: String, baud: u32, timeout:u64) -> Controller {
        Controller {
            name,
            baud,
            timeout,
            serial: None,
        }
    }

    pub fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let port = serialport::new(self.name.clone(), self.baud)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .open()?;
        self.serial = Some(port);
        Ok(())
    }

    pub fn write(&mut self, data: &str) -> Result<String, Box<dyn Error>> {
        if let Some(ref mut serial) = self.serial {
            serial.write_all(format!("{}\r\n", data).as_bytes())?;

            let mut buf: Vec<u8> = vec![0; 32];
            match serial.read(buf.as_mut_slice()) {
                Ok(size) => {
                    let response = String::from_utf8(buf[..size].to_vec())?;
                    Ok(response)
                }
                Err(e) => Err(Box::new(e)),
            }
        } else {
            Err(Box::from(io::Error::new(
                io::ErrorKind::NotConnected,
                "Serial port not connected",
            )))
        }
    }

    pub fn read(&mut self) -> Result<String, Box<dyn Error>> {
        if let Some(ref mut serial) = self.serial {
            let mut buf: Vec<u8> = vec![0; 512];
            match serial.read(buf.as_mut_slice()) {
                Ok(size) => {
                    let response = String::from_utf8(buf[..size].to_vec())?;
                    Ok(response)
                }
                Err(e) => Err(Box::new(e)),
            }
        } else {
            Err(Box::from(io::Error::new(
                io::ErrorKind::NotConnected,
                "Failed Read Serial Port ",
            )))
        }
    }
}
