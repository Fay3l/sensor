use std::vec;
use serde::Serialize;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};


pub struct Ld2410C {
    path: String,
    baud_rate: u32,
    stream: Option<SerialStream>,
}
#[derive(Debug,Clone, PartialEq,Serialize)]
enum DataType {
    EngineeringMode=0x01,
    TargetBasicInformation=0x02,
}
impl DataType {
    fn find_type(data: &[u8]) -> Self {
        match data[0] {
            0x01 => Self::EngineeringMode,
            0x02 => Self::TargetBasicInformation,
            _ => Self::EngineeringMode, // Default to EngineeringMode if unknown
        }
    }
}
#[derive(Debug,Clone,Serialize)]
enum TargetStatus {
    NoTarget,
    CampaignTarget,
    StationnaryTarget,
    CampaignAndStationaryTarget,
}
impl TargetStatus {
    fn find_status(data: &[u8]) -> Self {
        match data[2] {
            0x00 => Self::NoTarget,
            0x01 => Self::CampaignTarget,
            0x02 => Self::StationnaryTarget,
            0x03 => Self::CampaignAndStationaryTarget,
            _ => Self::NoTarget, // Default to NoTarget if unknown
        }
    }
}
#[derive(Debug,Clone,Serialize)]
#[allow(dead_code)]
struct EngineeringModel {
    maximum_mov_distance_gate:u8,
    maximum_static_distance_gate:u8,
    mouvement_distance_gates:Vec<u8>,
    static_distance_gates:Vec<u8>,
    retain_data:Vec<u8>,
}

impl EngineeringModel {
    fn new(data:&[u8])->Self{
        Self {
            maximum_mov_distance_gate:data[11],
            maximum_static_distance_gate:data[12],
            mouvement_distance_gates:data[13..data[11]as usize+14].to_vec(),
            static_distance_gates:data[data[11]as usize+14..(data[11]as usize*2)+15].to_vec(),
            retain_data:data[(data[11]as usize*2)+15..data.len()-2].to_vec(),
        }
    }
}

#[derive(Debug,Clone,Serialize)]
#[allow(dead_code)]
struct TargetData {
    target_status: TargetStatus,
    movement_target_distance: u16,
    exercise_target: u8,
    stationary_target_distance: u16,
    stationary_target: u8,
    detection_distance: u16,
    engineering_model: Option<EngineeringModel>,
}
impl TargetData {
    fn new(target_status: TargetStatus, data: &[u8]) -> Self {
        Self {
            target_status,
            movement_target_distance: u16::from_be_bytes([data[4], data[3]]),
            exercise_target: data[5],
            stationary_target_distance: u16::from_be_bytes([data[7], data[6]]),
            stationary_target: data[8],
            detection_distance: u16::from_be_bytes([data[10], data[9]]),
            engineering_model: if data[0] == 0x01 {
                Some(EngineeringModel::new(data))
            } else {
                None
            },
        }
    }
}
#[derive(Debug,Clone,Serialize)]
#[allow(dead_code)]
pub struct Ld2410CData {
    data_type: DataType,
    head: u8,
    target_data: TargetData,
    tail: u8,
    calibration: u8,
}

impl Ld2410CData {
    fn new(data_type: DataType, target_data: TargetData, data: &[u8]) -> Self {
        if data_type == DataType::TargetBasicInformation {
            return Self {
                data_type,
                head: data[1],
                target_data,
                tail: data[11],
                calibration: data[12],
            };
        } else {
            return Self {
                data_type,
                head: data[1],
                target_data,
                tail: data[33],
                calibration: data[34],
            };
        }
    }
}

struct Ld2410CFrame {
    frame_header: Vec<u8>,
    intraframe_length: Vec<u8>,
    intraframe_data: Ld2410CCommand,
    end_frame: Vec<u8>,
}

struct Ld2410CCommand {
    word: Vec<u8>,
    value: Vec<u8>,
}
// pub struct  Ld2410CResponse{
//     frame_header:Vec<u8>,
//     intraframe_length:Vec<u8>,
//     intraframe_data:Ld2410CCommand,
//     end_frame:Vec<u8>,
// }

// impl Ld2410CResponse{
//     pub fn new(response:Vec<u8>)-> Self{
//         Self{
//             frame_header:response[0..4].to_vec(),
//             intraframe_length:response[4..6].to_vec(),
//             intraframe_data:Ld2410CCommand::new(response[6..8].to_vec(),response[8..10].to_vec()),
//             end_frame:response[10..14].to_vec(),
//         }
//     }
// }
#[derive(Clone)]
pub enum BluetoothModule {
    TurnOn,
    TurnOff,
}

impl Ld2410CFrame {
    fn new(data_length: Vec<u8>, command: Ld2410CCommand) -> Self {
        Self {
            frame_header: vec![0xFD, 0xFC, 0xFB, 0xFA],
            intraframe_length: data_length,
            intraframe_data: command,
            end_frame: vec![0x04, 0x03, 0x02, 0x01],
        }
    }
    fn to_u8(&self) -> Vec<u8> {
        [
            &self.frame_header[..],
            &self.intraframe_length[..],
            &self.intraframe_data.word[..],
            &self.intraframe_data.value[..],
            &self.end_frame[..],
        ]
        .concat()
    }
}

impl Ld2410CCommand {
    fn new(word: Vec<u8>, value: Vec<u8>) -> Self {
        Self { word, value }
    }
}

impl BluetoothModule {
    fn to_vec(&self) -> Vec<u8> {
        match self {
            BluetoothModule::TurnOn => vec![0x01, 0x00],
            BluetoothModule::TurnOff => vec![0x00, 0x00],
        }
    }
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
    pub async fn read_data(&mut self) -> anyhow::Result<Ld2410CData> {
        let mut buf = [0u8; 1024];
        let res = self.stream.as_mut().unwrap().read(&mut buf).await?;
        if buf[4] == 0x0D || buf[4] == 0x23  {
            println!("Received: {:?}", &buf[6..buf[4] as usize + 6]);
            let data = &buf[6..buf[4] as usize + 6];
            let data_type = DataType::find_type(data);
            let target_status = TargetStatus::find_status(data);
            let target_data = TargetData::new(target_status, data);
            let ld2410cdata = Ld2410CData::new(data_type, target_data, data);
            println!("Data Type: {:?}", ld2410cdata);
            Ok(ld2410cdata)
        } else {
            Err(anyhow::anyhow!("Invalid data received"))
        }
    }

    async fn response_configuration(&mut self) {
        let mut buf = [0u8; 1024];
        match self.stream.as_mut().unwrap().read(&mut buf).await {
            Ok(n) => {
                println!("Received: {:?}", &buf[..n]);
            }
            Err(e) => {
                println!("Error fn read_data_(): {:?}",e);
            }
        }
    }
    async fn write_data(&mut self, command: &[u8]) {
        match self.stream.as_mut().unwrap().write(command).await {
            Ok(_) => {
                println!("Command sent: {:02X?}", command);
            }
            Err(e) => {
                println!("Failed to send command: {:?}", e);
                return;
            }
        }
    }
    async fn set_enabling_configuration(&mut self) {
        let command = Ld2410CCommand::new(vec![0xFF, 0x00], vec![0x01, 0x00]);
        let data_length = vec![0x04, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await;
        self.response_configuration().await;
    }
    async fn set_ending_configuration(&mut self) {
        let command = Ld2410CCommand::new(vec![0xFE, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await;
        self.response_configuration().await;
    }
    pub async fn read_firmware_version(&mut self) {
        self.set_enabling_configuration().await;
        let command = Ld2410CCommand::new(vec![0xA0, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await;
        self.response_configuration().await;
        self.set_ending_configuration().await;
    }
    pub async fn set_bluetooth_module(&mut self, module: BluetoothModule) {
        self.set_enabling_configuration().await;
        let command = Ld2410CCommand::new(vec![0xA4, 0x00], module.to_vec());
        let data_length = vec![0x04, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await;
        self.response_configuration().await;
        self.set_ending_configuration().await;
    }
    pub async fn set_engineering_mode(&mut self) {
        self.set_enabling_configuration().await;
        let command = Ld2410CCommand::new(vec![0x62, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await;
        self.response_configuration().await;
        self.set_ending_configuration().await;
    }
    pub async fn set_engineering_mode_off(&mut self) {
        self.set_enabling_configuration().await;
        let command = Ld2410CCommand::new(vec![0x63, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await;
        self.response_configuration().await;
        self.set_ending_configuration().await;
    }
    pub async fn read_prameter(&mut self) {
        self.set_enabling_configuration().await;
        let command = Ld2410CCommand::new(vec![0x61, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await;
        self.response_configuration().await;
        self.set_ending_configuration().await;
    }
}
