use eframe::egui;
use egui::Color32;

const CELL_SIZE: f32 = 32.0;
const GRID_SIZE: usize = 8;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "DMGTile",
        options,
        Box::new(|_cc| Ok(Box::<DMGTile>::default())),
    )
}

enum Tool {
    Draw,
    Bucket,
}

struct DMGTile {
    pixels: [u8; 64],
    previous_pixels: Option<usize>,
    dirty: bool,
    texture: Option<egui::TextureHandle>,
    current_shade: u8,
    right_shade: u8,
    tool: Tool,
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
        }
    }
}

impl DMGTile {
    fn shade_color(shade: u8) -> egui::Color32 {
        match shade {
            0 => egui::Color32::from_gray(255),
            1 => egui::Color32::from_gray(170),
            2 => egui::Color32::from_gray(85),
            _ => egui::Color32::from_gray(0),
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
                let color = Self::shade_color(shade);
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
}

impl eframe::App for DMGTile {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            if self.dirty {
                self.rebuild_texture(ui.ctx());
            }

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

            ui.vertical(|ui| {
                if ui.button("Pen").clicked() {
                    self.tool = Tool::Draw;
                }
                if ui.button("Fill").clicked() {
                    self.tool = Tool::Bucket;
                }

                if ui.button("Up").clicked() { // TODO: Replace every text with icon
                    Self::shift_up(self);
                }
                if ui.button("Left").clicked() {
                    Self::shift_left(self);
                }
                if ui.button("Right").clicked() {
                    Self::shift_right(self);
                }
                if ui.button("Down").clicked() {
                    Self::shift_down(self);
                }

                if ui.button("Flip H").clicked() {
                    Self::flip_horizontally(self);
                }

                if ui.button("Flip V").clicked() {
                    Self::flip_vertically(self);
                }

                if ui.button("R90").clicked() {
                    Self::rotate_90_clockwise(self);
                }

                ui.label("test");
                ui.label("test");
            });

            if (response.clicked() || response.dragged() || response.secondary_clicked())
                && let Some(pointer_pos) = response.interact_pointer_pos()
            {
                let shade = if ui.input(|i| i.pointer.secondary_down()) || response.secondary_clicked() {
                    self.right_shade
                } else {
                    self.current_shade
                };

                match self.tool {
                    Tool::Draw => self.draw(origin, pointer_pos, shade),
                    Tool::Bucket => {
                        if response.clicked() || response.secondary_clicked() {
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
            }
            ui.horizontal(|ui| {
                ui.label("L");
                egui::Frame::default()
                    .fill(Self::shade_color(self.current_shade))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(50)))
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        let text_color = if self.current_shade <= 1 { Color32::BLACK } else { Color32::WHITE };
                        ui.label(egui::RichText::new(self.current_shade.to_string()).color(text_color));
                    });

                ui.label("R");
                egui::Frame::default()
                    .fill(Self::shade_color(self.right_shade))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(50)))
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        let text_color = if self.right_shade <= 1 { Color32::BLACK } else { Color32::WHITE };
                        ui.label(egui::RichText::new(self.right_shade.to_string()).color(text_color));
                    });


                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    for shade in 0u8..4 {
                        let text_color = if shade <= 1 { Color32::BLACK } else { Color32::WHITE };
                        let selected = self.current_shade == shade;
                        let right_selected = self.right_shade == shade;
                        let stroke = if selected {
                            egui::Stroke::new(2.0, egui::Color32::BLUE)
                        } else if right_selected {
                            egui::Stroke::new(2.0, egui::Color32::RED)
                        } else {
                            egui::Stroke::NONE
                        };
                        let button = egui::Button::new(egui::RichText::new(shade.to_string()).color(text_color))
                            .fill(Self::shade_color(shade))
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
            });
        });
    }
}
