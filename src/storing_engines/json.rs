use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use crate::Frame;

pub fn read(path: String) -> anyhow::Result<Vec<Frame>> {
    let mut file = load_file(path);
    let mut content = String::new();

    file.read_to_string(&mut content)?;
    let frames: Vec<Frame> = serde_json::from_str(&content)?;

    Ok(frames)
}

pub fn write(frame: Frame, path: String) -> anyhow::Result<()> {
    let mut stored_frames = read(path.clone())?;
    stored_frames.push(frame);

    let mut file = load_file(path);
    file.write_all(serde_json::to_string(&stored_frames)?.as_ref())?;

    Ok(())
}

fn load_file(path: String) -> File {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .expect("Cannot open file")
}
