use std::convert::Infallible;

use crate::{ld2410c, rd03d, tf_luna};
use askama::Template;
use axum::{
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    routing::get,
    Router,
};
use futures::Stream;
use serde::Serialize;
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use tower_http::cors::{Any, CorsLayer};

#[derive(Template, Serialize)]
#[template(path = "tfluna.html")]
struct TfLunaTemplate {
    pub data: tf_luna::TfLunaData, // Replace TfLunaData with the actual data type returned by TfLuna::read_data()
}

#[derive(Template, Serialize)]
#[template(path = "rd03d.html")]
struct Rd03dTemplate<'a> {
    targets: &'a [rd03d::Target],
}

#[derive(Template, Serialize)]
#[template(path = "ld2410c.html")]
struct Ld2410cTemplate {
    data: ld2410c::Ld2410CData,
}

pub async fn api(port: String, set_data_type_ld2410c: ld2410c::DataType) -> Router {
    let rd03d_port = port.clone();
    let rd03d_sse_port = port.clone();
    let ld2410c_port = port.clone();
    let ld2410c_sse_port = port.clone();
    Router::new()
        .route("/rd03d", get(move || rd03d_handler(rd03d_port.clone())))
        .route(
            "/rd03d/sse",
            get(move || rd03d_sse_handler(rd03d_sse_port.clone())),
        )
        .route(
            "/ld2410c",
            get(move || ld2410c_handler(ld2410c_port.clone())),
        )
        .route(
            "/ld2410c/sse",
            get(move || ld2410c_sse_handler(ld2410c_sse_port.clone(), set_data_type_ld2410c)),
        )
        .route("/tfluna", get({
            let port = port.clone();
            move || tf_luna_handler(port.clone())
        }))
        .route("/tfluna/sse", get({
            let port = port.clone();
            move || tf_luna_sse_handler(port.clone())
        }))
        .layer(CorsLayer::new().allow_origin(Any))
}

async fn rd03d_sse_handler(port: String) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let stream = IntervalStream::new(interval).then(move |_| {
        let port = port.clone();
        async move {
            let mut rd03d = rd03d::RD03D::new(port.clone());
            if rd03d.connect().await.is_ok() {
                if rd03d.update().await.unwrap() {
                    let target1 = rd03d.get_target(1);
                    let target2 = rd03d.get_target(2);
                    let target3 = rd03d.get_target(3);
                    if let (Some(t1), Some(t2), Some(t3)) = (target1, target2, target3) {
                        rd03d.targets = vec![t1.clone(), t2.clone(), t3.clone()];
                        let data = serde_json::to_string(&rd03d.targets)
                            .unwrap_or_else(|_| "[]".to_string());
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
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn rd03d_handler(port: String) -> axum::response::Html<String> {
    let mut rd03d = rd03d::RD03D::new(port);
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

async fn ld2410c_handler(port: String) -> axum::response::Html<String> {
    let mut ld2410c = ld2410c::Ld2410C::new(port);
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
async fn ld2410c_sse_handler(
    port: String,
    data_type: ld2410c::DataType,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let port = port.clone();
    let stream = IntervalStream::new(tokio::time::interval(std::time::Duration::from_secs(1)))
        .then(move |_| {
            let port = port.clone();
            {
                let value = data_type.clone();
                async move {
                    match value {
                        ld2410c::DataType::EngineeringMode => {
                            let mut ld2410c = ld2410c::Ld2410C::new(port);
                            let mut data = String::from("{}");

                            if ld2410c.set_engineering_mode().await.is_ok()
                                && ld2410c.connect().await.is_ok()
                            {
                                match ld2410c.read_data().await {
                                    Ok(d) => {
                                        data = serde_json::to_string(&d)
                                            .unwrap_or_else(|_| "{}".to_string())
                                    }
                                    Err(e) => eprintln!("Erreur lecture LD2410C: {e}"),
                                }
                            }
                            Ok::<_, Infallible>(Event::default().data(data))
                        }
                        ld2410c::DataType::TargetBasicInformation => {
                            let mut ld2410c = ld2410c::Ld2410C::new(port);
                            let mut data = String::from("{}");
                            if ld2410c.connect().await.is_ok() {
                                match ld2410c.read_data().await {
                                    Ok(d) => {
                                        data = serde_json::to_string(&d)
                                            .unwrap_or_else(|_| "{}".to_string())
                                    }
                                    Err(e) => eprintln!("Erreur lecture LD2410C: {e}"),
                                }
                            };
                            Ok::<_, Infallible>(Event::default().data(data))
                        }
                        _ => Ok::<_, Infallible>(Event::default().data("{}")),
                    }
                }
            }
        });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn tf_luna_handler(
    port: String,
) -> axum::response::Html<String> {
    let mut tf_luna = crate::tf_luna::TfLuna::new(port);
    if let Err(e) = tf_luna.connect().await {
        return axum::response::Html(format!("<p>Erreur connexion TF-Luna: {e}</p>"));
    }
    let data = match tf_luna.read_data().await {
        Ok(d) => d,
        Err(e) => return axum::response::Html(format!("<p>Erreur lecture TF-Luna: {e}</p>")),
    };
    let tpl = TfLunaTemplate { data };
    axum::response::Html(tpl.render().unwrap())
}

async fn tf_luna_sse_handler(
    port: String,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let port = port.clone();
    let stream = IntervalStream::new(tokio::time::interval(std::time::Duration::from_secs(1)))
        .then(move |_| {
            let port = port.clone();
            async move {
                let mut tf_luna = crate::tf_luna::TfLuna::new(port);
                if tf_luna.connect().await.is_ok() {
                    match tf_luna.read_data().await {
                        Ok(data) => {
                            Ok(Event::default().data(serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string())))
                        }
                        Err(e) => {
                            eprintln!("Erreur lecture TF-Luna: {e}");
                            Ok(Event::default().data("{}"))
                        }
                    }
                } else {
                    Ok(Event::default().data("{}"))
                }
            }
        });
    Sse::new(stream).keep_alive(KeepAlive::default())
}