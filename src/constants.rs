pub const BRAVE_SEARCH_BASE: &'static str = "  https://api.search.brave.com/res/v1/web/search";

pub const BRAVE_SUGGEST_BASE: &'static str = "  https://api.search.brave.com/res/v1/suggest/search";

pub const COUNTRY_CODES: [&'static str; 36] = [
  "AR", "AU", "AT", "BE", "BR",
  "CA", "CL", "DK", "FI", "FR",
  "DE", "HK", "IN", "ID", "IT",
  "JP", "KR", "MY", "MX", "NL",
  "NZ", "NO", "CN", "PL", "PT",
  "PH", "RU", "SA", "ZA", "ES",
  "SE", "CH", "TW", "TR", "GB",
  "US"
];

pub fn match_country_code(key: &str) -> Option<String> {
  let cc = key.to_uppercase();
  let cc_key = match cc.as_str() {
    "UK" => "GB",
    _ => cc.as_str()
  };
  if let Some(cc_k) = COUNTRY_CODES.into_iter().find(|k| *k == cc_key) {
    Some(cc_k.to_string())
  } else {
    None
  }
}