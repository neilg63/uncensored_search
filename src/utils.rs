use urlencoding::encode;

pub fn build_query_string(options: &[(&str, String)]) -> String {
  let mut params: Vec<String> = Vec::new();
  for pair in options {
    let (key, value) = pair;
    params.push([*key, encode(&value).to_string().as_str()].join("="));
  }
  if params.len() > 0 {
    format!("?{}", params.join("&"))
  } else {
    return "".to_owned()
  }
}

pub fn build_query_option<'a>(key: &'a str, value: String) -> (&'a str, String) {
  (key, value)
}