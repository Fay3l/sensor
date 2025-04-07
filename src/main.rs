// use tokio::time::{sleep, Duration};
// use std::time::Instant;

pub mod tf_luna;
pub mod ld2410c;

#[tokio::main]
async fn main() {
    let mut ld2410c = ld2410c::Ld2410C::new("COM7".to_string());
    ld2410c.connect().await.unwrap();
    // ld2410c.set_enabling_configuration().await;
    // ld2410c.set_bluetooth_module(ld2410c::BluetoothModule::TurnOn).await;
    // ld2410c.read_firmware_version().await;
    // ld2410c.set_ending_configuration().await;
    loop {
        ld2410c.read_data().await.unwrap();
    }
}





