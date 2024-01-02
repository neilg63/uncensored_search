extern crate regex;
extern crate redis;
extern crate slug;

mod routes;
mod search;
mod models;
mod constants;
mod cache;
mod utils;
mod options;
mod exclusions;

use axum::Router;
use std::net::SocketAddr;
use std::time::Duration;
use axum::{
    http::{header, HeaderValue},
    routing::{get, post},
};
use tower_http::{
    limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
    cors::CorsLayer
};

use routes::*;

fn get_max_timeout_secs() -> u64 {
     // timeout requests after 5 minutes, returning 408 status code
    let max_timeout_val = if let Ok(mt_val) = dotenv::var("MAX_TIMEOUT") { mt_val } else { "300".to_owned() };
    if let Ok(to_secs) = u64::from_str_radix(&max_timeout_val, 10) {
        to_secs
    } else {
        300
    }
}

fn get_port_number() -> u16 {
    let env_port = if let Ok(port_ref) = dotenv::var("PORT") { port_ref } else { "3000".to_owned() };
    if let Ok(p) = u16::from_str_radix(&env_port, 10) {
        p
    } else { 
        3000
    }
}

#[tokio::main]
async fn main() {
    let max_timeout_secs = get_max_timeout_secs();
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/search", get(search_data_response))
        .route("/suggest", get(suggest_data_response))

        .route("/exclusions", get(list_exclusion_patterns))
        .layer(CorsLayer::permissive())
        .layer(TimeoutLayer::new(Duration::from_secs(max_timeout_secs)))
        // don't allow request bodies larger than 1024 bytes, returning 413 status code
        .layer(RequestBodyLimitLayer::new(1024))
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::SERVER,
            HeaderValue::from_static("rust-axum"),
        ));
    let app = app.fallback(handler_404);
    let port = get_port_number();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::debug!("listening on {}", addr);
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

