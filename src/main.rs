use eframe::egui;
use egui::Color32;

mod export;
mod project;

const CELL_SIZE: f32 = 32.0;
const GRID_SIZE: usize = 8;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([500.0, 400.0]),
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

struct DMGTile {
    pub pixels: [u8; 64],
    previous_pixels: Option<usize>,
    dirty: bool,
    texture: Option<egui::TextureHandle>,
    current_shade: u8,
    right_shade: u8,
    tool: Tool,
    palette: Palette,
    undo_stack: Vec<[u8; 64]>,
    redo_stack: Vec<[u8; 64]>,
    stroke_in_progress: bool,
    export_window: export::ExportWindow,
}

impl Default for DMGTile {
    fn default() -> Self {
        Self {
            pixels: [0; 64],
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
                let shade = self.pixels[index];
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
        let target_shade = self.pixels[start_index];

        if target_shade == new_shade {
            return; // already the same shade
        }

        let mut stack = vec![start_index];

        while let Some(index) = stack.pop() {
            if self.pixels[index] != target_shade {
                continue;
            }

            self.pixels[index] = new_shade;

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
    }

    fn draw(&mut self, origin: egui::Pos2, pointer_pos: egui::Pos2, color: u8) {
        let relative = pointer_pos - origin;
        let col = (relative.x / CELL_SIZE) as i32;
        let row = (relative.y / CELL_SIZE) as i32;

        if col >= 0 && col < GRID_SIZE as i32 && row >= 0 && row < GRID_SIZE as i32 {
            let index = row as usize * GRID_SIZE + col as usize;

            if self.previous_pixels != Some(index) {
                self.pixels[index] = color;
                self.previous_pixels = Some(index);
                self.dirty = true; // mark for texture rebuild next frame
            }
        }
    }

    fn shift_up(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            let source_row = (row + 1) % GRID_SIZE;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.pixels[source_row * GRID_SIZE + col];
            }
        }

        self.pixels = new_pixels;
        self.dirty = true;
    }

    fn shift_down(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            let source_row = (row + GRID_SIZE - 1) % GRID_SIZE;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.pixels[source_row * GRID_SIZE + col];
            }
        }

        self.pixels = new_pixels;
        self.dirty = true;
    }

    fn shift_left(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let source_col = (col + 1) % GRID_SIZE;
                new_pixels[row * GRID_SIZE + col] = self.pixels[row * GRID_SIZE + source_col];
            }
        }

        self.pixels = new_pixels;
        self.dirty = true;
    }

    fn shift_right(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let source_col = (col + GRID_SIZE - 1) % GRID_SIZE;
                new_pixels[row * GRID_SIZE + col] = self.pixels[row * GRID_SIZE + source_col];
            }
        }

        self.pixels = new_pixels;
        self.dirty = true;
    }

    fn flip_horizontally(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            let mirrored_row = GRID_SIZE - 1 - row;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.pixels[mirrored_row * GRID_SIZE + col];
            }
        }

        self.pixels = new_pixels;
        self.dirty = true;
    }

    fn flip_vertically(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let mirrored_col = GRID_SIZE - 1 - col;
                new_pixels[row * GRID_SIZE + col] = self.pixels[row * GRID_SIZE + mirrored_col];
            }
        }

        self.pixels = new_pixels;
        self.dirty = true;
    }

    fn rotate_90_clockwise(&mut self) {
        let mut new_pixels = [0u8; 64];

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.pixels[(GRID_SIZE - 1 - col) * GRID_SIZE + row];
            }
        }

        self.pixels = new_pixels;
        self.dirty = true;
    }

    fn save_dialog(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("tile.dmgtile")
            .add_filter("DMGTile project", &["dmgtile"])
            .save_file()
        {
            match project::save_to_file(&self.pixels, &path) {
                Ok(_) => println!("Successfully saved to {:?}", path),
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
                Ok(pixels) => {
                    self.pixels = pixels;
                    self.dirty = true; // force texture rebuild next frame
                    println!("Successfully opened {:?}", path);
                }
                Err(e) => println!("Failed to open project : {}", e),
            }
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.pixels);
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.pixels);
            self.pixels = prev;
            self.dirty = true;
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.pixels);
            self.pixels = next;
            self.dirty = true;
        }
    }
}

impl eframe::App for DMGTile {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let (undo_press, redo_pressed) = ui.ctx().input(|i| {
            let cmd = i.modifiers.command;
            (
                cmd && !i.modifiers.shift && i.key_pressed(egui::Key::Z),
                cmd && i.modifiers.shift && i.key_pressed(egui::Key::Z),
            )
        });

        if undo_press {
            self.undo();
        }

        if redo_pressed {
            self.redo();
        }

        self.export_window.show(ui.ctx(), &self.pixels);

        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        self.open_dialog();
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
                        println!("{:?}", self.pixels);
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
                        }

                        if ui.button("Green").clicked() {
                            self.palette = Palette::ClassicGreen;
                            self.dirty = true;
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
            });
        });
    }
}
