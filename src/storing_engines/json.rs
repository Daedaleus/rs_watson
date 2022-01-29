use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use crate::config::load;
use crate::Frame;

pub fn read() -> anyhow::Result<Vec<Frame>> {
    let json_path = load()?.json_path;
    let mut file = load_file(json_path);
    let mut content = String::new();

    file.read_to_string(&mut content)?;
    let frames: Vec<Frame> = serde_json::from_str(&content)?;

    Ok(frames)
}

pub fn write(frame: Frame) -> anyhow::Result<()> {
    let json_path = load()?.json_path;
    let mut stored_frames = read()?;
    stored_frames.push(frame);
    let mut file = load_file(json_path);
    file.write_all(serde_json::to_string(&stored_frames)?.as_ref())?;

    Ok(())
}

pub fn write_all(frames: Vec<Frame>) -> anyhow::Result<()> {
    let json_path = load()?.json_path;
    let mut file = load_file(json_path);
    file.write_all(serde_json::to_string(&frames)?.as_ref())?;
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
