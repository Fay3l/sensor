
pub mod ld2410c;
pub mod tf_luna;
pub mod api;
pub mod tof200f;
pub mod rd03d;

#[tokio::main]
async fn main() {
    let app = api::api("COM7".to_string()).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    // let mut rd03d = rd03d::RD03D::new("COM7".to_string());
    // match rd03d.connect().await {
    //     Ok(_) => println!("Connected to rd03d"),
    //     Err(e) => println!("Failed to connect to rd03d: {:?}", e),
    // }
    // loop{
    //     if (rd03d.update().await.unwrap()){
    //         let target = rd03d.get_target(1).unwrap();
    //         let target2 = rd03d.get_target(2).unwrap();
    //         let target3 = rd03d.get_target(3).unwrap();
    //         println!("Target 1: {:?}", target);
    //         println!("Target 2: {:?}", target2);
    //         println!("Target 3: {:?}", target3);
    //     }
    //     else {
    //         println!("No targets detected");
    //     }
    // }
}

