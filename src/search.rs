use chrono::Duration;
use reqwest::Error;
use serde_json;


use crate::{models::ResultSet, constants::BRAVE_SEARCH_BASE, cache::{redis_get_results, redis_set_results}, options::BraveSearchOptions, utils::build_query_string};

pub async fn fetch_search_results(options: &BraveSearchOptions) -> Result<ResultSet, Error> {
  let uri = [BRAVE_SEARCH_BASE, &build_query_string(&options.to_tuples())].concat();
  let api_key = dotenv::var("BRAVE_SEARCH").unwrap_or("".to_string());
  let client = reqwest::Client::new();
  
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

pub async fn get_search_results(options: &BraveSearchOptions) -> Result<ResultSet, Error> {
  let key = options.to_cache_key();
  if let Some(result) = redis_get_results(&key, Duration::minutes(60)) {
    Ok(result)
  } else {
    let result_set = fetch_search_results(options).await;
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