use serde_json::json;
use axum::{
    response::IntoResponse,
    http::StatusCode,
    extract,
    Json,
};
use serde_with::{skip_serializing_none};
use serde::{Serialize, Deserialize};
use crate::search::get_search_results;

// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryParams {
  q: Option<String>,
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

pub async fn search_data_response(params: extract::Query<QueryParams>) -> impl IntoResponse {
    let mut response = json!({
        "valid": false,
    });
    if let Some(q_string) = params.q.clone() {
        let result_set_data = get_search_results(&q_string).await;
        if let Ok(result_set) = result_set_data {
          response = json!(result_set)
        }
    }
    (StatusCode::OK, Json(response))
}
