

use tokio::io::AsyncReadExt;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub struct TOF200F {
    path: String,
    baud_rate: u32,
    stream: Option<SerialStream>,
}
impl TOF200F {
    pub fn new(path: String) -> Self {
        Self {
            path,
            baud_rate: 115200,
            stream: None,
        }
    }

    pub fn set_baud_rate(&mut self, baud_rate: u32) {
        self.baud_rate = baud_rate;
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        match tokio_serial::new(&self.path, self.baud_rate).open_native_async() {
            Ok(stream) => {
                self.stream = Some(stream);
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
    
    pub async fn read_data(&mut self)-> anyhow::Result<()>{
        let mut buf = [0u8; 1024];
        match self.stream.as_mut().unwrap().read(&mut buf).await {
            Ok(n) => {
                println!("Buffer: {:?}", &buf[3..5]);
                println!("Distance: {:?} mm", (&buf[4]+(255*&buf[3])));
                Ok(())
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }
}