use std::fs;
use std::path::Path;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct TdWatsonFrame {
    timestamp1: i64,
    timestamp2: i64,
    status: String,
    id: String,
    items: Vec<String>,
    timestamp3: i64,
}

/**
[
 [
  1658206800,
  1658216700,
  "projb",
  "7cf346662e1049e2a9717c729cd695d1",
  [
   "taga"
  ],
  1658225005
 ],
 [
  1658216700,
  1658217600,
  "proja",
  "45352e42521242f6babebecfc104bad5",
  [
   "tagb"
  ],
  1658225019
 ],
**/

fn from_file(path: impl AsRef<Path>) {
    let data = fs::read_to_string(path).expect("Unable to read file");
    let json_value: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");

    // Ausgabe des geparsten Objekts
    println!("{:#?}", json_value);

    // Deserialisierung der JSON-Werte in die benannte Struktur
    let parsed: Vec<Vec<TdWatsonFrame>> = json_value.as_array().expect("Expected an array")
        .iter()
        .map(|inner_array| {
            inner_array.as_array().expect("Expected an array of arrays")
                .iter()
                .map(|frame| {
                    serde_json::from_value(frame.clone()).expect("Failed to deserialize frame")
                })
                .collect()
        })
        .collect();

    // Ausgabe des geparsten Objekts
    println!("{:#?}", parsed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        from_file("/Users/kei/Library/Application Support/watson/frames")
    }
}