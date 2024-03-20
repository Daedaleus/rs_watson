use std::fs::File;
use std::path::Path;

use anyhow::Context;
use uuid::Uuid;

pub(crate) mod entries;
pub(crate) mod entry;

pub fn get_or_create_file(file_name: impl AsRef<Path>) -> anyhow::Result<File> {
    let file = File::open(&file_name);
    let file = file.unwrap_or_else(|error| {
        println!("Error: {}", error);
        println!("Creating watson.json");
        File::create(file_name)
            .context("Failed to open filepath for writing")
            .unwrap()
    });
    Ok(file)
}

fn gen_id() -> anyhow::Result<String> {
    Ok(Uuid::new_v4().simple().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_id() {
        let id = gen_id().unwrap();
        assert_eq!(id.len(), 32);
    }

    #[test]
    fn test_gen_id_unique() {
        let id1 = gen_id().unwrap();
        let id2 = gen_id().unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_gen_id_spacer() {
        let id = gen_id().unwrap();
        assert!(!id.contains('-'));
        assert!(!id.contains('_'));
        assert!(!id.contains(' '));
    }

    #[test]
    fn test_get_or_create_file() {
        let file = get_or_create_file("test.json").unwrap();
        assert!(file.metadata().unwrap().is_file());
    }

    #[test]
    fn test_get_or_create_file_exists() {
        let file = get_or_create_file("test.json").unwrap();
        assert!(file.metadata().unwrap().is_file());
        let file = get_or_create_file("test.json").unwrap();
        assert!(file.metadata().unwrap().is_file());
    }
}
