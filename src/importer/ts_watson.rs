use std::{fs};
use std::path::Path;
use anyhow::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TdWatsonFrame {
    start_time: Option<i64>,
    end_time: Option<i64>,
    project: Option<String>,
    id: Option<String>,
    tags: Option<Vec<String>>,
    timestamp: Option<i64>,
}

fn from_file(path: impl AsRef<Path>) -> Result<Vec<TdWatsonFrame>, Error> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    parse(data)
}

fn parse(content: String) -> Result<Vec<TdWatsonFrame>, Error> {
    let objects: Vec<TdWatsonFrame> = serde_json::from_str(&content).expect("Failed to parse JSON");
    Ok(objects)
}

#[cfg(test)]
mod tests {
    use crate::importer::ts_watson::{parse};

    #[test]
    fn test() {
        let data = r#"[
            [ 1658206800, 1658216700, "projb", "7cf346662e1049e2a9717c729cd695d1", ["taga"], 1658225005 ],
            [ null, null, "", "", [], null ],
            [ null, 1658217600, "proja", "45352e42521242f6babebecfc104bad5", ["tagb"], 1658225019 ]
            ]"#;
        let result = parse(data.to_string());
    }
}