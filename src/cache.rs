use redis::{Commands, RedisResult, Connection, Client};
use chrono::{Local, Duration};
use serde::{Serialize, Deserialize};
use crate::models::*;

pub fn  redis_client() -> RedisResult<Connection> {
  let client = Client::open("redis://127.0.0.1/")?;
  client.get_connection()
}

pub fn get_timestamp() -> i64 {
  let dt = Local::now();
  dt.timestamp()
}

pub fn seconds_ago(ts: i64) -> i64 {
  let now_ts = get_timestamp();
  now_ts - ts
}

fn redis_get_opt_string(key: &str) -> Option<String> {
  if let Ok(mut connection) =  redis_client() {
      let result: String = connection.get(key.to_owned()).unwrap_or("".to_owned());
      Some(result)
  } else {
      None
  }
}


pub fn  redis_set_results(key: &str, result: &ResultSet) -> Option<ResultSet> {
  if let Ok(mut connection) =  redis_client() {
      match serde_json::to_string(result) {
        Ok(value) => match connection.set::<String,String,String>(key.to_string(), value) {
          Ok(_result) => Some(result.to_owned()),
          Err(_error) => None,
        },
        _ => None
      }
  } else {
    None
  }
}

pub fn redis_get_results(key: &str, age: Duration) -> Option<ResultSet> {
  if let Some(result) = redis_get_opt_string(key) {
      if result.len() > 0 {
          let mut data: ResultSet = serde_json::from_str(&result).unwrap_or(ResultSet::empty());
          let max_secs = age.num_seconds();
          if data.retrieved_age() < max_secs {
            data.set_cached();
            Some(data)
          } else {
            None
          }
      } else {
          None
      }
  } else {
      None
  }
}