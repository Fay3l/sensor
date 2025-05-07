use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialPortBuilderExt;

struct RD03DCommand {
    pub command_word: Vec<u8>,
    pub command_value: Vec<u8>,
}
impl RD03DCommand {
    pub fn new(command_word: Vec<u8>, command_value: Vec<u8>) -> Self {
        Self {
            command_word,
            command_value,
        }
    }
}
struct RD03DFrame {
    frame_header: Vec<u8>,
    frame_length: Vec<u8>,
    frame_data: RD03DCommand,
    end_frame: Vec<u8>,
}
impl RD03DFrame {
    fn new(frame_length: Vec<u8>, frame_data: RD03DCommand) -> Self {
        Self {
            frame_header: [0xFD, 0xFC, 0xFB, 0xFA].to_vec(),
            frame_length,
            frame_data,
            end_frame: [0x04, 0x03, 0x02, 0x01].to_vec(),
        }
    }
    fn to_u8(&self) -> Vec<u8> {
        [
            &self.frame_header[..],
            &self.frame_length[..],
            &self.frame_data.command_word[..],
            &self.frame_data.command_value[..],
            &self.end_frame[..],
        ]
        .concat()
    }
}
pub struct RD03D {
    pub path: String,
    pub baud_rate: u32,
    pub stream: Option<tokio_serial::SerialStream>,
}

impl RD03D {
    pub fn new(path: String) -> Self {
        Self {
            path,
            baud_rate: 256000,
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
    pub async fn read_data(&mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; 1024];
        match self.stream.as_mut().unwrap().read(&mut buf).await {
            Ok(n) => {
                println!("{:02x?}", &buf[..n]);
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
    pub async fn write_data(&mut self, data: &[u8]) -> anyhow::Result<()> {
        match self.stream.as_mut().unwrap().write(data).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
    pub async fn open_command_mode(&mut self) -> anyhow::Result<()> {
        let command = RD03DCommand::new(vec![0xFF, 0x00], vec![0x01, 0x00]);
        let frame = RD03DFrame::new(vec![0x04, 0x00], command);
        let data = frame.to_u8();
        self.write_data(&data).await?;
        self.read_data().await?;
        Ok(())
    }
    pub async fn close_command_mode(&mut self) -> anyhow::Result<()> {
        let command = RD03DCommand::new(vec![0xFE, 0x00], vec![0x00, 0x00]);
        let frame = RD03DFrame::new(vec![0x02, 0x00], command);
        let data = frame.to_u8();
        self.write_data(&data).await?;
        self.read_data().await?;
        Ok(())
    }
    pub async fn set_mode(&mut self, mode: u8) -> anyhow::Result<()> {
        let command = RD03DCommand::new(vec![0x12, 0x00], vec![mode, 0x00]);
        let frame = RD03DFrame::new(vec![0x08, 0x00], command);
        let data = frame.to_u8();
        self.write_data(&data).await?;
        self.read_data().await?;
        Ok(())
    }
}
