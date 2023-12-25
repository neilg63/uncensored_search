use std::fs;
use serde::{Serialize, Deserialize};

use crate::cache::{redis_get_exclusions, redis_set_exclusions};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlPattern {
  pattern: String,
  name: String,
}

pub fn get_exclusion_patterns() -> Vec<UrlPattern> {
  let fpath = dotenv::var("PATH_TO_EXCLUDE_PATTERNS").unwrap_or("exclusion_patterns.json".to_owned());
  let mut rows: Vec<UrlPattern> = vec![];
  if let Ok(contents) =  fs::read_to_string(fpath) {    
    if let Ok(data) = serde_json::from_str::<Vec<UrlPattern>>(&contents) {
      rows = data;
    }
  }
  rows
}

pub fn get_exclusion_pattern_strings() -> Vec<String> {
  let cached_rows = redis_get_exclusions();
  let items = if cached_rows.len() > 0 {
    cached_rows
  } else {
    let rows = get_exclusion_patterns();
    if rows.len() > 0 {
      redis_set_exclusions(&rows);
    }
    rows
  };
  items.into_iter().map(|row| row.pattern).collect::<Vec<String>>()
}