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
use std::{convert::Infallible, time::Duration};
use tower_http::cors::{Any, CorsLayer};
use crate::ld2410c;
use std::sync::Arc;
use tokio::sync::Mutex;
pub async fn api() -> Router<Arc<Mutex<ld2410c::Ld2410C>>> {
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

    Router::new()
        .route("/sse", get(sse_handler))
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(sensor)
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
