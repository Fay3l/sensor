use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    routing::get,
    Router,
};
use futures::Stream;
use std::sync::Arc;
use std::{convert::Infallible, time::Duration};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
pub mod ld2410c;
pub mod tf_luna;

#[tokio::main]
async fn main() {
    let sensor = Arc::new(Mutex::new(ld2410c::Ld2410C::new("COM8".to_string())));
    sensor.lock().await.set_baud_rate(115200);
    sensor.lock().await.connect().await.unwrap();
    // let s = sensor.clone();
    // tokio::spawn(async move{
    //     loop {
    //         let res = s.lock().await.read_data().await.unwrap();
    //         println!("{:?}", res);
    //     }
    // });
    
    sensor.lock().await.set_engineering_mode().await.unwrap();
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app: Router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/sse", get(sse_handler))
        .layer(cors)
        .with_state(sensor.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2000").await.unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn sse_handler(
    State(sensor): State<Arc<Mutex<ld2410c::Ld2410C>>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let sens = Arc::clone(&sensor);
    let stream = async_stream::try_stream! {
        loop {
            let data = {
                let mut sensor = sens.lock().await;
                match sensor.read_data().await {
                    Ok(data) => data, // Données valides
                    Err(e) => {
                        eprintln!("Error reading data: {:?}", e); // Log de l'erreur
                        continue; // Ignorer cette itération et continuer
                    }
                }
            };

            let event = Event::default()
                .event("message")
                .data(serde_json::to_string(&data).unwrap());
            yield event;

            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}
