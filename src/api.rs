use std::convert::Infallible;

use crate::rd03d;
use askama::Template;
use axum::{response::{sse::{Event, KeepAlive}, IntoResponse, Sse}, routing::get, Json, Router};
use futures::Stream;
use serde::{Deserialize, Serialize};
use futures::stream;
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use tower_http::cors::{Any, CorsLayer};

#[derive(Template,Serialize)]
#[template(path = "rd03d.html")]
struct Rd03dTemplate<'a> {
    targets: &'a [rd03d::Target],
}

pub async fn api() -> Router {
    Router::new()
        .route("/rd03d", get(rd03d_handler))
        .route("/rd03d/sse", get(rd03d_sse_handler))
        .layer(CorsLayer::new().allow_origin(Any))
}

#[axum::debug_handler]
async fn rd03d_json_handler() -> impl IntoResponse {
    let mut rd03d = rd03d::RD03D::new("COM7".to_string());
    if rd03d.connect().await.is_ok() {
        let _ = rd03d.update().await;
        Json(rd03d.targets.clone())
    } else {
        Json(Vec::<rd03d::Target>::new())
    }
}

async fn rd03d_sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let stream = IntervalStream::new(interval).then(|_| async {
        let mut rd03d = rd03d::RD03D::new("COM7".to_string());
        if rd03d.connect().await.is_ok() {
            let _ = rd03d.update().await;
            let data = serde_json::to_string(&rd03d.targets).unwrap_or_else(|_| "[]".to_string());
            Ok(Event::default().data(data))
        } else {
            Ok(Event::default().data("[]"))
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}


async fn rd03d_handler() -> axum::response::Html<String> {
    let mut rd03d = rd03d::RD03D::new("COM7".to_string());
    if let Err(e) = rd03d.connect().await {
        return axum::response::Html(format!("<p>Erreur connexion RD03D: {e}</p>"));
    }
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        let _ = rd03d.update().await;
        let tpl = Rd03dTemplate {
            targets: &rd03d.targets,
        };
        return axum::response::Html(tpl.render().unwrap());
    }
}
