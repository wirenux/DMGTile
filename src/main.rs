use eframe::egui;
use egui::Color32;

use std::path::PathBuf;

mod export;
mod project;

const MAX_TILES: usize = 128;

const CELL_SIZE: f32 = 32.0;
const GRID_SIZE: usize = 8;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([600.0, 450.0]),
        ..Default::default()
    };
    eframe::run_native(
        "DMGTile",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<DMGTile>::default())
        })
    )
}

enum Tool {
    Draw,
    Bucket,
}

enum Palette {
    Grayscale,
    ClassicGreen,
}

#[derive(Clone)]
struct Snapshot {
    tiles: Vec<[u8; 64]>,
    modified: Vec<bool>,
}

struct DMGTile {
    tiles: Vec<[u8; 64]>,
    current_tile: usize,
    previous_pixels: Option<usize>,
    dirty: bool,
    texture: Option<egui::TextureHandle>,
    current_shade: u8,
    right_shade: u8,
    tool: Tool,
    palette: Palette,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    stroke_in_progress: bool,
    export_window: export::ExportWindow,
    thumbnails: Vec<Option<egui::TextureHandle>>,
    modified: Vec<bool>,
    current_path: Option<PathBuf>,
}

impl Default for DMGTile {
    fn default() -> Self {
        Self {
            tiles: vec![[0u8; 64]; MAX_TILES],
            current_tile: 0,
            previous_pixels: None,
            dirty: true, // force first-frame render
            texture: None,
            current_shade: 3,
            right_shade: 0,
            tool: Tool::Draw,
            palette: Palette::ClassicGreen,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            stroke_in_progress: false,
            export_window: export::ExportWindow::default(),
            thumbnails: vec![None; MAX_TILES],
            modified: vec![false; MAX_TILES],
            current_path: None,
        }
    }
}

impl DMGTile {
    fn shade_color(shade: u8, palette: &Palette) -> egui::Color32 {
        match palette {
            Palette::Grayscale => match shade {
                0 => egui::Color32::from_hex("#ffffff").unwrap(),
                1 => egui::Color32::from_hex("#aaaaaa").unwrap(),
                2 => egui::Color32::from_hex("#555555").unwrap(),
                _ => egui::Color32::from_hex("#000000").unwrap(),
            },
            Palette::ClassicGreen => match shade {
                0 => egui::Color32::from_hex("#e0f8d0").unwrap(),
                1 => egui::Color32::from_hex("#88c070").unwrap(),
                2 => egui::Color32::from_hex("#346856").unwrap(),
                _ => egui::Color32::from_hex("#081820").unwrap(),
            },
        }
    }

    fn rebuild_texture(&mut self, ctx: &egui::Context) {
        let mut image = egui::ColorImage::new(
            [GRID_SIZE, GRID_SIZE],
            vec![egui::Color32::BLACK; GRID_SIZE * GRID_SIZE],
        );

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let index = row * GRID_SIZE + col;
                let shade = self.tiles[self.current_tile][index];
                let color = Self::shade_color(shade, &self.palette);
                image.pixels[index] = color;
            }
        }

        self.texture = Some(ctx.load_texture(
            "tile_canvas", // name
            image,
            egui::TextureOptions::NEAREST,
        ));

        self.dirty = false;
    }

    fn bucket_fill(&mut self, start_index: usize, new_shade: u8) {
        let target_shade = self.tiles[self.current_tile][start_index];

        if target_shade == new_shade {
            return; // already the same shade
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
                stack.push(index - GRID_SIZE); // up
            }
            if row < GRID_SIZE - 1 {
                stack.push(index + GRID_SIZE); // down
            }
            if col > 0 {
                stack.push(index - 1); // left
            }
            if col < GRID_SIZE - 1 {
                stack.push(index + 1); // right
            }
        }

        self.dirty = true;
        self.thumbnails[self.current_tile] = None;
    }

    fn draw(&mut self, origin: egui::Pos2, pointer_pos: egui::Pos2, color: u8) {
        let relative = pointer_pos - origin;
        let col = (relative.x / CELL_SIZE) as i32;
        let row = (relative.y / CELL_SIZE) as i32;

        if col >= 0 && col < GRID_SIZE as i32 && row >= 0 && row < GRID_SIZE as i32 {
            let index = row as usize * GRID_SIZE + col as usize;

            if self.previous_pixels != Some(index) {
                self.tiles[self.current_tile][index] = color;
                self.previous_pixels = Some(index);
                self.dirty = true; // mark for texture rebuild next frame
                self.thumbnails[self.current_tile] = None;
                self.modified[self.current_tile] = true;
            }
        }
    }

    fn shift_up(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            let source_row = (row + 1) % GRID_SIZE;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][source_row * GRID_SIZE + col];
            }
        }

        self.tiles[self.current_tile] = new_pixels;
        self.dirty = true;
        self.thumbnails[self.current_tile] = None;
    }

    fn shift_down(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            let source_row = (row + GRID_SIZE - 1) % GRID_SIZE;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][source_row * GRID_SIZE + col];
            }
        }

        self.tiles[self.current_tile] = new_pixels;
        self.dirty = true;
        self.thumbnails[self.current_tile] = None;
    }

    fn shift_left(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let source_col = (col + 1) % GRID_SIZE;
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][row * GRID_SIZE + source_col];
            }
        }

        self.tiles[self.current_tile] = new_pixels;
        self.dirty = true;
        self.thumbnails[self.current_tile] = None;
    }

    fn shift_right(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let source_col = (col + GRID_SIZE - 1) % GRID_SIZE;
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][row * GRID_SIZE + source_col];
            }
        }

        self.tiles[self.current_tile] = new_pixels;
        self.dirty = true;
        self.thumbnails[self.current_tile] = None;
    }

    fn flip_horizontally(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            let mirrored_row = GRID_SIZE - 1 - row;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][mirrored_row * GRID_SIZE + col];
            }
        }

        self.tiles[self.current_tile] = new_pixels;
        self.dirty = true;
        self.thumbnails[self.current_tile] = None;
    }

    fn flip_vertically(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let mirrored_col = GRID_SIZE - 1 - col;
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][row * GRID_SIZE + mirrored_col];
            }
        }

        self.tiles[self.current_tile] = new_pixels;
        self.dirty = true;
        self.thumbnails[self.current_tile] = None;
    }

    fn rotate_90_clockwise(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][(GRID_SIZE - 1 - col) * GRID_SIZE + row];
            }
        }

        self.tiles[self.current_tile] = new_pixels;
        self.dirty = true;
        self.thumbnails[self.current_tile] = None;
    }

    fn save(&mut self) {
        if let Some(path) = self.current_path.clone() {
            match project::save_to_file(&self.tiles, &self.modified, &path) {
                Ok(_) => println!("Successfully saved to {:?}", path),
                Err(e) => println!("Failed to save project : {}", e),
            }
        } else {
            self.save_dialog();
        }
    }

    fn save_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("tile.dmgtile")
            .add_filter("DMGTile project", &["dmgtile"])
            .save_file()
        {
            match project::save_to_file(&self.tiles, &self.modified, &path) {
                Ok(_) => {
                    println!("Successfully saved to {:?}", path);
                    self.current_path = Some(path);
                }
                Err(e) => println!("Failed to save project : {}", e),
            }
        }
    }

    fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("DMGTile project", &["dmgtile"])
            .pick_file()
        {
            match project::load_from_file(&path) {
                Ok((tiles, modified)) => {
                    self.tiles = tiles;
                    self.modified = modified;
                    self.dirty = true;
                    self.thumbnails = vec![None; MAX_TILES];
                    println!("Successfully opened {:?}", path);
                    self.current_path = Some(path);
                }
                Err(e) => println!("Failed to open project : {}", e),
            }
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(Snapshot {
            tiles: self.tiles.clone(),
            modified: self.modified.clone()
        });
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                tiles: self.tiles.clone(),
                modified: self.modified.clone(),
            });
            self.tiles = prev.tiles;
            self.modified = prev.modified;
            self.dirty = true;
            self.thumbnails = vec![None; MAX_TILES];
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                tiles: self.tiles.clone(),
                modified: self.modified.clone(),
            });
            self.tiles = next.tiles;
            self.modified = next.modified;
            self.dirty = true;
            self.thumbnails = vec![None; MAX_TILES];
        }
    }

    fn thumbnails_for(&mut self, ctx: &egui::Context, idx: usize) -> egui::TextureHandle {
        if self.thumbnails[idx].is_none() {
            let mut image = egui::ColorImage::new(
                [GRID_SIZE, GRID_SIZE],
                vec![egui::Color32::BLACK; GRID_SIZE * GRID_SIZE],
            );
            for i in 0..GRID_SIZE * GRID_SIZE {
                image.pixels[i] = Self::shade_color(self.tiles[idx][i], &self.palette)
            }
            self.thumbnails[idx] = Some(ctx.load_texture(
                format!("thumb_{idx}"),
                image,
                egui::TextureOptions::NEAREST,
            ));
        }

        self.thumbnails[idx].as_ref().unwrap().clone()
    }
}

impl eframe::App for DMGTile {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let (undo_press, redo_pressed, save_pressed) = ui.ctx().input(|i| {
            let cmd = i.modifiers.command;
            (
                cmd && !i.modifiers.shift && i.key_pressed(egui::Key::Z),
                cmd && i.modifiers.shift && i.key_pressed(egui::Key::Z),
                cmd && i.key_pressed(egui::Key::S),
            )
        });

        if undo_press {
            self.undo();
        }

        if redo_pressed {
            self.redo();
        }

        if save_pressed {
            self.save();
        }

        self.export_window.show(ui.ctx(), &self.tiles, &self.modified);

        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New project").clicked() {
                        self.push_undo();
                        self.tiles = vec![[0u8; 64]; MAX_TILES];
                        self.modified = vec![false; MAX_TILES];
                        self.dirty = true;
                        self.thumbnails = vec![None; MAX_TILES];
                        self.current_path = None;
                        ui.close();
                    }
                    if ui.button("Open...").clicked() {
                        self.open_dialog();
                        ui.close();
                    }
                    if ui.button("Save").clicked() {
                        self.save();
                        ui.close();
                    }
                    if ui.button("Save As...").clicked() {
                        self.save_dialog();
                        ui.close();
                    }
                    if ui.button("Export As...").clicked() {
                        self.export_window.open();
                        ui.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        self.undo();
                        ui.close();
                    }
                    if ui.button("Redo").clicked() {
                        self.redo();
                        ui.close();
                    }
                });
                ui.menu_button("Dev", |ui| {
                    if ui.button("Print self.pixels[]").clicked() {
                        println!("{:?}", self.tiles[self.current_tile]);
                        ui.close();
                    }
                })
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if self.dirty {
                self.rebuild_texture(ui.ctx());
            }

            ui.horizontal(|ui| {
                ui.vertical(|ui| { // Toolbar
                    ui.set_width(45.0);

                    ui.scope(|ui| {
                        ui.spacing_mut().button_padding = egui::vec2(2.0, 2.0);

                        let pen_selected = matches!(self.tool, Tool::Draw);
                        if ui.add(
                            egui::Button::image(
                                egui::Image::new(egui::include_image!("../assets/aseprite/pen.png"))
                                .texture_options(egui::TextureOptions::NEAREST)
                                .fit_to_exact_size(egui::vec2(32.0, 32.0)),
                            )
                            .selected(pen_selected)
                        ).clicked() {
                            self.tool = Tool::Draw;
                        }

                        let bucket_selected = matches!(self.tool, Tool::Bucket);
                        if ui.add(
                            egui::Button::image(
                                egui::Image::new(egui::include_image!("../assets/aseprite/bucket.png"))
                                .texture_options(egui::TextureOptions::NEAREST)
                                .fit_to_exact_size(egui::vec2(32.0, 32.0)),
                            )
                            .selected(bucket_selected)
                        ).clicked() {
                            self.tool = Tool::Bucket;
                        }

                        ui.separator();

                        if ui.add(
                            egui::Button::image(
                                egui::Image::new(egui::include_image!("../assets/aseprite/up.png"))
                                .texture_options(egui::TextureOptions::NEAREST)
                                .fit_to_exact_size(egui::vec2(32.0, 32.0)),
                            )
                        ).clicked() {
                            self.push_undo();
                            Self::shift_up(self);
                        }

                        if ui.add(
                            egui::Button::image(
                                egui::Image::new(egui::include_image!("../assets/aseprite/left.png"))
                                .texture_options(egui::TextureOptions::NEAREST)
                                .fit_to_exact_size(egui::vec2(32.0, 32.0)),
                            )
                        ).clicked() {
                            self.push_undo();
                            Self::shift_left(self);
                        }

                        if ui.add(
                            egui::Button::image(
                                egui::Image::new(egui::include_image!("../assets/aseprite/right.png"))
                                .texture_options(egui::TextureOptions::NEAREST)
                                .fit_to_exact_size(egui::vec2(32.0, 32.0)),
                            )
                        ).clicked() {
                            self.push_undo();
                            Self::shift_right(self);
                        }

                        if ui.add(
                            egui::Button::image(
                                egui::Image::new(egui::include_image!("../assets/aseprite/down.png"))
                                .texture_options(egui::TextureOptions::NEAREST)
                                .fit_to_exact_size(egui::vec2(32.0, 32.0)),
                            )
                        ).clicked() {
                            self.push_undo();
                            Self::shift_down(self);
                        }

                        if ui.add(
                            egui::Button::image(
                                egui::Image::new(egui::include_image!("../assets/aseprite/flipV.png")) // it's reversed cause there is 2 way to understand it
                                .texture_options(egui::TextureOptions::NEAREST)
                                .fit_to_exact_size(egui::vec2(32.0, 32.0)),
                            )
                        ).clicked() {
                            self.push_undo();
                            Self::flip_horizontally(self);
                        }

                        if ui.add(
                            egui::Button::image(
                                egui::Image::new(egui::include_image!("../assets/aseprite/flipH.png"))
                                .texture_options(egui::TextureOptions::NEAREST)
                                .fit_to_exact_size(egui::vec2(32.0, 32.0)),
                            )
                        ).clicked() {
                            self.push_undo();
                            Self::flip_vertically(self);
                        }

                        if ui.add(
                            egui::Button::image(
                                egui::Image::new(egui::include_image!("../assets/aseprite/rotate.png"))
                                .texture_options(egui::TextureOptions::NEAREST)
                                .fit_to_exact_size(egui::vec2(32.0, 32.0)),
                            )
                        ).clicked() {
                            self.push_undo();
                            Self::rotate_90_clockwise(self);
                        }
                    });
                });

                ui.vertical(|ui| {
                    let size = egui::vec2(CELL_SIZE * GRID_SIZE as f32, CELL_SIZE * GRID_SIZE as f32);
                    let texture = self.texture.as_ref().unwrap();

                    let response = ui.add(
                        egui::Image::new(texture)
                            .fit_to_exact_size(size)
                            .sense(egui::Sense::click_and_drag()),
                    );

                    let origin = response.rect.min;

                    let painter = ui.painter_at(response.rect);
                    let line_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(50));

                    for i in 0..=GRID_SIZE {
                        let offset = i as f32 * CELL_SIZE;

                        // vertical line
                        painter.line_segment(
                            [origin + egui::vec2(offset, 0.0), origin + egui::vec2(offset, size.y)],
                            line_stroke,
                        );

                        // horizontal line
                        painter.line_segment(
                            [origin + egui::vec2(0.0, offset), origin + egui::vec2(size.x, offset)],
                            line_stroke,
                        );
                    }

                    if (response.clicked() || response.dragged() || response.secondary_clicked())
                        && let Some(pointer_pos) = response.interact_pointer_pos()
                    {
                        if !self.stroke_in_progress {
                            self.push_undo();
                            self.stroke_in_progress = true;
                        }

                        let shade = if ui.input(|i| i.pointer.secondary_down()) || response.secondary_clicked() {
                            self.right_shade
                        } else {
                            self.current_shade
                        };

                        match self.tool {
                            Tool::Draw => self.draw(origin, pointer_pos, shade),
                            Tool::Bucket => {
                                if response.clicked() || response.secondary_clicked() || response.dragged() {
                                    let relative = pointer_pos - origin;
                                    let col = (relative.x / CELL_SIZE) as i32;
                                    let row = (relative.y / CELL_SIZE) as i32;

                                    if col >= 0 && col < GRID_SIZE as i32 && row >= 0 && row < GRID_SIZE as i32 {
                                        let index = row as usize * GRID_SIZE + col as usize;
                                        self.bucket_fill(index, shade);
                                    }
                                }
                            }
                        }
                    } else {
                        self.previous_pixels = None;
                        self.stroke_in_progress = false;
                    }
                    ui.horizontal(|ui| {
                        ui.label("L");
                        egui::Frame::default()
                            .fill(Self::shade_color(self.current_shade, &self.palette))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(50)))
                            .inner_margin(egui::Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                let text_color = if self.current_shade <= 1 { Color32::BLACK } else { Color32::WHITE };
                                ui.label(egui::RichText::new(self.current_shade.to_string()).color(text_color));
                            });

                        ui.label("R");
                        egui::Frame::default()
                            .fill(Self::shade_color(self.right_shade, &self.palette))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(50)))
                            .inner_margin(egui::Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                let text_color = if self.right_shade <= 1 { Color32::BLACK } else { Color32::WHITE };
                                ui.label(egui::RichText::new(self.right_shade.to_string()).color(text_color));
                            });

                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;

                            for shade in 0u8..4 {
                                let text_color = if shade <= 1 { Color32::BLACK } else { Color32::WHITE };
                                let selected = self.current_shade == shade;
                                let right_selected = self.right_shade == shade;
                                let stroke = if selected {
                                    egui::Stroke::new(1.0, egui::Color32::BLUE)
                                } else if right_selected {
                                    egui::Stroke::new(1.0, egui::Color32::RED)
                                } else {
                                    egui::Stroke::NONE
                                };
                                let button = egui::Button::new(egui::RichText::new(shade.to_string()).color(text_color))
                                    .fill(Self::shade_color(shade, &self.palette))
                                    .stroke(stroke)
                                    .min_size(egui::vec2(24.0, 20.0));

                                let response = ui.add(button);
                                if response.clicked() {
                                    self.current_shade = shade;
                                }
                                if response.secondary_clicked() {
                                    self.right_shade = shade;
                                }
                            }
                        });

                        ui.separator();

                        if ui.button("Gray").clicked() {
                            self.palette = Palette::Grayscale;
                            self.dirty = true;
                            self.thumbnails[self.current_tile] = None;
                        }

                        if ui.button("Green").clicked() {
                            self.palette = Palette::ClassicGreen;
                            self.dirty = true;
                            self.thumbnails[self.current_tile] = None;
                        }
                    });
                });
                ui.vertical(|ui| {
                    let texture = self.texture.as_ref().unwrap();

                    egui::Frame::default()
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(50)))
                        .show(ui, |ui| {
                            ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(32.0, 32.0)));
                        });

                    egui::Frame::default()
                        .inner_margin(0.0)
                        .show(ui, |ui| {
                            ui.scope(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                                
                                for _row in 0..4 {
                                    ui.horizontal(|ui| {
                                        for _col in 0..4 {
                                            ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(32.0, 32.0)));
                                        }
                                    });
                                }
                            });
                        });
                });
                ui.vertical(|ui| {
                    ui.set_width(70.0);
                    egui::ScrollArea::vertical()
                        .max_height(CELL_SIZE * GRID_SIZE as f32)
                        .show(ui, |ui| {
                            for idx in 0..self.tiles.len() {
                                let selected = self.current_tile == idx;
                                ui.horizontal(|ui| {
                                    let label_response = ui.add(
                                        egui::Button::selectable(selected, format!("{idx}")).min_size(egui::vec2(32.0, 18.0)),
                                    );
                                    
                                    let thumb = self.thumbnails_for(ui.ctx(), idx);
                                    let image_response = ui.add(
                                        egui::Image::new(&thumb)
                                            .fit_to_exact_size(egui::vec2(16.0, 16.0))
                                            .sense(egui::Sense::click()),
                                    );

                                    if label_response.clicked() || image_response.clicked() {
                                        self.current_tile = idx;
                                        self.dirty = true;
                                        self.thumbnails[self.current_tile] = None;
                                    }
                                });
                            }
                        });
                });
            });
        });
    }
}
