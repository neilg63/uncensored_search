use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::cache::get_timestamp;

pub fn extract_string(value: &Value, key: &str) -> Option<String> {
  if let Some(inner) = value.get(key) {
    if let Some(text)  = inner.as_str() {
      Some(text.to_owned())
    } else {
      None
    }
  } else {
    None
  }
}

pub fn extract_string_or_empty(value: &Value, key: &str) -> String {
  extract_string(value, key).unwrap_or("".to_string())
}

pub fn extract_object_vec(value: &Value, key: &str) -> Vec<Value> {
  if let Some(results_value) = value.get(key) {
    if let Some(rows) = results_value.as_array() {
      rows.to_owned()
    } else {
      vec![]
    }
  } else {
    vec![]
  }
}

pub fn extract_inner_results(json: &Value, key: &str) -> Vec<SearchResult> {
  let mut results: Vec<SearchResult> = Vec::new();
  if let Some(data_map) = json[key].as_object() {
    if let Some(inner) = data_map.get("results") {
      if let Some(rows) = inner.as_array() {
        for row in rows {
          results.push(SearchResult::new(&row));
        }
      }
    }
  }
  results
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
  pub uri: String,
  pub title: String,
  pub summary: String,
  pub date: String,
}

impl  SearchResult {
  pub fn new(json: &Value) -> Self {
    let uri = extract_string_or_empty(&json, "url");
    let title = extract_string_or_empty(&json, "title");
    let summary = extract_string_or_empty(&json, "description");
    let date = extract_string_or_empty(&json, "page_age");
    SearchResult {
      uri,
      title,
      summary,
      date
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSet {
  pub valid: bool,
  pub count: usize,
  pub results: Vec<SearchResult>,
  pub ts: i64,
  pub cached: bool
}

impl  ResultSet {
    
    
  pub fn new(json: &Value) -> Self {
    let is_obj = json.is_object();
    let keys = if is_obj { json.as_object().unwrap().keys().into_iter().map(|k| k.as_str()).collect::<Vec<&str>>() } else { vec![] };
    let valid = keys.contains(&"mixed") && (keys.contains(&"news") || keys.contains(&"web"));
    let mut results: Vec<SearchResult> = extract_inner_results(json, "news");
    let web_results: Vec<SearchResult> = extract_inner_results(json, "web");
    if web_results.len() > 0 {
      for result in web_results {
        results.push(result);
      }
    }
    let count = results.len();
    let ts = get_timestamp();
    ResultSet {
      valid,
      count,
      results,
      ts,
      cached: false
    }
  }
    
  pub fn empty() -> Self {
    ResultSet {
      valid: false,
      count: 0,
      results: Vec::new(),
      ts: 0,
      cached: false
    }
  }

  pub fn retrieved_age(&self) -> i64 {
    get_timestamp() - self.ts
  }

  pub fn set_cached(&mut self) -> Self {
    self.cached = true;
    self.to_owned()
  }

}

