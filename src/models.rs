use serde::{Serialize, Deserialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use crate::{cache::get_timestamp, options::{BraveSearchOptions, SearchProvider}, utils::{find_position_in_strings, uri_is_excluded}, exclusions::get_exclusion_pattern_strings};

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

pub fn extract_inner_results(json: &Value, key: &str, offset: usize) -> Vec<SearchResult> {
  let mut results: Vec<SearchResult> = Vec::new();
  if let Some(data_map) = json[key].as_object() {
    if let Some(inner) = data_map.get("results") {
      if let Some(rows) = inner.as_array() {
        let mut index = offset;
        for row in rows {
          results.push(SearchResult::new(&row, index));
          index += 1;
        }
      }
    }
  }
  results
}

pub fn extract_mojeek_results(json: &Value, key: &str, offset: usize) -> Vec<SearchResult> {
  let mut results: Vec<SearchResult> = Vec::new();
  if let Some(rows) = json[key].as_array() {
    let mut index = offset;
    for row in rows {
      results.push(SearchResult::new_from_mojeek(&row, index));
      index += 1;
    }
  }
  results
}

pub fn extract_suggest_results(json: &Value) -> Vec<String> {
  let mut results: Vec<String> = Vec::new();
  if let Some(rows) = json["results"].as_array() {
    for row in rows {
      if let Some(item) = row.as_object() {
        if let Some(query_field) = item.get("query") {
          if let Some(text) = query_field.as_str() {
            results.push(text.to_owned());
          }
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
  pub provider: SearchProvider,
  pub weight: usize
}

impl  SearchResult {
  pub fn new(json: &Value, weight: usize) -> Self {
    let uri = extract_string_or_empty(&json, "url");
    let title = extract_string_or_empty(&json, "title");
    let summary = extract_string_or_empty(&json, "description");
    let date = extract_string_or_empty(&json, "page_age");
    SearchResult {
      uri,
      title,
      summary,
      date,
      provider: SearchProvider::Brave,
      weight: weight * 7, // greater weight, lower ranking
    }
  }

  pub fn new_from_mojeek(json: &Value, weight: usize) -> Self {
    let uri = extract_string_or_empty(&json, "url");
    let title = extract_string_or_empty(&json, "title");
    let summary = extract_string_or_empty(&json, "desc");
    let date = chrono::Utc::now().to_rfc3339();
    SearchResult {
      uri,
      title,
      summary,
      date,
      provider: SearchProvider::Mojeek,
      weight: weight * 4 // less weight, higher ranking
    }
  }

  pub fn subtract_weight(&mut self, value: usize) {
    if value < self.weight {
      self.weight -= value;
    } else {
      self.weight = 0;
    }
  }

}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSet {
  pub valid: bool,
  pub count: usize,
  pub results: Vec<SearchResult>,
  pub ts: i64,
  pub lang: Option<String>,
  pub cc: Option<String>,
  pub page: u16,
  pub removed: usize,
  pub cached: bool
}

impl  ResultSet {
    
    
  pub fn new(json: &Value, options: &BraveSearchOptions) -> Self {
    let is_obj = json.is_object();
    let keys = if is_obj { json.as_object().unwrap().keys().into_iter().map(|k| k.as_str()).collect::<Vec<&str>>() } else { vec![] };
    let valid = keys.contains(&"mixed") && (keys.contains(&"news") || keys.contains(&"web"));
    let offset = options.offset.unwrap_or(0) as usize;
    let mut results: Vec<SearchResult> = extract_inner_results(json, "news", offset);
    let web_results: Vec<SearchResult> = extract_inner_results(json, "web", offset + results.len());
    if web_results.len() > 0 {
      for result in web_results {
        results.push(result);
      }
    }
    let count = results.len();
    let ts = get_timestamp();
    let page = options.page();
    let cc = options.country_code();
    let lang = options.lang();
    ResultSet {
      valid,
      count,
      results,
      ts,
      page,
      cc,
      lang,
      removed: 0,
      cached: false
    }
  }

  pub fn new_from_mojeek(json: &Value, options: &BraveSearchOptions) -> Self {
    let is_obj = json.is_object();
    let keys = if is_obj { json.as_object().unwrap().keys().into_iter().map(|k| k.as_str()).collect::<Vec<&str>>() } else { vec![] };
    let valid = false;
    let offset = options.offset.unwrap_or(0) as usize;
    if keys.contains(&"response") {
      if let Some(_data_map) = json["response"].as_object() {
        let results: Vec<SearchResult> = extract_mojeek_results(&json["response"], "results", offset);
        let count = results.len();
        let ts = get_timestamp();
        let page = options.page();
        let cc = options.country_code();
        let lang = options.lang();
        ResultSet {
          valid,
          count,
          results,
          ts,
          page,
          cc,
          lang,
          removed: 0,
          cached: false
        }
      } else {
        ResultSet::empty()  
      }
    } else {
      ResultSet::empty()
    }
    
  }
    
  pub fn empty() -> Self {
    ResultSet {
      valid: false,
      count: 0,
      results: Vec::new(),
      ts: 0,
      cached: false,
      lang: None,
      cc: None,
      removed: 0,
      page: 0
    }
  }

  pub fn retrieved_age(&self) -> i64 {
    get_timestamp() - self.ts
  }

  pub fn set_cached(&mut self) -> Self {
    self.cached = true;
    self.to_owned()
  }

  pub fn merge_results(&mut self, other_set: ResultSet) {
    self.ts = get_timestamp();
    let current_uris = self.results.clone().into_iter().map(|row| row.uri).collect::<Vec<String>>();
    for row in other_set.results {
      if let Some(current_index) = find_position_in_strings(&current_uris, &row.uri) {
        if let Some(current_row) = self.results.get_mut(current_index.to_owned()) {
          current_row.subtract_weight(row.weight);
        }
      } else {
        self.results.push(row);
      }
    }
    self.results.sort_by(|a,b| a.weight.cmp(&b.weight));
    self.count = self.results.len();
  }

  pub fn exclude_by_patterns(&mut self) {
    let full_count = self.count;
    let pattern_strings = get_exclusion_pattern_strings();
    self.results = self.results.clone().into_iter().filter(|row| !uri_is_excluded(&pattern_strings, &row.uri)).collect();
    self.count = self.results.len();
    self.removed = full_count - self.count
  }

}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSuggestResultSet {
  pub valid: bool,
  pub count: usize,
  pub results: Vec<String>,
  pub ts: i64,
  pub lang: Option<String>,
  pub cc: Option<String>,
  pub cached: bool
}

impl  AutoSuggestResultSet {
    
    
  pub fn new(json: &Value, options: &BraveSearchOptions) -> Self {
    let is_obj = json.is_object();
    let keys = if is_obj { json.as_object().unwrap().keys().into_iter().map(|k| k.as_str()).collect::<Vec<&str>>() } else { vec![] };
    let valid = keys.contains(&"results");
    let results: Vec<String> = extract_suggest_results(json);

    let count = results.len();
    let ts = get_timestamp();
    let cc = options.country_code();
    let lang = options.lang();
    AutoSuggestResultSet {
      valid,
      count,
      results,
      ts,
      cc,
      lang,
      cached: false
    }
  }
    
  pub fn empty() -> Self {
    AutoSuggestResultSet {
      valid: false,
      count: 0,
      results: Vec::new(),
      ts: 0,
      cached: false,
      lang: None,
      cc: None,
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

