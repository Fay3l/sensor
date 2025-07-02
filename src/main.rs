pub mod api;
pub mod ld2410c;
pub mod rd03d;
pub mod tf_luna;
pub mod tof200f;

#[tokio::main]
async fn main() {
    // let app = api::api("COM7".to_string(),ld2410c::DataType::EngineeringMode).await;
    // let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    //     .await
    //     .unwrap();
    // println!("listening on {}", listener.local_addr().unwrap());
    // axum::serve(listener, app).await.unwrap();
    let mut tof200f = tof200f::TOF200F::new("COM7".to_string());
    if let Err(e) = tof200f.connect().await {
        eprintln!("Error connecting to Tof200F: {}", e);
        return;
    }
    else {
        loop {
            match tof200f.read_data().await {
                Ok(data) => {
                    println!("Distance: {:?} cm", data);
                }
                Err(e) => {
                    eprintln!("Error reading data from Tof200F: {}", e);
                }
                
            }
        }
    }
}
