use tokio::io::AsyncReadExt;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub struct Ld2410C {
    path: String,
    baud_rate: u32,
    stream: Option<SerialStream>,
}
impl Ld2410C {
    pub fn new(path: String) -> Self {
        Self {
            path,
            // Set defaut baud rate to 256000
            // This can be changed later using the `set_baud_rate` method
            baud_rate: 256000,
            stream: None,
        }
    }

    pub fn set_baud_rate(&mut self, baud_rate: u32) {
        self.baud_rate = baud_rate;
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match tokio_serial::new(&self.path, self.baud_rate).open_native_async() {
            Ok(stream) => {
                self.stream = Some(stream);
                Ok(())
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn read_data(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 32];
        match self.stream.as_mut().unwrap().read(&mut buf).await {
            Ok(n) => {
                println!("Received: {:3?}", &buf[..n]);
                Ok(buf[..n].to_vec())
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
}