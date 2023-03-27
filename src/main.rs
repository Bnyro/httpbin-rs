use std::{collections::HashMap, net::SocketAddr};

use axum::{
    body::Body,
    extract::{ConnectInfo, OriginalUri},
    http::Request,
    routing::any,
    Json, Router,
};
use serde::Serialize;

#[derive(Serialize)]
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
    let app = Router::new().route("/*path", any(index));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap()
}

async fn index(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    OriginalUri(original_uri): OriginalUri,
    request: Request<Body>,
) -> axum::extract::Json<JsonResponse> {
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
        queries,
        origin: addr.to_string(),
    };
    Json(response)
}
