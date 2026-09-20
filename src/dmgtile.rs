use gpui::*;
use gpui::prelude::FluentBuilder;
use std::path::PathBuf;
use std::time::Instant;

use crate::actions::{Brush, Bucket, Copy, Cut, Eraser, FlipH, FlipV, NewFile, OpenFile, Paste,
    Redo, Rotate, Save, SaveAs, ShiftDown, ShiftLeft, ShiftRight, ShiftUp, ToastDev, ToastShiftDev, Undo, EraseTile, ExportC, ExportBin, OpenGithub, OpenStardance
};
use crate::images::TileImageCache;
use crate::palette::{shade_color, Palette};
use crate::tools::{Snapshot, Tool};
use crate::components::corner_notch;

pub const MAX_TILES: usize = 128;
pub const CELL_SIZE: f32 = 32.0;
pub const GRID_SIZE: usize = 8;
pub const PREVIEW_PIXEL_SIZE: f32 = 12.0;
pub const PATTERN_PIXEL_SIZE: f32 = 6.0;
pub const PATTERN_REPEAT: usize = 4;



pub enum MenuEntry {
    Separator,
    Action {
        label: &'static str,
        make: fn() -> Box<dyn Action>,
    },
}

pub struct MenuDef {
    pub name: &'static str,
    pub entries: Vec<MenuEntry>,
}

pub struct Toast {
    pub message: String,
    pub is_error: bool,
    pub spawn_time: Instant,
}

pub struct DMGTile {
    pub tiles: Vec<[u8; 64]>,
    pub current_tile: usize,
    pub previous_pixels: Option<usize>,
    pub current_shade: u8,
    pub tool: Tool,
    pub palette: Palette,
    pub undo_stack: Vec<Snapshot>,
    pub redo_stack: Vec<Snapshot>,
    pub stroke_in_progress: bool,
    pub modified: Vec<bool>,
    pub current_path: Option<PathBuf>,
    pub clipboard: Option<[u8; 64]>,
    pub toast: Option<Toast>,
    pub focus_handle: FocusHandle,
    pub eraser_active: bool,
    pub pattern_image_cache: Option<TileImageCache>,
    pub preview_image_cache: Option<TileImageCache>,
    pub tile_thumb_cache: Vec<Option<TileImageCache>>,
    pub dev_mode: bool,
    pub open_menu: Option<usize>,
    pub first_frame: bool,
}

fn menu_definitions() -> Vec<MenuDef> {
    use MenuEntry::*;
    vec![
        MenuDef {
            name: "File",
            entries: vec![
                Action { label: "New...",      make: || Box::new(NewFile) },
                Action { label: "Open...",     make: || Box::new(OpenFile) },
                Separator,
                Action { label: "Save",        make: || Box::new(Save) },
                Action { label: "Save As...",  make: || Box::new(SaveAs) },
                Separator,
                Action { label: "Export .bin", make: || Box::new(ExportBin) },
                Action { label: "Export .c",   make: || Box::new(ExportC) },
            ],
        },
        MenuDef {
            name: "Edit",
            entries: vec![
                Action { label: "Undo",            make: || Box::new(Undo) },
                Action { label: "Redo",            make: || Box::new(Redo) },
                Separator,
                Action { label: "Erase Current Tile", make: || Box::new(EraseTile) },
            ],
        },
        MenuDef {
            name: "Help",
            entries: vec![
                Action { label: "GitHub",    make: || Box::new(OpenGithub) },
                Action { label: "Stardance", make: || Box::new(OpenStardance) },
            ],
        },
    ]
}

impl DMGTile {
    pub fn new(cx: &mut Context<Self>) -> Self {
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
            dev_mode: std::env::args().any(|arg| arg == "--dev"),
            open_menu: None,
            first_frame: true,
        }
    }
    pub const MENU_BUTTON_W: f32 = 72.0;
    pub const MENU_BAR_H: f32 = 28.0;
    fn render_menu_bar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let defs = menu_definitions();

        div()
            .flex()
            .flex_row()
            .h(px(Self::MENU_BAR_H))
            .w_full()
            .bg(rgb(0x0c1418))
            .border_b_1()
            .border_color(rgb(0x1a262c))
            .children(defs.iter().enumerate().map(|(i, def)| {
                let open = self.open_menu == Some(i);
                div()
                    .id(("menu-btn", i))
                    .w(px(Self::MENU_BUTTON_W))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .bg(if open { rgb(0x1a262c) } else { rgb(0x0c1418) })
                    .text_color(rgb(0x88c070))
                    .hover(|s| s.bg(rgb(0x1a262c)))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                        this.open_menu = if this.open_menu == Some(i) { None } else { Some(i) };
                        cx.notify();
                    }))
                    .child(def.name)
            }))
    }

    fn render_menu_popup(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let i = self.open_menu?;
        let defs = menu_definitions();
        let def = defs.get(i)?;
        let left = i as f32 * Self::MENU_BUTTON_W;

        Some(
            div()
                .absolute()
                .top(px(Self::MENU_BAR_H))
                .left(px(left))
                .min_w(px(200.0))
                .bg(rgb(0x1a262c))
                .border_1()
                .border_color(rgb(0x323232))
                .flex()
                .flex_col()
                .py_1()
                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                    this.open_menu = None;
                    cx.notify();
                }))
                .children(def.entries.iter().enumerate().map(|(idx, entry)| match entry {
                    MenuEntry::Separator => div()
                        .h(px(1.0))
                        .bg(rgb(0x323232))
                        .my_1()
                        .into_any_element(),
                    MenuEntry::Action { label, make } => {
                        let make: fn() -> Box<dyn Action> = *make;
                        let label: &'static str = label;
                        div()
                            .id(("menu-item", i * 1000 + idx))
                            .px_3()
                            .py_1()
                            .text_color(rgb(0xe0f8d0))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x323232)))
                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
                                window.dispatch_action(make(), cx);
                                this.open_menu = None;
                                cx.notify();
                            }))
                            .child(label)
                            .into_any_element()
                    }
                }))
        )
    }
}

impl Render for DMGTile {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.first_frame {
            self.first_frame = false;
            window.focus(&self.focus_handle, cx);
        }
        let bg_color = rgb(0x08171c);
        let border_color = rgb(0x88C070);

        div()
            .key_context("DMGTile")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .font_family("Pixter-Display")
            .bg(bg_color)
            .when(cfg!(not(target_os = "macos")), |el| el.child(self.render_menu_bar(cx)))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_grow(1.0)
                    .min_h_0()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_grow(1.0)
                            .h_full()
                            .child(self.render_toolbar(cx))
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
                                                            .on_action(cx.listener(|this, _: &FlipH, _, cx| {
                                                                this.flip_horizontally();
                                                                cx.notify();
                                                            }))
                                                            .on_action(cx.listener(|this, _: &FlipV, _, cx| {
                                                                this.flip_vertically();
                                                                cx.notify();
                                                            }))
                                                            .on_action(cx.listener(|this, _: &Rotate, _, cx| {
                                                                this.rotate_90_clockwise();
                                                                cx.notify();
                                                            }))
                                                            .on_action(cx.listener(|this, _: &EraseTile, _, cx| {
                                                                this.erase_tile();
                                                                cx.notify();
                                                                this.set_toast("Tile Erased".to_string(), false, cx);
                                                            }))
                                                            .on_action(cx.listener(|this, _: &NewFile, _, cx| this.new_project(cx)))
                                                            .on_action(cx.listener(|this, _: &OpenFile, _, cx| this.open_project(cx)))
                                                            .on_action(cx.listener(|this, _: &Save, _, cx| this.save_project(cx)))
                                                            .on_action(cx.listener(|this, _: &SaveAs, _, cx| this.save_project_as(cx)))
                                                            .on_action(cx.listener(|this, _: &ExportBin, _, cx| this.export_bin(cx)))
                                                            .on_action(cx.listener(|this, _: &ExportC, _, cx| this.export_c(cx)))
                                                            .when(self.dev_mode, |el| {
                                                                el.on_action(cx.listener(|this, _: &ToastDev, _, cx| {
                                                                    this.set_toast("Test Toast".to_string(), true, cx);
                                                                    cx.notify();
                                                                }))
                                                                .on_action(cx.listener(|this, _: &ToastShiftDev, _, cx| {
                                                                    this.set_toast("Test Toast Shift".to_string(), false, cx);
                                                                    cx.notify();
                                                                }))
                                                            })
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
                                                                        let color = shade_color(shade, &self.palette);

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
                                            .child(corner_notch(bg_color, true, true, 2.0))
                                            .child(corner_notch(bg_color, true, false, 2.0))
                                            .child(corner_notch(bg_color, false, true, 2.0))
                                            .child(corner_notch(bg_color, false, false, 2.0)),
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
                            .w(px(80.0))
                            .h_full()
                            .bg(rgb(0x0c1418))
                            .border_l_1()
                            .border_color(rgb(0x1a262c))
                            .child(self.render_tile_list(cx)),
                    ),
            )
            .children(self.render_menu_popup(cx))
            .children(self.render_toast(window, cx))
    }
}
