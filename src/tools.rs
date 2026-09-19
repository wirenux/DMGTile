use crate::dmgtile::{DMGTile, GRID_SIZE};

pub enum Tool {
    Draw,
    Bucket,
}

#[derive(Clone)]
pub struct Snapshot {
    pub tiles: Vec<[u8; 64]>,
    pub modified: Vec<bool>,
}

impl DMGTile {
    pub fn active_shade(&self) -> u8 {
        if self.eraser_active { 0 } else { self.current_shade }
    }

    pub fn apply_tools(&mut self, index: usize, shade: u8) -> bool {
        match self.tool {
            Tool::Draw => {
                let changed = self.paint_pixel(index, shade);
                self.previous_pixels = Some(index);
                changed
            }
            Tool::Bucket => self.bucket_fill(index, shade),
        }
    }

    pub fn bucket_fill(&mut self, start_index: usize, new_shade: u8) -> bool {
        let target_shade = self.tiles[self.current_tile][start_index];

        if target_shade == new_shade {
            return false;
        }

        let mut stack = vec![start_index];

        while let Some(index) = stack.pop() {
            if self.tiles[self.current_tile][index] != target_shade {
                continue;
            }

            self.tiles[self.current_tile][index] = new_shade;

            let row = index / GRID_SIZE;
            let col = index % GRID_SIZE;

            if row > 0 {
                stack.push(index - GRID_SIZE);
            }
            if row < GRID_SIZE - 1 {
                stack.push(index + GRID_SIZE);
            }
            if col > 0 {
                stack.push(index - 1);
            }
            if col < GRID_SIZE - 1 {
                stack.push(index + 1);
            }
        }

        self.modified[self.current_tile] = true;
        true
    }

    pub fn paint_pixel(&mut self, index: usize, shade: u8) -> bool {
        if self.tiles[self.current_tile][index] != shade {
            self.tiles[self.current_tile][index] = shade;
            self.modified[self.current_tile] = true;
            true
        } else {
            false
        }
    }

    pub fn start_stroke(&mut self) {
        if !self.stroke_in_progress {
            self.push_undo();
            self.stroke_in_progress = true;
        }
    }

    pub fn end_stroke(&mut self) {
        self.stroke_in_progress = false;
        self.previous_pixels = None;
    }

    pub fn copy_tile(&mut self) {
        self.clipboard = Some(self.tiles[self.current_tile]);
    }

    pub fn cut_tile(&mut self) {
        self.copy_tile();
        self.push_undo();
        self.tiles[self.current_tile] = [0u8; 64];
        self.modified[self.current_tile] = true;
    }

    pub fn paste_tile(&mut self) {
        if let Some(data) = self.clipboard {
            self.push_undo();
            self.tiles[self.current_tile] = data;
            self.modified[self.current_tile] = true;
        }
    }

    pub fn shift_up(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            let source_row = (row + 1) % GRID_SIZE;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] =
                    self.tiles[self.current_tile][source_row * GRID_SIZE + col];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    pub fn shift_down(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            let source_row = (row + GRID_SIZE - 1) % GRID_SIZE;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] =
                    self.tiles[self.current_tile][source_row * GRID_SIZE + col];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    pub fn shift_left(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let source_col = (col + 1) % GRID_SIZE;
                new_pixels[row * GRID_SIZE + col] =
                    self.tiles[self.current_tile][row * GRID_SIZE + source_col];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    pub fn shift_right(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let source_col = (col + GRID_SIZE - 1) % GRID_SIZE;
                new_pixels[row * GRID_SIZE + col] =
                    self.tiles[self.current_tile][row * GRID_SIZE + source_col];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    pub fn flip_horizontally(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            let mirrored_row = GRID_SIZE - 1 - row;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] =
                    self.tiles[self.current_tile][mirrored_row * GRID_SIZE + col];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    pub fn flip_vertically(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let mirrored_col = GRID_SIZE - 1 - col;
                new_pixels[row * GRID_SIZE + col] =
                    self.tiles[self.current_tile][row * GRID_SIZE + mirrored_col];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    pub fn rotate_90_clockwise(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] =
                    self.tiles[self.current_tile][(GRID_SIZE - 1 - col) * GRID_SIZE + row];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    pub fn push_undo(&mut self) {
        self.undo_stack.push(Snapshot {
            tiles: self.tiles.clone(),
            modified: self.modified.clone(),
        });
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                tiles: self.tiles.clone(),
                modified: self.modified.clone(),
            });
            self.tiles = prev.tiles;
            self.modified = prev.modified;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                tiles: self.tiles.clone(),
                modified: self.modified.clone(),
            });
            self.tiles = next.tiles;
            self.modified = next.modified;
        }
    }

    pub fn erase_tile(&mut self) {
        self.push_undo();
        self.tiles[self.current_tile] = [0u8; 64];
        self.modified[self.current_tile] = false;
    }
}
