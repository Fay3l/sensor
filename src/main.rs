// use tokio::time::{sleep, Duration};
// use std::time::Instant;

use std::sync::{Arc, Mutex};

use axum::{extract::State, routing::get, Router};

pub mod ld2410c;
pub mod tf_luna;

#[tokio::main]
async fn main() {
    let sensor = Arc::new(Mutex::new(ld2410c::Ld2410C::new("COM8".to_string())));
    // Connecter le capteur
    {
        sensor.lock().unwrap().connect().await.unwrap();
        sensor.lock().unwrap().set_engineering_mode().await;
    }
    let app: Router = Router::new()
        .route("/sse", get(sse_handler))
        .with_state(sensor.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2000").await.unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
#[axum::debug_handler]
async fn sse_handler(State(sensor): State<Arc<Mutex<ld2410c::Ld2410C>>>) {
    
    
}
