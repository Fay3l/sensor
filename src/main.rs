
pub mod ld2410c;
pub mod tf_luna;
pub mod api;
pub mod tof200f;
pub mod rd03d;

#[tokio::main]
async fn main() {
    let mut rd03d = rd03d::RD03D::new("COM7".to_string());
    match rd03d.connect().await {
        Ok(_) => println!("Connected to rd03d"),
        Err(e) => println!("Failed to connect to rd03d: {:?}", e),
    }
    loop{
        rd03d.read_data().await.unwrap();
    }
}

