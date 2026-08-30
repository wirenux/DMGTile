use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

const MAX_TILES: usize = 128;

#[derive(Serialize, Deserialize)]
pub struct TileEntry {
    pub id: usize,
    pub pixels: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct TileProject {
    pub tiles: Vec<TileEntry>,
}

pub fn save_to_file<P: AsRef<Path>>(tiles: &[[u8; 64]], modified: &[bool], path: P) -> io::Result<()> {
    let entries = tiles
        .iter()
        .zip(modified)
        .enumerate()
        .filter(|&(_, (_, used))| *used)
        .map(|(id, (pixels, _))| TileEntry { id, pixels: pixels.to_vec() })
        .collect();
    let project = TileProject { tiles: entries };
    let json = serde_json::to_string_pretty(&project)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, json)
}


pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<(Vec<[u8; 64]>, Vec<bool>)> {
    let json = fs::read_to_string(path)?;
    let project: TileProject = serde_json::from_str(&json)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut tiles = vec![[0u8; 64]; MAX_TILES];
    let mut modified = vec![false; MAX_TILES];

    for entry in project.tiles {
        if entry.id < tiles.len() {
            let arr: [u8; 64] = entry.pixels.try_into().map_err(|v: Vec<u8>| {
                io::Error::new(io::ErrorKind::InvalidData, format!("tile {} has {} pixels, expected 64", entry.id, v.len()))
            })?;
            tiles[entry.id] = arr;
            modified[entry.id] = true;
        }
    }

    Ok((tiles, modified))
}
