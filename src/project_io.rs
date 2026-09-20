use gpui::*;
use std::path::PathBuf;
use std::vec;

use crate::dmgtile::{DMGTile, MAX_TILES};
use crate::{export, project};

impl DMGTile {
    pub fn new_project(&mut self, cx: &mut Context<Self>) {
        self.tiles = vec![[0u8; 64]; MAX_TILES];
        self.modified = vec![false; MAX_TILES];
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_tile = 0;
        self.current_path = None;
        self.tile_thumb_cache = vec![None; MAX_TILES];
        self.preview_image_cache = None;
        self.pattern_image_cache = None;
        self.set_toast("New project".to_string(), false, cx);
    }

    pub fn open_project(&mut self, cx: &mut Context<Self>) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("DMGTile Project", &["dmgtile"])
            .pick_file()
        else {
            return;
        };

        match project::load_from_file(&path) {
            Ok((tiles, modified)) => {
                self.tiles = tiles;
                self.modified = modified;
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.current_tile = 0;
                self.current_path = Some(path.clone());
                self.tile_thumb_cache = vec![None; MAX_TILES];
                self.preview_image_cache = None;
                self.pattern_image_cache = None;
                self.set_toast(format!("Loaded {}", path.display()), false, cx);
            }
            Err(e) => {
                self.set_toast(format!("Failed to load: {}", e), true, cx);
            }
        }
    }

    pub fn save_project(&mut self, cx: &mut Context<Self>) {
        match self.current_path.clone() {
            Some(path) => self.save_project_to(path, cx),
            None => self.save_project_as(cx),
        }
    }

    pub fn save_project_as(&mut self, cx: &mut Context<Self>) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("DMGTile Project", &["dmgtile"])
            .set_file_name("project.dmgtile")
            .save_file()
        else {
            return;
        };
        self.save_project_to(path, cx);
    }

    pub fn save_project_to(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        match project::save_to_file(&self.tiles, &self.modified, &path) {
            Ok(()) => {
                self.current_path = Some(path.clone());
                self.set_toast(format!("Saved {}", path.display()), false, cx);
            }
            Err(e) => {
                self.set_toast(format!("Failed to save: {}", e), true, cx);
            }
        }
    }

    // Export

    pub fn export_bin(&mut self, cx: &mut Context<Self>) {
        if !self.modified.iter().any(|&m| m) {
            self.set_toast("Nothing to export".to_string(), true, cx);
            return;
        }

        let Some(path) = rfd::FileDialog::new()
            .add_filter("GameBoy Tile Binary", &["bin"])
            .set_file_name("tile.bin")
            .save_file()
        else {
            return;
        };

        match export::export_to_bin(&self.tiles, &self.modified, &path) {
            Ok(()) => self.set_toast(format!("Exported {}", path.display()), false, cx),
            Err(e) => self.set_toast(format!("Export failed: {}", e), true, cx),
        }
    }

    pub fn export_c(&mut self, cx: &mut Context<Self>) {
        if !self.modified.iter().any(|&m| m) {
            self.set_toast("Nothing to export".to_string(), true, cx);
            return;
        }

        let Some(path) = rfd::FileDialog::new()
            .add_filter("C Source", &["c"])
            .set_file_name("tile.c")
            .save_file()
        else {
            return;
        };

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(export::sanitize_c_ident)
            .unwrap_or_else(|| "tiles".to_string());

        match export::export_to_c(&self.tiles, &self.modified, &name, &path) {
            Ok(()) => self.set_toast(format!("Exported {}", path.display()), false, cx),
            Err(e) => self.set_toast(format!("Export failed: {}", e), true, cx),
        }
    }
}

