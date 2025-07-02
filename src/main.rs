pub mod api;
pub mod ld2410c;
pub mod rd03d;
pub mod tf_luna;
pub mod tof200f;

#[tokio::main]
async fn main() {
    let app = api::api("COM7".to_string(),ld2410c::DataType::EngineeringMode).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    // let mut tf_luna = tf_luna::TfLuna::new("COM7".to_string());
    // if let Err(e) = tf_luna.connect().await {
    //     eprintln!("Erreur connexion TF-Luna: {e}");
    //     return;
    // } else {
    //     loop {
    //         let res = tf_luna.read_data().await.unwrap();
    //         println!("Distance: {:?} cm", res);
    //     }
    // }
}
