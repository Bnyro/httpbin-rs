use std::{collections::HashMap, net::SocketAddr};

use axum::{
    body::Body,
    extract::{rejection::PathRejection, ConnectInfo, OriginalUri, Path},
    http::{Request, StatusCode},
    routing::any,
    Json, Router,
};
use serde::Serialize;

#[derive(Serialize, Debug)]
struct JsonResponse {
    url: String,
    path: String,
    method: String,
    headers: HashMap<String, String>,
    queries: HashMap<String, String>,
    origin: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", any(index))
        .route("/*path", any(index));

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap()
}

async fn index(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    _path: Result<Path<String>, PathRejection>,
    OriginalUri(original_uri): OriginalUri,
    request: Request<Body>,
) -> (StatusCode, axum::extract::Json<JsonResponse>) {
    let queries: HashMap<String, String> = request
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);

    let mut headers: HashMap<String, String> = HashMap::new();
    for (key, value) in request.headers() {
        headers.insert(
            key.to_string(),
            value.to_str().unwrap_or_default().to_string(),
        );
    }

    let response = JsonResponse {
        url: original_uri.to_string(),
        path: request.uri().path().to_owned(),
        method: request.method().to_string(),
        headers,
        queries: queries.clone(),
        origin: addr.to_string(),
    };

    let status_param: u16 = queries
        .get("status")
        .unwrap_or(&String::from("200"))
        .parse()
        .unwrap_or(200);

    let status_code = StatusCode::from_u16(status_param);
    (status_code.unwrap_or(StatusCode::OK), Json(response))
}
