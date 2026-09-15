use gpui::*;
use gpui::prelude::FluentBuilder;

use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Instant;
use std::sync::Arc;

use image::Rgba as ImageRgba;

#[path = "../components/mod.rs"]
mod components;

use components::gameboy_button;

const MAX_TILES: usize = 128;

const CELL_SIZE: f32 = 32.0;
const GRID_SIZE: usize = 8;

const PIXEL: f32 = 2.0;

const PREVIEW_PIXEL_SIZE: f32 = 12.0;
const PATTERN_PIXEL_SIZE: f32 = 6.0;
const PATTERN_REPEAT: usize = 4;
const TILE_THUMB_PIXEL: f32 = 1.5;

actions!(dmgtile, [Quit, NewFile, OpenFile, Save, Undo, Redo, Copy, Paste, Cut, ShowAbout, Eraser, Brush, Bucket, ShiftUp, ShiftDown, ShiftLeft, ShiftRight]);

fn set_app_menus(cx: &mut App) {
    cx.set_menus(vec![
        Menu {
            name: "DMGTile".into(),
            items: vec![
                MenuItem::action("About DMGTile", ShowAbout),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
            disabled: false,
        },
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New", NewFile),
                MenuItem::action("Open", OpenFile),
                MenuItem::action("Save", Save),
            ],
            disabled: false,
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::action("Undo", Undo),
                MenuItem::action("Redo", Redo),
            ],
            disabled: false,
        },
        Menu {
            name: "Help".into(),
            items: vec![MenuItem::action("About", ShowAbout)],
            disabled: false,
        },
    ]);
}

#[derive(Clone)]
struct TileImageCache {
    key: ([u8; 64], Palette, u32),
    image: Arc<RenderImage>,
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

#[derive(PartialEq, Eq, Clone, Copy)]
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
    pattern_image_cache: Option<TileImageCache>,
    preview_image_cache: Option<TileImageCache>,
    tile_thumb_cache: Vec<Option<TileImageCache>>,
}

impl DMGTile {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            tiles: vec![[0u8; 64]; MAX_TILES],
            current_tile: 0,
            previous_pixels: None,
            current_shade: 3,
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
            pattern_image_cache: None,
            preview_image_cache: None,
            tile_thumb_cache: vec![None; MAX_TILES],
        }
    }

    fn get_or_build_tile_image(
        cache: &mut Option<TileImageCache>,
        tile: [u8; 64],
        palette: Palette,
        pixel_scale: u32,
    ) -> Arc<RenderImage> {
        let key = (tile, palette, pixel_scale);
        if let Some(existing) = cache && existing.key == key {
            return existing.image.clone();
        }
        let image = Self::build_tile_render_image(tile, palette, pixel_scale);
        *cache = Some(TileImageCache { key, image: image.clone() });
        image
    }

    fn active_shade(&self) -> u8 {
        if self.eraser_active { 0 } else { self.current_shade }
    }

    // =======
    //  Tools
    // =======

    fn apply_tools(&mut self, index: usize, shade: u8)  -> bool{
        match self.tool {
            Tool::Draw => {
                let changed = self.paint_pixel(index, shade);
                self.previous_pixels = Some(index);
                changed
            }
            Tool::Bucket => self.bucket_fill(index, shade),
        }
    }

    fn bucket_fill(&mut self, start_index: usize, new_shade: u8) -> bool {
        let target_shade = self.tiles[self.current_tile][start_index];

        if target_shade == new_shade {
            return false; // already the same shade
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
        true
    }

    
    fn paint_pixel(&mut self, index: usize, shade: u8) -> bool {
        if self.tiles[self.current_tile][index] != shade {
            self.tiles[self.current_tile][index] = shade;
            self.modified[self.current_tile] = true;
            true
        } else {
            false
        }
    }

    fn start_stroke(&mut self) {
        if !self.stroke_in_progress {
            self.push_undo();
            self.stroke_in_progress = true;
        }
    }

    fn end_stroke(&mut self) {
        self.stroke_in_progress = false;
        self.previous_pixels = None;
    }

    fn copy_tile(&mut self) {
        self.clipboard = Some(self.tiles[self.current_tile]);
    }

    fn cut_tile(&mut self) {
        self.copy_tile();
        self.push_undo();
        self.tiles[self.current_tile] = [0u8; 64];
        self.modified[self.current_tile] = true;
    }

    fn paste_tile(&mut self) {
        if let Some(data) = self.clipboard {
            self.push_undo();
            self.tiles[self.current_tile] = data;
            self.modified[self.current_tile] = true;
        }
    }

    fn shift_up(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            let source_row = (row + 1) % GRID_SIZE;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][source_row * GRID_SIZE + col];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    fn shift_down(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            let source_row = (row + GRID_SIZE - 1) % GRID_SIZE;
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][source_row * GRID_SIZE + col];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    fn shift_left(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let source_col = (col + 1) % GRID_SIZE;
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][row * GRID_SIZE + source_col];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    fn shift_right(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let source_col = (col + GRID_SIZE - 1) % GRID_SIZE;
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][row * GRID_SIZE + source_col];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    fn rotate_90_clockwise(&mut self) {
        self.push_undo();
        let mut new_pixels = [0u8; 64];
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                new_pixels[row * GRID_SIZE + col] = self.tiles[self.current_tile][(GRID_SIZE - 1 - col) * GRID_SIZE + row];
            }
        }
        self.tiles[self.current_tile] = new_pixels;
        self.modified[self.current_tile] = true;
    }

    fn corner_notch(color: Rgba, top: bool, left: bool, multiplicator: f32) -> impl IntoElement {
        div()
            .absolute()
            .w(px(PIXEL * multiplicator))
            .h(px(PIXEL * multiplicator))
            .bg(color)
            .when(top, |s| s.top(px(0.)))
            .when(!top, |s| s.bottom(px(0.)))
            .when(left, |s| s.left(px(0.)))
            .when(!left, |s| s.right(px(0.)))
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

    fn shade_rgb_bytes(shade: u8, palette: &Palette) -> (u8, u8, u8) {
        match palette {
            Palette::Grayscale => match shade {
                0 => (0xff, 0xff, 0xff),
                1 => (0xaa, 0xaa, 0xaa),
                2 => (0x55, 0x55, 0x55),
                _ => (0x00, 0x00, 0x00),
            },
            Palette::ClassicGreen => match shade {
                0 => (0xe0, 0xf8, 0xd0),
                1 => (0x88, 0xc0, 0x70),
                2 => (0x34, 0x68, 0x56),
                _ => (0x08, 0x18, 0x20),
            },
        }
    }

    fn build_tile_render_image(tile: [u8; 64], palette: Palette, pixel_scale: u32) -> Arc<RenderImage> {
        let dim = GRID_SIZE as u32 * pixel_scale;
        let mut buffer = image::RgbaImage::new(dim, dim);

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let shade = tile[row * GRID_SIZE + col];
                let (r, g, b) = Self::shade_rgb_bytes(shade, &palette);
                let pixel = ImageRgba([b, g, r, 255]);

                for dy in 0..pixel_scale {
                    for dx in 0..pixel_scale {
                        let x = col as u32 * pixel_scale + dx;
                        let y = row as u32 * pixel_scale + dy;
                        buffer.put_pixel(x, y, pixel);
                    }
                }
            }
        }

        let frame = image::Frame::new(buffer);
        Arc::new(RenderImage::new(vec![frame]))
    }

    fn render_tile_list(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let thumb_size = px(GRID_SIZE as f32 * TILE_THUMB_PIXEL);

        div()
            .id("tile-list")
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .bg(rgb(0x08171C))
            .children((0..MAX_TILES).map(|i| {
                let selected = self.current_tile == i;
                let image = Self::get_or_build_tile_image(
                    &mut self.tile_thumb_cache[i],
                    self.tiles[i],
                    self.palette,
                    2,
                );

                div()
                    .id(("tile-thumb", i))
                    .flex()
                    .items_center()
                    .justify_center()
                    .p_1()
                    .border_2()
                    .border_color(if selected { rgb(0x88C070) } else { rgba(0x00000000) })
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.current_tile = i;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .pt_1()
                            .pr_2()
                            .text_color(if selected { rgb(0xffffff) } else { rgb(0x88c070) })
                            .child(format!("{}", i))
                    )
                    .child(img(image).w(thumb_size * 2.0).h(thumb_size * 2.0))
            }))
    }

    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let shade = self.active_shade();
        let shade_color = Self::shade_color(shade, &self.palette);

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .pl_5()
            // Current shade indicator
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(div().text_color(rgb(0x86C06C)).child("L")) // TODO: Change letter
                    .child(
                        div()
                            .size(px(20.0))
                            .relative()
                            .pt(px(2.0)) // make the number centered with the Pixter-Display font
                            .bg(shade_color)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_color(if shade <= 1 { rgb(0x000000) } else { rgb(0xffffff) })
                                    .child(shade.to_string()),
                            )
                            .child(Self::corner_notch(rgb(0x1a1a1a), true, true, 1.0))
                            .child(Self::corner_notch(rgb(0x1a1a1a), true, false, 1.0))
                            .child(Self::corner_notch(rgb(0x1a1a1a), false, true, 1.0))
                            .child(Self::corner_notch(rgb(0x1a1a1a), false, false, 1.0))
                    ),
            )
            // Separator
            .child(div().w(px(1.0)).h(px(20.0)).bg(rgb(0x323232)))
            // Shade selector
            .child(
                div()
                    .flex()
                    .flex_row()
                    .children((0u8..4).map(|s| {
                        let color = Self::shade_color(s, &self.palette);
                        let selected = self.current_shade == s;
                        let text_color = if s <= 1 { rgb(0x000000) } else { rgb(0xffffff) };
                        let behind_bg = rgb(0x1a1a1a);
                        let border_color = if selected {
                            Self::shade_color(if s < 1 { 1 } else { 0 }, &self.palette)
                        } else {
                            rgba(0x08171cff)
                        };

                        div()
                            .id(("shade", s as usize))
                            .relative()
                            .size(px(24.0))
                            .pt(px(2.0))
                            .bg(color)
                            .border_2()
                            .border_color(border_color)
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
                            .when(!selected, |el| {
                                el.child(Self::corner_notch(behind_bg, true, true, 1.0))
                                    .child(Self::corner_notch(behind_bg, true, false, 1.0))
                                    .child(Self::corner_notch(behind_bg, false, true, 1.0))
                                    .child(Self::corner_notch(behind_bg, false, false, 1.0))
                            })
                    }))
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
                        gameboy_button("btn-palette-gray", "Gray", cx, |this, _event, cx| {
                            this.palette = Palette::Grayscale;
                            cx.notify();
                        })
                    )
                    .child(
                        gameboy_button("btn-palette-green", "Green", cx, |this, _event, cx| {
                            this.palette = Palette::ClassicGreen;
                            cx.notify();
                        })
                    ),
            )
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
        }
    }

    fn render_pattern_chunk(&mut self) -> impl IntoElement {
        let tile_px = px(GRID_SIZE as f32 * PATTERN_PIXEL_SIZE);
        let image = Self::get_or_build_tile_image(
            &mut self.pattern_image_cache,
            self.tiles[self.current_tile],
            self.palette,
            PATTERN_PIXEL_SIZE as u32,
        );

        div()
            // .top(px(128.0))
            .flex()
            .flex_col()
            .children((0..PATTERN_REPEAT).map(move |_| {
                let image = image.clone();
                div()
                    .flex()
                    .flex_row()
                    .children((0..PATTERN_REPEAT).map(move |_| {
                        img(image.clone()).w(tile_px).h(tile_px)
                    }))
            }))
    }

    fn render_preview_panel(&mut self) -> impl IntoElement {
        let chunk_size = px(PATTERN_REPEAT as f32 * GRID_SIZE as f32 * PATTERN_PIXEL_SIZE);
        let preview_size = px(GRID_SIZE as f32 * PREVIEW_PIXEL_SIZE);
        let bg_color = rgb(0x08171c);
        let border_color = rgb(0x88C070);

        let preview_image = Self::get_or_build_tile_image(
            &mut self.preview_image_cache,
            self.tiles[self.current_tile],
            self.palette,
            PREVIEW_PIXEL_SIZE as u32,
        );

        div()
            .pl(px(128.0))
            .flex()
            .flex_col()
            .items_center()
            .gap_4()
            .child(
                div()
                    .relative()
                    .child(
                        div()
                            .border_4()
                            .border_color(border_color)
                            .child(img(preview_image).w(preview_size).h(preview_size)),
                    )
                    .child(Self::corner_notch(bg_color, true, true, 2.0))
                    .child(Self::corner_notch(bg_color, true, false, 2.0))
                    .child(Self::corner_notch(bg_color, false, true, 2.0))
                    .child(Self::corner_notch(bg_color, false, false, 2.0)),
            )
            .child(
                div()
                    .relative()
                    .child(
                        div()
                            .border_4()
                            .border_color(border_color)
                            .child(self.render_pattern_chunk()),
                    )
                    .child(Self::corner_notch(bg_color, true, true, 2.0))
                    .child(Self::corner_notch(bg_color, true, false, 2.0))
                    .child(Self::corner_notch(bg_color, false, true, 2.0))
                    .child(Self::corner_notch(bg_color, false, false, 2.0)),
            )
    }
}

impl Render for DMGTile {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg_color = rgb(0x08171c);
        let border_color = rgb(0x88C070);

        div()
            .key_context("DMGTile")
            .size_full()
            .flex()
            .flex_row()
            .font_family("Pixter-Display")
            .bg(bg_color)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_grow(1.0)
                    .h_full()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .p_4()
                            .gap_4()
                            .child(
                                div()
                                    .relative()
                                    .child(
                                        div()
                                            .border_4()
                                            .border_color(border_color)
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
                                                    .on_action(cx.listener(|this, _: &Undo, _, cx| {
                                                        this.undo();
                                                        cx.notify();
                                                    }))
                                                    .on_action(cx.listener(|this, _: &Redo, _, cx| {
                                                        this.redo();
                                                        cx.notify();
                                                    }))
                                                    .on_action(cx.listener(|this, _: &Copy, _, cx| {
                                                        this.copy_tile();
                                                        cx.notify();
                                                    }))
                                                    .on_action(cx.listener(|this, _: &Cut, _, cx| {
                                                        this.cut_tile();
                                                        cx.notify();
                                                    }))
                                                    .on_action(cx.listener(|this, _: &Paste, _, cx| {
                                                        this.paste_tile();
                                                        cx.notify();
                                                    }))
                                                    .on_action(cx.listener(|this, _: &ShiftUp, _, cx| {
                                                        this.shift_up();
                                                        cx.notify();
                                                    }))
                                                    .on_action(cx.listener(|this, _: &ShiftDown, _, cx| {
                                                        this.shift_down();
                                                        cx.notify();
                                                    }))
                                                    .on_action(cx.listener(|this, _: &ShiftLeft, _, cx| {
                                                        this.shift_left();
                                                        cx.notify();
                                                    }))
                                                    .on_action(cx.listener(|this, _: &ShiftRight, _, cx| {
                                                        this.shift_right();
                                                        cx.notify();
                                                    }))
                                                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                                                        if event.keystroke.key == "r" && !event.is_held {
                                                            this.rotate_90_clockwise();
                                                            cx.notify();
                                                        }
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
                                                                    .size(px(CELL_SIZE + 16.0))
                                                                    .bg(color)
                                                                    .border(px(0.5))
                                                                    .border_color(rgb(0x323232))
                                                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                                                        this.start_stroke();
                                                                        let shade = this.active_shade();
                                                                        let changed = this.apply_tools(index, shade);
                                                                        this.previous_pixels = Some(index);
                                                                        if changed {
                                                                            cx.notify();
                                                                        }
                                                                    }))
                                                                    .on_mouse_move(cx.listener(move |this, _event: &MouseMoveEvent, _, cx| {
                                                                        if !this.stroke_in_progress || this.previous_pixels == Some(index) {
                                                                            return;
                                                                        }
                                                                        let shade = this.active_shade();
                                                                        let changed = this.apply_tools(index, shade);
                                                                        this.previous_pixels = Some(index);
                                                                        if changed {
                                                                            cx.notify();
                                                                        }
                                                                    }))
                                                            }))
                                                    })),
                                            ),
                                    )
                                    .child(Self::corner_notch(bg_color, true, true, 2.0))
                                    .child(Self::corner_notch(bg_color, true, false, 2.0))
                                    .child(Self::corner_notch(bg_color, false, true, 2.0))
                                    .child(Self::corner_notch(bg_color, false, false, 2.0)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .w(px(140.0))
                                    .h_full()
                                    .child(self.render_preview_panel()),
                            ),
                    )
                    .child(self.render_status_bar(cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    .w(px(64.0))
                    .h_full()
                    .bg(rgb(0x0c1418))
                    .border_l_1()
                    .border_color(rgb(0x1a262c))
                    .child(self.render_tile_list(cx)),
            )
    }
}

fn main() {
    Application::with_platform(gpui_platform::current_platform(false)).run(|cx: &mut App| {
        cx.activate(true); // focus on the app
        cx.set_cursor_hide_mode(CursorHideMode::Never); // prevent mouse flicker when undo/redo

        let font_bytes = include_bytes!("../../font/Pixter-Display.ttf").to_vec(); // TODO: when release change path to the good one

        cx.text_system()
            .add_fonts(vec![Cow::Owned(font_bytes)])
            .expect("Failed to load custom font");

        cx.on_action(|_: &Quit, cx| { cx.quit(); });
        cx.on_action(|_: &NewFile, _cx| { println!("New"); });
        cx.on_action(|_: &OpenFile, _cx| { println!("Open"); });
        cx.on_action(|_: &Save, _cx| { println!("Save"); });
        cx.on_action(|_: &ShowAbout, _cx| { println!("About"); });

        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, Some("DMGTile")), // faster quit than using the traffic light
            KeyBinding::new("cmd-z", Undo, Some("DMGTile")),
            KeyBinding::new("cmd-shift-z", Redo, Some("DMGTile")),
            KeyBinding::new("e", Eraser, Some("DMGTile")),
            KeyBinding::new("b", Brush, Some("DMGTile")),
            KeyBinding::new("g", Bucket, Some("DMGTile")),
            KeyBinding::new("up", ShiftUp, Some("DMGTile")),
            KeyBinding::new("down", ShiftDown, Some("DMGTile")),
            KeyBinding::new("left", ShiftLeft, Some("DMGTile")),
            KeyBinding::new("right", ShiftRight, Some("DMGTile")),
            KeyBinding::new("cmd-c", Copy, Some("DMGTile")),
            KeyBinding::new("cmd-x", Cut, Some("DMGTile")),
            KeyBinding::new("cmd-v", Paste, Some("DMGTile")),
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
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle, cx);
            })
            .ok();
    });
}
