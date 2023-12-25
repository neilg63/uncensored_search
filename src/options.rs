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
  pub cc: Option<String>, // country-specific searches are geolocal
  pub lang: Option<String>,
  pub p: Option<i64>, // page=1 is the first
  pub cached: Option<i16>, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BraveSearchOptions {
  pub q: String,
  pub safesearch: SafeMode,
  pub cc: Option<String>,
  pub language: Option<String>,
  pub offset: Option<u16>,
}

impl BraveSearchOptions {
  pub fn new(params: &Query<QueryParams>) -> Self {
    let q = params.q.clone().unwrap_or("".to_string());
    let safekey = params.safe.clone();
    let safesearch = SafeMode::from_opt_key(safekey);
    let cc_opt = params.cc.clone();
    let offset_i64 = params.p.unwrap_or(0) - 1;
    let offset = if offset_i64 >= 0 && offset_i64 <= u16::MAX as i64 { Some(offset_i64 as u16) } else { None };
    let lang_str = params.lang.clone().unwrap_or("".to_string());
    let language = if lang_str.len() > 0 && lang_str.len() < 4 { Some(lang_str.to_lowercase()) } else { None };
    let cc = match cc_opt {
      Some(cc_key) => match_country_code(&cc_key),
      _ => None
    };
    BraveSearchOptions {
      q,
      safesearch,
      cc,
      language,
      offset
    }
  }

  pub fn to_cache_key(&self) -> String {
    slugify(&[
        "cs",
        &self.q,
        self.safesearch.to_short().as_str(),
        self.cc.clone().unwrap_or("all".to_string()).as_str(),
        self.language.clone().unwrap_or("_".to_string()).as_str(),
        self.offset.unwrap_or(0).to_string().as_str()
      ].join("_"))
  }

  pub fn to_suggest_cache_key(&self) -> String {
    slugify(&[
        "br_sugg",
        &self.q,
        self.cc.clone().unwrap_or("all".to_string()).as_str(),
        self.language.clone().unwrap_or("_".to_string()).as_str()
      ].join("_"))
  }

  pub fn cc_val(&self) -> String {
    if let Some(cc_val) = self.cc.clone() {
      cc_val
    } else {
      "".to_owned()
    }
  }

  pub fn lang_code(&self, default_code: &str) -> String {
    if let Some(lang) = self.language.clone() {
      lang
    } else {
      default_code.to_owned()
    }
  }

  pub fn lang(&self) -> Option<String> {
    self.language.clone()
  }

  pub fn country_code(&self) -> Option<String> {
    self.cc.clone()
  }

  pub fn page(&self) -> u16 {
    self.offset.unwrap_or(0) + 1
  }

  pub fn to_tuples(&self) -> Vec<(&str, String)> {
    let mut tuples: Vec<(&str, String)> = vec![("q", self.q.clone()), self.safesearch.to_option()];
    if self.cc.is_some() {
      tuples.push(("country", self.cc_val()));
      tuples.push(("is_geolocal", "true".to_string()));
    } else {
      tuples.push(("is_geolocal", "false".to_string()));
    }
    if self.offset.is_some() {
      tuples.push(("offset", self.offset.unwrap_or(0).to_string()));
    }
    if self.language.is_some() {
      tuples.push(("language", self.lang_code("")));
    }
    tuples
  }

  pub fn to_mojeek_tuples(&self) -> Vec<(&str, String)> {
    let api_key = dotenv::var("MOJEEK_SEARCH").unwrap_or("".to_owned());
    let mut tuples: Vec<(&str, String)> = vec![
      ("q", self.q.clone()),
      ("api_key", api_key),
      ("fmt", "json".to_owned()),
      ("t", 20.to_string()),
    ];
    if self.cc.is_some() {
      tuples.push(("rbb", self.cc_val().to_uppercase()));
    }
    /* if self.offset.is_some() {
      tuples.push(("offset", self.offset.unwrap_or(0).to_string()));
    }*/
    let lang_code = self.lang_code("en").to_uppercase();
    tuples.push(("lb", lang_code));
    if self.language.is_none() {
      tuples.push(("lbb", 50.to_string()));
    }
    tuples
  }

  pub fn to_suggest_tuples(&self) -> Vec<(&str, String)> {
    let mut tuples: Vec<(&str, String)> = vec![
      ("q", self.q.clone()),
      self.safesearch.to_option(),
      ("count", 20.to_string())
    ];
    if self.cc.is_some() {
      tuples.push(("country", self.cc_val()));
      
    }
    if self.language.is_some() {
      tuples.push(("lang", self.lang_code("")));
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SearchProvider {
  #[serde(rename = "textsurf")]
  Text, // TextSurf
  #[serde(rename = "brave")]
  Brave,
  #[serde(rename = "mojeek")]
  Mojeek,
}