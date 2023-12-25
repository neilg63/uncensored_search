use regex::*;

pub trait PatternMatch {

  fn pattern_match_opt(&self, pattern: &str, case_insensitive: bool) -> Option<bool>;

  fn pattern_match(&self, pattern: &str, case_insensitive: bool) -> bool;

  fn pattern_replace(&self, pattern: &str, replacement: &str, case_insensitive: bool) -> Self where Self:Sized;

  fn pattern_replace_opt(&self, pattern: &str, replacement: &str,case_insensitive: bool) -> Option<Self> where Self:Sized;

  fn strip_non_chars(&self) -> Self where Self:Sized;

}

pub trait CharGroupMatch {
  fn has_digits(&self) -> bool;

  fn has_alphanumeric(&self) -> bool;

  fn has_alphabetic(&self) -> bool;
}

pub trait ExtractSegments {

  fn extract_segments(&self, separator: &str) -> Vec<Self> where Self:Sized;

  fn extract_head(&self, separator: &str) -> Self  where Self:Sized;

  fn extract_segment(&self, separator: &str, index: i32) -> Option<Self>  where Self:Sized;

  fn extract_inner_segment(&self, groups: &[(&str, i32)]) -> Option<Self>  where Self:Sized;

  fn extract_tail(&self, separator: &str) -> Self where Self:Sized;

  fn extract_head_pair(&self, separator: &str) -> (Self, Self)  where Self:Sized;

  fn extract_tail_pair(&self, separator: &str) -> (Self, Self)  where Self:Sized;

}

pub trait ToStrings {
  fn to_strings(&self) -> Vec<String>;
}

impl<T: ToString> ToStrings for Vec<T> {
  fn to_strings(&self) -> Vec<String> {
      self.into_iter().map(|s| s.to_string()).collect()
  }
}

impl<T: ToString> ToStrings for [T] {
  fn to_strings(&self) -> Vec<String> {
      self.into_iter().map(|s| s.to_string()).collect::<Vec<String>>()
  }
}


fn build_regex(pattern: &str, case_insensitive: bool) -> Result<Regex, Error> {
  let mut parts: Vec<&str> = vec![];
  if case_insensitive {
    parts.push("(?i)");
  }
  parts.push(pattern);
  let regex_str = parts. concat();
  Regex::new(&regex_str)
}

impl PatternMatch for String {

  ///
  /// Simple regex-compatible match method that will return an optional boolean 
  /// - Some(true) means the regex is valid and the string matches
  /// - Some(false) means the regex is valid and the string does not match
  /// - None means the regex is not valid and can this not be evaluated
  /// 
  fn pattern_match_opt(&self, pattern: &str, case_insensitive: bool) -> Option<bool> {
    if let Ok(re) = build_regex(pattern, case_insensitive) {
      Some(re.is_match(self))
    } else {
      None
    }
}

  ///
  /// Simpple regex-compatible match method that will return false 
  /// if the pattern does not match the source string or the regex fails
  /// 
  fn pattern_match(&self, pattern: &str, case_insensitive: bool) -> bool {
      if let Ok(re) = build_regex(pattern, case_insensitive) {
        re.is_match(self)
      } else {
        false
      }
  }

  ///
  /// Optional regex-enabledd replace method that will return None if the regex fails
  /// 
  fn pattern_replace_opt(&self, pattern: &str, replacement: &str, case_insensitive: bool) -> Option<String> {
    if let Ok(re) = build_regex(pattern, case_insensitive) {
      Some(re.replace_all(self, replacement).to_string())
    } else {
      None
    }  
  }

  ///
  /// Simple regex-enabledd replace method that will return the same string if the regex fails
  /// 
  fn pattern_replace(&self, pattern: &str, replacement: &str, case_insensitive: bool) -> String {
    self.pattern_replace_opt(pattern, replacement, case_insensitive).unwrap_or(self.to_owned())
  }

  fn strip_non_chars(&self) -> String {
    self.chars().into_iter().filter(|c| c.is_alphanumeric()).collect::<String>()
  }

}

impl CharGroupMatch for String {

  fn has_digits(&self) -> bool {
      self.chars().any(|c| char::is_digit(c, 10))
  }

  fn has_alphanumeric(&self) -> bool {
      self.chars().any(char::is_alphanumeric)
  }

  fn has_alphabetic(&self) -> bool {
    self.chars().any(char::is_alphabetic)
  }
}

impl PatternMatch for Vec<String> {

  ///
  /// Simple regex-compatible match method that will return an optional boolean 
  /// on a vector of strngs. The regex need only be compiled once
  /// - Some(true) means the regex is valid and the string matches
  /// - Some(false) means the regex is valid and the string does not match
  /// - None means the regex is not valid and can this not be evaluated
  /// 
  fn pattern_match_opt(&self, pattern: &str, case_insensitive: bool) -> Option<bool> {
    if let Ok(re) = build_regex(pattern, case_insensitive) {
      let matched = self.into_iter().any(|segment| re.is_match(segment));
      Some(matched)
    } else {
      None
    }
}

  ///
  /// Simpple regex-compatible match method that will return false 
  /// if the pattern does not match the source string or the regex fails
  /// 
  fn pattern_match(&self, pattern: &str, case_insensitive: bool) -> bool {
    self.pattern_match_opt(pattern, case_insensitive).unwrap_or(false)
  }

  ///
  /// Optional regex-enabledd replace method that will return None if the regex fails
  /// 
  fn pattern_replace_opt(&self, pattern: &str, replacement: &str, case_insensitive: bool) -> Option<Vec<String>> {
    if let Ok(re) = build_regex(pattern, case_insensitive) {
      let replacements = self.into_iter().map(|segment| re.replace_all(segment, replacement).to_string()).collect::<Vec<String>>();
      Some(replacements)
    } else {
      None
    }  
  }

  ///
  /// Simple regex-enabledd replace method that will return the same string if the regex fails
  /// 
  fn pattern_replace(&self, pattern: &str, replacement: &str, case_insensitive: bool) -> Vec<String> {
    self.pattern_replace_opt(pattern, replacement, case_insensitive).unwrap_or(self.to_owned())
  }

  fn strip_non_chars(&self) -> Vec<String> {
    self.into_iter().map(|segment| segment.strip_non_chars()).collect()
  }

}



impl ExtractSegments for String {
  fn extract_segments(&self, separator: &str) -> Vec<String> {
    let splitter = self.split(separator);
    splitter.into_iter().map(|s| s.to_string()).collect::<Vec<String>>()
  }

  fn extract_head(&self, separator: &str) -> String {
    if let Some((head, _tail)) = self.split_once(separator) {
      head.to_string()
    } else {
      self.to_owned()
    }
  }

  fn extract_tail(&self, separator: &str) -> String {
    let parts = self.extract_segments(separator);
    if parts.len() > 0 {
      parts.last().unwrap_or(self).to_owned()
    } else {
      self.to_owned()
    }
  }

  fn extract_segment(&self, separator: &str, index: i32) -> Option<String> {
    let parts = self.extract_segments(separator);
    let num_parts = parts.len();
    let target_index = if index >= 0 { index as usize } else { (num_parts as i32 + index) as usize };
    if target_index < num_parts {
      if let Some(segment) = parts.get(target_index) {
        Some(segment.to_owned())
      } else {
        None
      }
    } else {
      None
    }
  }

  fn extract_inner_segment(&self, groups: &[(&str, i32)]) -> Option<String> {
    if groups.len() > 0 {
      let mut matched: Option<String> = None;
      let mut current_string = self.clone();
      for group in groups {
        if current_string.len() > 0 {
          let (separator, index) = group;
          matched = current_string.extract_segment(*separator, *index);
          current_string = matched.clone().unwrap_or("".to_string());
        }
      }
      matched
    } else {
      None
    }
  }

  /// 
  /// Extract a tupe of the head and remainder, like split_once but returns Strings
  fn extract_head_pair(&self, separator: &str) -> (String, String) {
    if let Some((head, tail)) = self.split_once(separator) {
      (head.to_string(), tail.to_string())
    } else {
      ("".to_owned(), self.to_owned())
    }
  }

  /// 
  /// Extract a tupe of the tail and remainder, like split_once in reverse and returning Strings
  fn extract_tail_pair(&self, separator: &str) -> (String, String) {
    let parts = self.extract_segments(separator);
    let mut head = "".to_string();
    if parts.len() > 0 {
      let tail = parts.last().unwrap_or(self).to_owned();
      let num_parts = parts.len();
      if num_parts > 1 {
        let mut head_parts: Vec<&str> = vec![];
        let head_end = num_parts - 1;
        for i in 0..head_end {
          if let Some(part) = parts.get(i) {
            head_parts.push(part);
          }
        }
        head = head_parts.join(separator);
      }
      (tail, head)
    } else {
      (self.to_owned(), head)
    }
  }

}