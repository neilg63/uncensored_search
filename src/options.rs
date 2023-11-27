use axum::extract::Query;
use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;
use slug::slugify;
use crate::constants::match_country_code;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryParams {
  pub q: Option<String>,
  pub safe: Option<String>,
  pub cc: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BraveSearchOptions {
  pub q: String,
  pub safesearch: SafeMode,
  pub cc: Option<String>
}

impl BraveSearchOptions {
  pub fn new(params: &Query<QueryParams>) -> Self {
    let q = params.q.clone().unwrap_or("".to_string());
    let safekey = params.safe.clone();
    let safesearch = SafeMode::from_opt_key(safekey);
    let cc_opt = params.cc.clone();
    let cc = match cc_opt {
      Some(cc_key) => match_country_code(&cc_key),
      _ => None
    };
    BraveSearchOptions {
      q,
      safesearch,
      cc
    }
  }

  pub fn to_cache_key(&self) -> String {
    slugify(&["brave", &self.q, self.safesearch.to_short().as_str(), self.cc.clone().unwrap_or("all".to_string()).as_str()].join("_"))
  }

  pub fn cc_val(&self) -> String {
    if let Some(cc_val) = self.cc.clone() {
      cc_val.clone()
    } else {
      "".to_owned()
    }
  }

  pub fn to_tuples(&self) -> Vec<(&str, String)> {
    let mut tuples: Vec<(&str, String)> = vec![("q", self.q.clone()), self.safesearch.to_option()];
    if self.cc.is_some() {
      tuples.push(("country", self.cc_val()));
    }
    tuples
  }


}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SafeMode {
  Off,
  Moderate,
  Strict,
}
impl SafeMode {
  pub fn from_key(key: &str) -> Self {
    let lc_key = key.to_lowercase();
    match lc_key.as_str() {
      "on" | "2" | "strict" => SafeMode::Strict,
      "m" | "mild" | "partial" | "1" | "moderate" => SafeMode::Moderate,
      _ => SafeMode::Off,
    }
  }

  pub fn from_opt_key(key: Option<String>) -> Self {
    let ref_key = key.unwrap_or("off".to_string());
    SafeMode::from_key(&ref_key)
  }

  pub fn to_option(&self) -> (&str, String) {
    let value = match self {
      SafeMode::Strict => "strict",
      SafeMode::Moderate => "moderate",
      _ => "off",
    };
    ("safemode", value.to_owned())
  }

  pub fn to_short(&self) -> String {
    match self {
      SafeMode::Strict => "y",
      SafeMode::Moderate => "m",
      _ => "n",
    }.to_string()
  }

}