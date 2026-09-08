use gpui::*;

use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Instant;

const MAX_TILES: usize = 128;

const CELL_SIZE: f32 = 32.0;
const GRID_SIZE: usize = 8;

actions!(dmgtile, [Quit, NewFile, OpenFile, Save, Undo, Redo, ShowAbout, Eraser, Brush, Bucket]);

fn set_app_menus(cx: &mut App) {
    cx.set_menus(vec![
        Menu {
            name: "DMGTile".into(),
            items: vec![
                MenuItem::action("About DMGTile", ShowAbout),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
        },
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New", NewFile),
                MenuItem::action("Open", OpenFile),
                MenuItem::action("Save", Save),
            ],
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::action("Undo", Undo),
                MenuItem::action("Redo", Redo),
            ],
        },
        Menu {
            name: "Help".into(),
            items: vec![MenuItem::action("About", ShowAbout)],
        },
    ]);
}

pub enum Event {
    Copy,
    Cut,
    Paste(String),
}

enum Tool {
    Draw,
    Bucket,
}

enum Palette {
    Grayscale,
    ClassicGreen,
}

struct Toast {
    message: String,
    is_error: bool,
    spawn_time: Instant,
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
    current_shade: u8,
    right_shade: u8,
    tool: Tool,
    palette: Palette,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    stroke_in_progress: bool,
    modified: Vec<bool>,
    current_path: Option<PathBuf>,
    clipboard: Option<[u8; 64]>,
    toast: Option<Toast>,
    focus_handle: FocusHandle,
    eraser_active: bool,
}

impl DMGTile {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            tiles: vec![[0u8; 64]; MAX_TILES],
            current_tile: 0,
            previous_pixels: None,
            current_shade: 3,
            right_shade: 0,
            tool: Tool::Draw,
            palette: Palette::ClassicGreen,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            stroke_in_progress: false,
            modified: vec![false; MAX_TILES],
            current_path: None,
            clipboard: None,
            toast: None,
            focus_handle: cx.focus_handle(),
            eraser_active: false,
        }
    }

    fn active_shade(&self) -> u8 {
        if self.eraser_active { 0 } else { self.current_shade }
    }

    // =======
    //  Tools
    // =======

    fn apply_tools(&mut self, index: usize, shade: u8) {
        match self.tool {
            Tool::Draw => {
                self.paint_pixel(index, shade);
                self.previous_pixels = Some(index);
            }
            Tool::Bucket => {
                self.bucket_fill(index, shade);
            }
        }
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

        self.modified[self.current_tile] = true;
    }

    
    fn paint_pixel(&mut self, index: usize, shade: u8) {
        if self.tiles[self.current_tile][index] != shade {
            self.tiles[self.current_tile][index] = shade;
            self.modified[self.current_tile] = true;
        }
    }

    fn start_stroke(&mut self) {
        if !self.stroke_in_progress {
            // self.push_undo();
            self.stroke_in_progress = true;
        }
    }

    fn end_stroke(&mut self) {
        self.stroke_in_progress = false;
        self.previous_pixels = None;
    }

    fn shade_color(shade: u8, palette: &Palette) -> Rgba {
        match palette {
            Palette::Grayscale => match shade {
                0 => rgb(0xffffff),
                1 => rgb(0xaaaaaa),
                2 => rgb(0x555555),
                _ => rgb(0x000000),
            },
            Palette::ClassicGreen => match shade {
                0 => rgb(0xe0f8d0),
                1 => rgb(0x88c070),
                2 => rgb(0x346856),
                _ => rgb(0x081820),
            },
        }
    }

    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let shade = self.active_shade();
        let shade_color = Self::shade_color(shade, &self.palette);

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .p_1()
            // Current shade indicator
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(div().text_color(rgb(0xffffff)).child("L"))
                    .child(
                        div()
                            .size(px(20.0))
                            .pl(px(4.0)) // make the number centered with the Early GameBoy font
                            .bg(shade_color)
                            .border_1()
                            .border_color(rgb(0x323232))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_color(if shade <= 1 { rgb(0x000000) } else { rgb(0xffffff) })
                                    .child(shade.to_string()),
                            ),
                    ),
            )
            // Separator
            .child(div().w(px(1.0)).h(px(20.0)).bg(rgb(0x323232)))
            // Shade selector (4 swatches)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .children((0u8..4).map(|s| {
                        let color = Self::shade_color(s, &self.palette);
                        let selected = self.current_shade == s;
                        let text_color = if s <= 1 { rgb(0x000000) } else { rgb(0xffffff) };

                        div()
                            .id(("shade", s as usize))
                            .size(px(24.0))
                            .pl(px(4.0)) // make the number centered with the Early GameBoy font
                            .bg(color)
                            .border_1()
                            .border_color(if selected { rgb(0x3080ff) } else { rgb(0x323232) })
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                this.current_shade = s;
                                this.eraser_active = false;
                                cx.notify();
                            }))
                            .child(div().text_color(text_color).child(s.to_string()))
                    })),
            )
            // Separator
            .child(div().w(px(1.0)).h(px(20.0)).bg(rgb(0x323232)))
            // Palette selector
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .child(
                        div()
                            .id("palette-gray")
                            .px_2()
                            .py_1()
                            .border_1()
                            .border_color(rgb(0x323232))
                            .text_color(rgb(0xffffff))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.palette = Palette::Grayscale;
                                cx.notify();
                            }))
                            .child("Gray"),
                    )
                    .child(
                        div()
                            .id("palette-green")
                            .px_2()
                            .py_1()
                            .border_1()
                            .border_color(rgb(0x323232))
                            .text_color(rgb(0xffffff))
                            .cursor_pointer()
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                this.palette = Palette::ClassicGreen;
                                cx.notify();
                            }))
                            .hover(|style| style.bg(rgb(0xffffff)))
                            .child("Green"),
                    ),
            )
    }
}

impl Render for DMGTile {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("DMGTile")
            .size_full()
            .flex()
            .flex_col()
            .font_family("Early GameBoy")
            .bg(rgb(0x0d1221))
            .child(
                div()
                    .track_focus(&self.focus_handle)
                    .flex()
                    .flex_col()
                    .on_action(cx.listener(|this, _: &Eraser, _, cx| {
                        this.tool = Tool::Draw;
                        this.eraser_active = true;
                        cx.notify();
                    }))
                    .on_action(cx.listener(|this, _: &Brush, _, cx| {
                        this.tool = Tool::Draw;
                        this.eraser_active = false;
                        cx.notify();
                    }))
                    .on_action(cx.listener(|this, _: &Bucket, _, cx| {
                        this.tool = Tool::Bucket;
                        this.eraser_active = false;
                        cx.notify();
                    }))
                    .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.end_stroke();
                        cx.notify();
                    }))
                    .on_mouse_up_out(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.end_stroke();
                        cx.notify();
                    }))
                    .children((0..GRID_SIZE).map(|row| {
                        div()
                            .flex()
                            .flex_row()
                            .children((0..GRID_SIZE).map(|col| {
                                let index = row * GRID_SIZE + col;
                                let shade = self.tiles[self.current_tile][index];
                                let color = Self::shade_color(shade, &self.palette);

                                div()
                                    .id(("pixel", index))
                                    .size(px(CELL_SIZE))
                                    .bg(color)
                                    .border(px(0.5))
                                    .border_color(rgb(0x323232))
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                        this.start_stroke();
                                        let shade = this.active_shade();
                                        this.apply_tools(index, shade);
                                        this.previous_pixels = Some(index);
                                        cx.notify();
                                    }))
                                    .on_mouse_move(cx.listener(move |this, _event: &MouseMoveEvent, _, cx| {
                                        if !this.stroke_in_progress || this.previous_pixels == Some(index) {
                                            return;
                                        }
                                        let shade = this.active_shade();
                                        this.apply_tools(index, shade);
                                        this.previous_pixels = Some(index);
                                        cx.notify();
                                    }))
                            }))
                    })),
            )
            .child(self.render_status_bar(cx))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.activate(true); // focus on the app

        let font_bytes = include_bytes!("../../font/gb.ttf").to_vec(); // TODO: when release change
                                                                       // path to the good one

        cx.text_system()
            .add_fonts(vec![Cow::Owned(font_bytes)])
            .expect("Failed to load custom font");

        cx.on_action(|_: &Quit, cx| { cx.quit(); });
        cx.on_action(|_: &NewFile, _cx| { println!("New"); });
        cx.on_action(|_: &OpenFile, _cx| { println!("Open"); });
        cx.on_action(|_: &Save, _cx| { println!("Save"); });
        cx.on_action(|_: &Undo, _cx| { println!("Undo"); });
        cx.on_action(|_: &Redo, _cx| { println!("Redo"); });
        cx.on_action(|_: &ShowAbout, _cx| { println!("About"); });

        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None), // faster quit than using the traffic light
            KeyBinding::new("e", Eraser, Some("DMGTile")),
            KeyBinding::new("b", Brush, Some("DMGTile")),
            KeyBinding::new("g", Bucket, Some("DMGTile")),
        ]);

        set_app_menus(cx);

        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(800.0), px(600.0)),
                    cx,
                ))),

                window_min_size: Some(size(px(400.0), px(300.0))),

                titlebar: Some(TitlebarOptions {
                    title: Some("DMGTile".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            |_, cx| cx.new(DMGTile::new),
        )
        .unwrap();

        window
            .update(cx, |view, window, _cx| {
                window.focus(&view.focus_handle);
            })
            .ok();
    });
}
