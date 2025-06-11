use std::convert::Infallible;

use crate::{ld2410c, rd03d};
use askama::Template;
use axum::{response::{sse::{Event, KeepAlive}, Sse}, routing::get, Router};
use futures::Stream;
use serde::Serialize;
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use tower_http::cors::{Any, CorsLayer};

#[derive(Template,Serialize)]
#[template(path = "rd03d.html")]
struct Rd03dTemplate<'a> {
    targets: &'a [rd03d::Target],
}

#[derive(Template, Serialize)]
#[template(path = "ld2410c.html")]
struct Ld2410cTemplate {
    data: ld2410c::Ld2410CData,
}

pub async fn api() -> Router {
    Router::new()
        .route("/rd03d", get(rd03d_handler))
        .route("/rd03d/sse", get(rd03d_sse_handler))
        .route("/ld2410c", get(ld2410c_handler))
        .route("/ld2410c/sse", get(ld2410c_sse_handler))  
        .layer(CorsLayer::new().allow_origin(Any))
}


async fn rd03d_sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let stream = IntervalStream::new(interval).then(|_| async {
        let mut rd03d = rd03d::RD03D::new("COM7".to_string());
        if rd03d.connect().await.is_ok() {
            if rd03d.update().await.unwrap() {
                let target1 = rd03d.get_target(1);
                let target2 = rd03d.get_target(2);
                let target3 = rd03d.get_target(3);
                if let (Some(t1), Some(t2), Some(t3)) = (target1, target2, target3) {
                    rd03d.targets = vec![t1.clone(), t2.clone(), t3.clone()];
                    let data = serde_json::to_string(&rd03d.targets).unwrap_or_else(|_| "[]".to_string());
                    Ok(Event::default().data(data))
                } else {
                    rd03d.targets.clear();
                    Ok(Event::default().data("[]"))
                }
            } else {
                Ok(Event::default().data("[]"))
            }
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

async fn ld2410c_handler() -> axum::response::Html<String> {
    let mut ld2410c = ld2410c::Ld2410C::new("COM7".to_string());
    if let Err(e) = ld2410c.connect().await {
        return axum::response::Html(format!("<p>Erreur connexion LD2410C: {e}</p>"));
    }
    let data = match ld2410c.read_data().await {
        Ok(d) => d,
        Err(e) => return axum::response::Html(format!("<p>Erreur lecture LD2410C: {e}</p>")),
    };
    let tpl = Ld2410cTemplate { data };
    axum::response::Html(tpl.render().unwrap())
}

// Handler SSE pour /ld2410c/sse
async fn ld2410c_sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = IntervalStream::new(tokio::time::interval(std::time::Duration::from_secs(1)))
        .then(|_| async {
            let mut ld2410c = ld2410c::Ld2410C::new("COM7".to_string());
            let mut data = String::from("{}");
            ld2410c.connect().await.unwrap();
            match ld2410c.read_data().await {
                Ok(d) => data = serde_json::to_string(&d).unwrap_or_else(|_| "{}".to_string()),
                Err(e) => eprintln!("Erreur lecture LD2410C: {e}"),
            }
            Ok::<_, Infallible>(Event::default().data(data))
        });
    Sse::new(stream).keep_alive(KeepAlive::default())
}