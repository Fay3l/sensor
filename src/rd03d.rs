use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialPortBuilderExt;
use std::f64::consts::PI;
use std::time::Duration;


#[derive(Debug, Clone,Serialize,Deserialize)]
pub struct Target {
    pub x: i16,           // mm
    pub y: i16,           // mm
    pub speed: i16,       // cm/s
    pub pixel_distance: u16, // mm
    pub distance: f64,    // mm
    pub angle: f64,       // degrés
}

impl Target {
    pub fn new(x: i16, y: i16, speed: i16, pixel_distance: u16) -> Self {
        let distance = ((x as f64).powi(2) + (y as f64).powi(2)).sqrt();
        let angle = (x as f64).atan2(y as f64) * 180.0 / PI;
        Self { x, y, speed, pixel_distance, distance, angle }
    }
}


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
    pub targets: Vec<Target>,
    pub buffer: Vec<u8>,
    pub multi_mode: bool,
}

impl RD03D {
    pub const SINGLE_TARGET_CMD: &'static [u8] = &[
        0xFD, 0xFC, 0xFB, 0xFA, 0x02, 0x00, 0x80, 0x00, 0x04, 0x03, 0x02, 0x01,
    ];
    pub const MULTI_TARGET_CMD: &'static [u8] = &[
        0xFD, 0xFC, 0xFB, 0xFA, 0x02, 0x00, 0x90, 0x00, 0x04, 0x03, 0x02, 0x01,
    ];
    pub fn new(path: String) -> Self {
        Self {
            path,
            baud_rate: 256000,
            stream: None,
            targets: Vec::new(),
            buffer: Vec::new(),
            multi_mode: true,
        }
    }


    pub fn set_baud_rate(&mut self, baud_rate: u32) {
        self.baud_rate = baud_rate;
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let stream = tokio_serial::new(&self.path, self.baud_rate).open_native_async()?;
        self.stream = Some(stream);
        tokio::time::sleep(Duration::from_millis(200)).await;
        self.set_multi_mode(self.multi_mode).await?;
        Ok(())
    }

    pub async fn set_multi_mode(&mut self, multi_mode: bool) -> anyhow::Result<()> {
        let cmd = if multi_mode { Self::MULTI_TARGET_CMD } else { Self::SINGLE_TARGET_CMD };
        if let Some(stream) = self.stream.as_mut() {
            stream.write_all(cmd).await?;
            stream.flush().await?;
            tokio::time::sleep(Duration::from_millis(200)).await;
            // Impossible de reset_input_buffer avec tokio_serial, donc on vide le buffer logiciel
            self.buffer.clear();
            self.multi_mode = multi_mode;
        }
        Ok(())
    }

    fn parse_signed16(high: u8, low: u8) -> i16 {
        let raw = ((high as u16) << 8) | (low as u16);
        let sign = if (raw & 0x8000) == 0 { 1 } else { -1 };
        let value = (raw & 0x7FFF) as i16;
        sign * value
    }

    fn decode_frame(data: &[u8]) -> Vec<Target> {
        let mut targets = Vec::new();
        if data.len() < 30 || data[0] != 0xAA || data[1] != 0xFF || data[data.len()-2] != 0x55 || data[data.len()-1] != 0xCC {
            return targets;
        }
        for i in 0..3 {
            let base = 4 + i * 8;
            let x = Self::parse_signed16(data[base+1], data[base]);
            let y = Self::parse_signed16(data[base+3], data[base+2]);
            let speed = Self::parse_signed16(data[base+5], data[base+4]);
            let pixel_distance = (data[base+6] as u16) | ((data[base+7] as u16) << 8);
            targets.push(Target::new(x, y, speed, pixel_distance));
        }
        targets
    }

    fn find_complete_frame(data: &[u8]) -> (Option<Vec<u8>>, &[u8]) {
        // Cherche le début de trame
        let mut start_idx = None;
        for i in 0..data.len().saturating_sub(1) {
            if data[i] == 0xAA && data[i+1] == 0xFF {
                start_idx = Some(i);
                break;
            }
        }
        let start = match start_idx {
            Some(idx) => idx,
            None => return (None, data),
        };
        // Cherche la fin de trame
        for i in (start+2)..data.len().saturating_sub(1) {
            if data[i] == 0x55 && data[i+1] == 0xCC {
                let frame = data[start..=i+1].to_vec();
                let remaining = &data[i+2..];
                return (Some(frame), remaining);
            }
        }
        (None, &data[start..])
    }

    pub async fn update(&mut self) -> anyhow::Result<bool> {
        // Lire les données disponibles
        let mut buf = [0u8; 256];
        if let Some(stream) = self.stream.as_mut() {
            let n = stream.read(&mut buf).await?;
            if n > 0 {
                self.buffer.extend_from_slice(&buf[..n]);
            }
        }
        // Limiter la taille du buffer
        if self.buffer.len() > 300 {
            self.buffer = self.buffer[self.buffer.len()-150..].to_vec();
        }
        // Extraire la dernière trame complète
        let mut latest_frame = None;
        let mut temp_buffer = self.buffer.clone();
        loop {
            let (frame, remaining) = Self::find_complete_frame(&temp_buffer);
            if let Some(f) = frame {
                latest_frame = Some(f);
                temp_buffer = remaining.to_vec();
            } else {
                break;
            }
        }
        // Mettre à jour le buffer
        if let Some(frame) = latest_frame {
            if let Some(pos) = self.buffer.windows(frame.len()).rposition(|w| w == frame.as_slice()) {
                self.buffer = self.buffer[pos+frame.len()..].to_vec();
            }
            let decoded = Self::decode_frame(&frame);
            if !decoded.is_empty() {
                self.targets = decoded;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn get_target(&self, target_number: usize) -> Option<&Target> {
        if target_number >= 1 && target_number <= self.targets.len() {
            self.targets.get(target_number - 1)
        } else {
            None
        }
    }

    pub async fn close(&mut self) -> anyhow::Result<()> {
        if let Some(stream) = self.stream.as_mut() {
            stream.shutdown().await?;
        }
        Ok(())
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
