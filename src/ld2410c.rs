// ld2410c.rs
// This file contains the implementation of the Ld2410C class, which is used to communicate with the LD2410C radar module.
// It includes methods for connecting to the module, reading data, and sending commands to configure the module's settings.
// Fayel MOHAMED
use serde::Serialize;
use std::vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub struct Ld2410C {
    path: String,
    baud_rate: u32,
    stream: Option<SerialStream>,
}
#[derive(Debug, Clone, PartialEq, Serialize)]
enum DataType {
    EngineeringMode = 0x01,
    TargetBasicInformation = 0x02,
    NoDataType,
}
impl DataType {
    fn find_type(data: &[u8]) -> Self {
        match data[0] {
            0x01 => Self::EngineeringMode,
            0x02 => Self::TargetBasicInformation,
            _ => Self::NoDataType, // Default to EngineeringMode if unknown
        }
    }
}
#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct EngineeringModel {
    maximum_mov_distance_gate: u8,
    maximum_static_distance_gate: u8,
    mouvement_distance_gates: Vec<u8>,
    static_distance_gates: Vec<u8>,
    retain_data: Vec<u8>,
}

impl EngineeringModel {
    fn new(data: &[u8]) -> Self {
        Self {
            maximum_mov_distance_gate: data[11],
            maximum_static_distance_gate: data[12],
            mouvement_distance_gates: data[13..data[11] as usize + 14].to_vec(),
            static_distance_gates: data[data[11] as usize + 14..(data[11] as usize * 2) + 15]
                .to_vec(),
            retain_data: data[(data[11] as usize * 2) + 15..data.len() - 2].to_vec(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
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
    fn default() -> Self {
        Self {
            data_type: DataType::NoDataType,
            head: 0x00,
            target_data: TargetData {
                target_status: TargetStatus::NoTarget,
                movement_target_distance: 0,
                exercise_target: 0,
                stationary_target_distance: 0,
                stationary_target: 0,
                detection_distance: 0,
                engineering_model: None,
            },
            tail: 0x00,
            calibration: 0x00,
        }
    }
}

struct Ld2410CFrame {
    frame_header: Vec<u8>,
    intraframe_length: Vec<u8>,
    intraframe_data: Ld2410CCommand,
    end_frame: Vec<u8>,
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
struct Ld2410CCommand {
    word: Vec<u8>,
    value: Vec<u8>,
}
impl Ld2410CCommand {
    fn new(word: Vec<u8>, value: Vec<u8>) -> Self {
        Self { word, value }
    }
}

pub enum GateValue{
    GateValue0,
    GateValue1,
    GateValue2,
    GateValue3,
    GateValue4,
    GateValue5,
    GateValue6,
    GateValue7,
    GateValue8,
    GateValueAll,
}

impl GateValue {
    fn to_vec(&self) -> Vec<u8> {
        match self {
            GateValue::GateValue0 => vec![0x00, 0x00,0x00,0x00],
            GateValue::GateValue1 => vec![0x01, 0x00,0x00,0x00],
            GateValue::GateValue2 => vec![0x02, 0x00,0x00,0x00],
            GateValue::GateValue3 => vec![0x03, 0x00,0x00,0x00],
            GateValue::GateValue4 => vec![0x04, 0x00,0x00,0x00],
            GateValue::GateValue5 => vec![0x05, 0x00,0x00,0x00],
            GateValue::GateValue6 => vec![0x06, 0x00,0x00,0x00],
            GateValue::GateValue7 => vec![0x07, 0x00,0x00,0x00],
            GateValue::GateValue8 => vec![0x08, 0x00,0x00,0x00],
            GateValue::GateValueAll => vec![0xFF, 0xFF,0x00,0x00],
        }
    }
}
struct GateSensitivity {
    distance_gate_word: Vec<u8>,
    distance_gate_value: GateValue,
    motion_sensitivity_word: Vec<u8>,
    motion_sensitivity_value: Vec<u8>,
    standstill_sensitivity_word: Vec<u8>,
    standstill_sensitivity_value: Vec<u8>,
}
impl GateSensitivity {
    fn new(
        distance_gate_value: GateValue,
        motion_sensitivity_value: u8,
        standstill_sensitivity_value: u8,
    ) -> Self {
        Self {
            distance_gate_word: vec![0x00, 0x00],
            distance_gate_value,
            motion_sensitivity_word:vec![0x01, 0x00],
            motion_sensitivity_value: vec![motion_sensitivity_value as u8, 0x00, 0x00, 0x00],
            standstill_sensitivity_word: vec![0x02, 0x00],
            standstill_sensitivity_value: vec![standstill_sensitivity_value as u8, 0x00, 0x00, 0x00],
        }
    }
    fn to_vec(&self) -> Vec<u8> {
        [
            &self.distance_gate_word[..],
            &self.distance_gate_value.to_vec()[..],
            &self.motion_sensitivity_word[..],
            &self.motion_sensitivity_value[..],
            &self.standstill_sensitivity_word[..],
            &self.standstill_sensitivity_value[..],
        ]
        .concat()
    }
    
}

pub enum BaudRate {
    BaudRate115200,
    BaudRate230400,
    BaudRate256000,
    BaudRate460800,
    BaudRate57600,
    BaudRate38400,
    BaudRate19200,
    BaudRate9600,
}
impl BaudRate {
    fn to_vec(&self) -> Vec<u8> {
        match self {
            BaudRate::BaudRate9600 => vec![0x01, 0x00],
            BaudRate::BaudRate19200 => vec![0x02, 0x00],
            BaudRate::BaudRate38400 => vec![0x03, 0x00],
            BaudRate::BaudRate57600 => vec![0x04, 0x00],
            BaudRate::BaudRate115200 => vec![0x05, 0x00],
            BaudRate::BaudRate230400 => vec![0x06, 0x00],
            BaudRate::BaudRate256000 => vec![0x07, 0x00],
            BaudRate::BaudRate460800 => vec![0x08, 0x00],
        }
    }
}
pub enum DistanceResolution {
    DistanceGate0_75m,
    DistanceGate0_2m,
}
impl DistanceResolution {
    fn to_vec(&self) -> Vec<u8> {
        match self {
            DistanceResolution::DistanceGate0_75m => vec![0x00,0x00],
            DistanceResolution::DistanceGate0_2m => vec![0x01,0x00],
        }
    }
}

#[derive(Clone)]
pub enum BluetoothModule {
    TurnOn,
    TurnOff,
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

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        match tokio_serial::new(&self.path, self.baud_rate).open_native_async() {
            Ok(stream) => {
                self.stream = Some(stream);
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
    pub async fn read_data(&mut self) -> anyhow::Result<Ld2410CData> {
        let mut buf = [0u8; 1024];
        self.stream.as_mut().unwrap().read(&mut buf).await?;
        if buf[4] == 0x0D || buf[4] == 0x23 {
            let data = &buf[6..buf[4] as usize + 6];
            let data_type = DataType::find_type(data);
            let target_status = TargetStatus::find_status(data);
            let target_data = TargetData::new(target_status, data);
            let ld2410cdata = Ld2410CData::new(data_type, target_data, data);
            Ok(ld2410cdata)
        }
        else {
            Ok(Ld2410CData::default())
        }
    }

    async fn response_configuration(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut buf = [0u8; 1024];
        match self.stream.as_mut().unwrap().read(&mut buf).await {
            Ok(n) => {
                Ok(buf[..n].to_vec())
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }

    async fn write_data(&mut self, command: &[u8])-> anyhow::Result<Vec<u8>> {
        match self.stream.as_mut().unwrap().write(command).await {
            Ok(_) => {
                Ok(command.to_vec())
            }
            Err(e) => {
                Err(e.into())
            }
        }
    }

    //set_enabling_configuration() Any other commands issued to the radar must be executed
    //after this command is issued, otherwise they are invalid.
    async fn set_enabling_configuration(&mut self)-> anyhow::Result<Vec<u8>> {
        let command = Ld2410CCommand::new(vec![0xFF, 0x00], vec![0x01, 0x00]);
        let data_length = vec![0x04, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        Ok( self.response_configuration().await?)
    }

    // set_ending_configuration() and the radar resumes working mode after execution.
    // If you need to issue other commands again, you need to send the enable configuration
    // command first
    async fn set_ending_configuration(&mut self)-> anyhow::Result<Vec<u8>> {
        let command = Ld2410CCommand::new(vec![0xFE, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        Ok( self.response_configuration().await?)
    }

    // read_firmware_version() This command reads the radar firmware version information.
    pub async fn read_firmware_version(&mut self)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0xA0, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    // set_bluetooth_module() This command sets the Bluetooth module to be turned on or off.
    pub async fn set_bluetooth_module(&mut self, module: BluetoothModule)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0xA4, 0x00], module.to_vec());
        let data_length = vec![0x04, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    // set_bluetooth_password() This command sets the Bluetooth password. The password is a 6-byte string,
    // which is used to connect to the radar module via Bluetooth. The default password is HiLink.
    pub async fn set_bluetooth_password(&mut self, password: String)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        if password.len() != 6 {
            return Err(anyhow::anyhow!("Password must be exactly 6 bytes long"));
        }
        let command = Ld2410CCommand::new(vec![0xA9, 0x00], password.as_bytes().to_vec());
        let data_length = vec![0x08, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    // obtaining_bluetooth_permissions() This command obtains the Bluetooth permissions of the radar module.
    // The password is a 6-byte string, which is used to connect to the radar module via Bluetooth.
    pub async fn obtaining_bluetooth_permissions(&mut self,password: String)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0xA8, 0x00], password.as_bytes().to_vec());
        let data_length = vec![0x08, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }


    // set_engineering_mode() This command opens the radar engineering mode. When the engineering mode is
    // turned on, each distance gate energy value will be added to the radar report data,
    // please refer to 2.3.2 Target Data Composition for detailed format. Engineering mode
    // is off by default after the module is powered on, this configuration value is lost when
    // power is lost.
    pub async fn set_engineering_mode(&mut self)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0x62, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    // set_engineering_mode_off() This command turns off the radar engineering mode.
    // After it is turned off, please refer
    // to 2.3.2 Target Data Composition for the format of radar report data.
    pub async fn set_engineering_mode_off(&mut self)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0x63, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }
    // read_parameter() This command allows you to read the current configuration parameters of the radar.
    pub async fn read_parameter(&mut self)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0x61, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    // reset_module() This command resets the radar module. After the reset, the radar will automatically
    pub async fn set_restart_module(&mut self)-> anyhow::Result<Vec<u8>>{
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0xA3, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    // restore_factory_settings() This command restores all the configuration values to their non-factory
    // values, which take effect after rebooting the module.
    pub async fn restore_factory_settings(&mut self)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0xA2, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    // set_distance_resolution_setting() This command sets the distance resolution of the module, that is how far away each distance gate
    // represents, the configuration value is not lost when power is lost, and the configuration
    // value takes effect after restarting the module.
    // Can be configured to 0.75m or 0.2m per distance gate, the maximum number of
    // distance gates supported are 8.
    pub async fn set_distance_resolution_setting(&mut self, distance_resolution: DistanceResolution)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0xAA, 0x00], distance_resolution.to_vec());
        let data_length = vec![0x04, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    // query_distance_resolutiion_setting() This command queries the module's current distance resolution setting, i.e. how far away each distance
    // gate represents.
    pub async fn query_distance_resolution_setting(&mut self)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0xAB, 0x00], vec![]);
        let data_length = vec![0x02, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    // get_mac_adress() This command reads the MAC address of the radar module.
    pub async fn get_mac_adress(&mut self)-> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0xA5, 0x00], vec![0x01, 0x00]);
        let data_length = vec![0x04, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    //set_serial_port_baud_rate() This command sets the serial port baud rate of the radar module.
    // The default baud rate is 256000, and the baud rate can be set to 460800, 230400, 115200, 57600, 38400, 19200, 9600.
    pub async fn set_serial_port_baud_rate(&mut self, baud_rate: BaudRate) -> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;
        let command = Ld2410CCommand::new(vec![0xA1, 0x00], baud_rate.to_vec());
        let data_length = vec![0x04, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }

    // set_distance_gate_sensitivity_configuration() This command configures the sensitivity of the distance gate, and the configured value
    // is not lost when power is dropped. It supports both configuring each distance gate
    // individually and configuring all distance gates to a uniform value at the same time. If
    // setting all distance gates sensitivity to the same value at the same time, the distance
    // gate value needs to be set to 0xFFFF.
    pub async fn set_distance_gate_sensitivity_configuration(
        &mut self,
        distance_gate: GateValue,
        motion_sensitivity: u8,
        standstill_sensitivity: u8,
    ) -> anyhow::Result<Vec<u8>> {
        self.set_enabling_configuration().await?;

        if motion_sensitivity > 100 || standstill_sensitivity > 100 {
            return Err(anyhow::anyhow!(
                "Sensitivity values must be between 0 and 100"
            ));
        }
        let command_value = GateSensitivity::new(distance_gate, motion_sensitivity, standstill_sensitivity);
        let command = Ld2410CCommand::new(vec![0x64, 0x00], command_value.to_vec());
        let data_length = vec![0x14, 0x00];
        let frame = Ld2410CFrame::new(data_length, command);
        self.write_data(&frame.to_u8()).await?;
        let response = self.response_configuration().await?;
        self.set_ending_configuration().await?;
        Ok(response)
    }
}
