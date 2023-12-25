use chrono::Duration;
use reqwest::Error;
use serde_json;

use crate::{models::{ResultSet, AutoSuggestResultSet}, constants::{BRAVE_SEARCH_BASE, BRAVE_SUGGEST_BASE, MOJEEK_SEARCH_BASE}, cache::{redis_get_results, redis_set_results, redis_get_suggest_results, redis_set_suggest_results}, options::{BraveSearchOptions, SearchProvider}, utils::build_query_string};

pub async fn fetch_search_results(options: &BraveSearchOptions) -> Result<ResultSet, Error> {
  let uri = [BRAVE_SEARCH_BASE, &build_query_string(&options.to_tuples())].concat();
  let api_key = dotenv::var("BRAVE_SEARCH").unwrap_or("".to_string());
  let client = reqwest::Client::new();
  
  let result = client.get(&uri).header("X-Subscription-Token", &api_key).send().await;
  match result {
      Ok(resp) => {
        let result  = resp.json::<serde_json::Value>().await;
        match result {
          Ok(json) => Ok(ResultSet::new(&json, options)),
          Err(err) => Err(err)
        }
      },
      Err(error) => Err(error)
  }
}

pub async fn fetch_search_results_mojeek(options: &BraveSearchOptions) -> Result<ResultSet, Error> {
  let uri = [MOJEEK_SEARCH_BASE, &build_query_string(&options.to_mojeek_tuples())].concat();
  let client = reqwest::Client::new();
  
  let result = client.get(&uri).send().await;
  
  match result {
      Ok(resp) => {
        let result  = resp.json::<serde_json::Value>().await;
        match result {
          Ok(json) => Ok(ResultSet::new_from_mojeek(&json, options)),
          Err(err) => Err(err)
        }
      },
      Err(error) => Err(error)
  }
}

pub async fn get_search_results(options: &BraveSearchOptions) -> Result<ResultSet, Error> {
  let key = options.to_cache_key(options.mode);
  if let Some(result) = redis_get_results(&key, Duration::minutes(60)) {
    Ok(result)
  } else {
    let result_set = fetch_search_results(options).await;
    if let Ok(mut result) = result_set {
      if options.mode.search_mojeek() {
        let result_set_mojeek = fetch_search_results_mojeek(options).await;
        if let Ok(result2) = result_set_mojeek {
          result.merge_results(result2);
        }
      }
      result.exclude_by_patterns();
      if result.valid {
        redis_set_results(&key, &result.clone());
      }
      Ok(result)
    } else {
      result_set
    }
  }
}

pub async fn fetch_suggest_results(options: &BraveSearchOptions) -> Result<AutoSuggestResultSet, Error> {
  let uri = [BRAVE_SUGGEST_BASE, &build_query_string(&options.to_suggest_tuples())].concat();
  let api_key = dotenv::var("BRAVE_SUGGEST").unwrap_or("".to_string());
  let client = reqwest::Client::new();
  
  let result = client.get(&uri).header("X-Subscription-Token", &api_key).send().await;
  match result {
      Ok(resp) => {
        let result  = resp.json::<serde_json::Value>().await;
        match result {
          Ok(json) => Ok(AutoSuggestResultSet::new(&json, options)),
          Err(err) => Err(err)
        }
      },
      Err(error) => Err(error)
  }
}

pub async fn get_suggest_results(options: &BraveSearchOptions) -> Result<AutoSuggestResultSet, Error> {
  let key = options.to_suggest_cache_key();
  if let Some(result) = redis_get_suggest_results(&key, Duration::minutes(1440)) {
    Ok(result)
  } else {
    let result_set = fetch_suggest_results(options).await;
    if let Ok(result) = result_set {
      if result.valid {
        redis_set_suggest_results(&key, &result.clone());
      }
      Ok(result)
    } else {
      result_set
    }
  }
}