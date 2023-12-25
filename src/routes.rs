use serde_json::json;
use axum::{
    response::IntoResponse,
    http::StatusCode,
    extract,
    Json,
};
use crate::{search::{get_search_results, get_suggest_results}, options::*, exclusions::get_exclusion_patterns, cache::{redis_get_exclusions, redis_set_exclusions}};

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

pub async fn suggest_data_response(params: extract::Query<QueryParams>) -> impl IntoResponse {
  let mut response = json!({
      "valid": false,
  });
  if params.q.is_some() {
    let options = BraveSearchOptions::new(&params);
    let result_set_data = get_suggest_results(&options).await;
    if let Ok(result_set) = result_set_data {
      response = json!(result_set)
    }
  }
  (StatusCode::OK, Json(response))
}


pub async fn list_exclusion_patterns(params: extract::Query<QueryParams>) -> impl IntoResponse {
  let skip_cache = params.cached.unwrap_or(1) < 1;
  
  let cached_rows = if skip_cache {
    vec![]
  } else {
    redis_get_exclusions()
  };
  let cached = !skip_cache && cached_rows.len() > 0;
  let items = if cached {
    cached_rows
  } else {
    let rows = get_exclusion_patterns();
    if rows.len() > 0 {
      redis_set_exclusions(&rows);
    }
    rows
  };
  let response = json!({"cached": cached, "items": items });
  (StatusCode::OK, Json(response))
}