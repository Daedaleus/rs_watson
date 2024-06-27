use std::fs;
use std::path::Path;

use anyhow::Error;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TdWatsonFrame {
    pub(crate) start_time: Option<i64>,
    pub(crate) end_time: Option<i64>,
    pub(crate) project: Option<String>,
    _id: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    _timestamp: Option<i64>,
}

impl TdWatsonFrame {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Vec<TdWatsonFrame>, Error> {
        let data = fs::read_to_string(path).expect("Unable to read file");
        Self::parse(data)
    }

    fn parse(content: String) -> Result<Vec<TdWatsonFrame>, Error> {
        let objects: Vec<TdWatsonFrame> =
            serde_json::from_str(&content).expect("Failed to parse JSON");
        Ok(objects)
    }
}

#[cfg(test)]
mod tests {
    use crate::importer::td_watson::TdWatsonFrame;

    #[test]
    fn test() {
        let data = r#"[
            [ 1658206800, 1658216700, "projb", "7cf346662e1049e2a9717c729cd695d1", ["taga"], 1658225005 ],
            [ null, null, "", "", [], null ],
            [ null, 1658217600, "proja", "45352e42521242f6babebecfc104bad5", ["tagb"], 1658225019 ]
            ]"#;
        let result = TdWatsonFrame::parse(data.to_string());
        assert!(result.is_ok());
    }
}
