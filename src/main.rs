
pub mod ld2410c;
pub mod tf_luna;
pub mod api;
pub mod tof200f;

#[tokio::main]
async fn main() {
    let mut tof200f = tof200f::TOF200F::new("COM7".to_string());
    tof200f.connect().await.unwrap();
    loop {
        tof200f.read_data().await.unwrap();
    }
}

