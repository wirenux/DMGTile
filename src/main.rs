use eframe::egui;

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

struct DMGTile {
    pixels: [u8; 64],
    previous_pixels: Option<usize>,
    dirty: bool,
    texture: Option<egui::TextureHandle>,
}

impl Default for DMGTile {
    fn default() -> Self {
        Self {
            pixels: [0; 64],
            previous_pixels: None,
            dirty: true, // force first-frame render
            texture: None,
        }
    }
}

impl DMGTile {
    fn rebuild_texture(&mut self, ctx: &egui::Context) {
        let mut image = egui::ColorImage::new(
            [GRID_SIZE, GRID_SIZE],
            vec![egui::Color32::BLACK; GRID_SIZE * GRID_SIZE],
        );

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let index = row * GRID_SIZE + col;
                let shade = self.pixels[index];
                let color = match shade {
                    0 => egui::Color32::from_gray(255),
                    1 => egui::Color32::from_gray(170),
                    2 => egui::Color32::from_gray(85),
                    _ => egui::Color32::from_gray(0),
                };
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

            if (response.clicked() || response.dragged())
                && let Some(pointer_pos) = response.interact_pointer_pos()
            {
                if ui.input(|i| i.pointer.secondary_down()) || response.secondary_clicked() {
                    self.draw(origin, pointer_pos, 0); // right click
                } else {
                    self.draw(origin, pointer_pos, 2); // left click
                }
            }
        });
    }
}
