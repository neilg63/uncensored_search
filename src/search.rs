use chrono::Duration;
use reqwest::Error;
use serde_json;
use slug::slugify;


use crate::{models::ResultSet, constants::BRAVE_SEARCH_BASE, cache::{redis_get_results, redis_set_results}};

pub async fn fetch_search_results(q: &str) -> Result<ResultSet, Error> {
  let uri = format!("{}?q={}", BRAVE_SEARCH_BASE, q);
  let api_key = dotenv::var("BRAVE_SEARCH").unwrap_or("".to_string());
  let client = reqwest::Client::new();
  println!("{}", uri);
  let result = client.get(&uri).header("X-Subscription-Token", &api_key).send().await;
  match result {
      Ok(resp) => {
        let result  = resp.json::<serde_json::Value>().await;
        match result {
          Ok(json) => Ok(ResultSet::new(&json)),
          Err(err) => Err(err)
        }
      },
      Err(error) => Err(error)
  }
}

pub async fn get_search_results(q: &str) -> Result<ResultSet, Error> {
  let key = slugify(&["brave", q].join("_"));
  if let Some(result) = redis_get_results(&key, Duration::minutes(1)) {
    Ok(result)
  } else {
    let result_set = fetch_search_results(q).await;
    if let Ok(result) = result_set {
      if result.valid {
        redis_set_results(&key, &result.clone());
      }
      Ok(result)
    } else {
      result_set
    }
  }
}