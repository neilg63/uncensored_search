use serde_json::json;
use axum::{
    response::IntoResponse,
    http::StatusCode,
    extract,
    Json,
};
use crate::{search::get_search_results, options::*};

// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

pub async fn search_data_response(params: extract::Query<QueryParams>) -> impl IntoResponse {
    let mut response = json!({
        "valid": false,
    });
    if params.q.is_some() {
      let options = BraveSearchOptions::new(&params);
      let result_set_data = get_search_results(&options).await;
      if let Ok(result_set) = result_set_data {
        response = json!(result_set)
      }
    }
    (StatusCode::OK, Json(response))
}
