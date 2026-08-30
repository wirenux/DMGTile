use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct TileProject {
    pub pixels: Vec<u8>,
}

pub fn save_to_file<P: AsRef<Path>>(pixels: &[u8; 64], path: P) -> io::Result<()> {
    let project = TileProject { pixels: pixels.to_vec() };
    let json = serde_json::to_string_pretty(&project)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, json)
}

pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<[u8; 64]> {
    let json = fs::read_to_string(path)?;
    let project: TileProject = serde_json::from_str(&json)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    project.pixels.try_into()
        .map_err(|v: Vec<u8>| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected 64 pixels, found {}", v.len()),
        ))
}
