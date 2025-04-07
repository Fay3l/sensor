use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};


pub struct TfLuna {
    path: String,
    baud_rate: u32,
    stream: Option<SerialStream>,
}

pub enum OutputFormat {
    NineByteCm = 0x01,
    PIX = 0x02,
    NineByteMm = 0x06,
    ThirtyTwoTimestamp = 0x07,
    IdZeroOutput = 0x08,
    EightByteCm = 0x09,
}

pub enum OutputFrequency {
    Freq1Hz = 1,
    Freq2Hz = 2,
    Freq4Hz = 4,
    Freq8Hz = 8,
    Freq16Hz = 16,
    Freq10Hz = 10,
    Freq32Hz = 32,
    Freq64Hz = 64,
    Freq128Hz = 128,
    Freq100Hz = 100,
    Freq250Hz = 250,
}

pub enum OutputMode {
    Frequency =0x03,
    DistanceLimit = 0x04,
    OutputFormat = 0x05,
    BaudRate = 0x06,
}

impl TfLuna {
    pub fn new(path: String) -> Self {
        Self {
            path,
            // Set default baud rate to 115200
            // This can be changed later using the `set_baud_rate` method
            baud_rate: 115200,
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
                println!("Received: {:?}", &buf[..n]);
                Ok(buf[..n].to_vec())
            }
            Err(e) => {
                println!("Error: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
    pub async fn write_data(&mut self, command: &[u8]) {
        match self.stream.as_mut().unwrap().write(&command).await {
            Ok(_) => {
                println!("Command sent: {:02X?}", command);
            }
            Err(e) => {
                println!("Failed to send command: {:?}", e);
                return;
            }
        }
    }

    pub async fn get_version_information(&mut self) {
        let command = [0x5A, 0x04, 0x01, 0x00];
        self.write_data(&command).await;
        let res = self.read_data().await.unwrap();
        if res.len() < 4 {
            println!("Error: Invalid response length");
            return;
        }
        let version = format!("{}.{}.{}", res[5], res[4], res[3]);
        println!("Version information sent. {:?}", version);
    }

    pub async fn set_output_format_setting(&mut self, format: OutputFormat) {
        let command: [u8; 5] = [0x5A, 0x05, 0x05, format as u8, 0x00];
        self.write_data(&command).await;
    }

    pub async fn set_baud_rate_setting(&mut self, baud_rate: u32) {
        let command = [
            0x5A,
            0x08,
            0x06,
            baud_rate.to_be_bytes()[3],
            baud_rate.to_be_bytes()[2],
            baud_rate.to_be_bytes()[1],
            baud_rate.to_be_bytes()[0],
            0x00,
        ];
        self.write_data(&command).await;
    }

    pub async fn set_distance_limit_setting(&mut self, dist_min: u16, dist_max: u16) {
        let command = [
            0x5A,
            0x09,
            0x3A,
            dist_min.to_be_bytes()[1],
            dist_min.to_be_bytes()[0],
            dist_max.to_be_bytes()[1],
            dist_max.to_be_bytes()[0],
            0x00,
            0x00,
        ];
        self.write_data(&command).await;
    }

    pub async fn set_output_frequency(&mut self, freq: OutputFrequency) {
        let bytes_freq = freq as u16;
        let command = [
            0x5A,
            0x06,
            0x3,
            bytes_freq.to_be_bytes()[1],
            bytes_freq.to_be_bytes()[0],
            0x00,
        ];
        self.write_data(&command).await;
    }
    pub async fn get_configuration(&mut self,output_mode: OutputMode) {
        let command = [0x5A, 0x05, 0x3F,output_mode as u8, 0x00];
        self.write_data(&command).await;
        self.read_data().await.unwrap();
    }
}
