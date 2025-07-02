
pub mod ld2410c;
pub mod tf_luna;
pub mod api;
pub mod tof200f;
pub mod rd03d;

#[tokio::main]
async fn main() {
    let app = api::api("COM7".to_string(),ld2410c::DataType::EngineeringMode).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    // let mut ld2410c = Ld2410C::new("COM7".to_string());
    // ld2410c.connect().await.unwrap();
    // loop {
    //     let res = ld2410c.read_data().await.unwrap();    
    //     println!("Received data: {:?}", res);
    // }
}

