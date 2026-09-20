use gpui::prelude::FluentBuilder;
use gpui::*;
use std::ops::Range;
use std::time::Instant;

use crate::components::gameboy_button;
use crate::images::get_or_build_tile_image;
use crate::palette::{shade_color, Palette};
use crate::tools::Tool;
use crate::components::{corner_notch, gameboy_icon_button};
use crate::dmgtile::{
    DMGTile, Toast, GRID_SIZE, MAX_TILES, PATTERN_PIXEL_SIZE, PATTERN_REPEAT, PREVIEW_PIXEL_SIZE,
};

impl DMGTile {
    pub fn render_toolbar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_pen_selected = matches!(self.tool, Tool::Draw) && !self.eraser_active;
        let is_eraser_selected = matches!(self.tool, Tool::Draw) && self.eraser_active;
        let is_bucket_selected = matches!(self.tool, Tool::Bucket);

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .p_2()
            .bg(rgb(0x08171C))
            .border_b_1()
            .border_color(rgb(0x1a262c))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .child(gameboy_icon_button("btn-pen", "pen.png", is_pen_selected, cx, |this, _, _, cx| {
                        this.tool = Tool::Draw;
                        this.eraser_active = false;
                        cx.notify();
                    }))
                    .child(gameboy_icon_button("btn-eraser", "eraser.png", is_eraser_selected, cx, |this, _, _, cx| {
                        this.tool = Tool::Draw;
                        this.eraser_active = true;
                        cx.notify();
                    }))
                    .child(gameboy_icon_button("btn-bucket", "bucket.png", is_bucket_selected, cx, |this, _, _, cx| {
                        this.tool = Tool::Bucket;
                        this.eraser_active = false;
                        cx.notify();
                    })),
            )
            .child(div().w(px(1.0)).h(px(24.0)).bg(rgb(0x323232)))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .child(gameboy_icon_button("btn-shift-left", "left.png", false, cx, |this, _, _, cx| {
                        this.shift_left();
                        cx.notify();
                    }))
                    .child(gameboy_icon_button("btn-shift-right", "right.png", false, cx, |this, _, _, cx| {
                        this.shift_right();
                        cx.notify();
                    }))
                    .child(gameboy_icon_button("btn-shift-up", "up.png", false, cx, |this, _, _, cx| {
                        this.shift_up();
                        cx.notify();
                    }))
                    .child(gameboy_icon_button("btn-shift-down", "down.png", false, cx, |this, _, _, cx| {
                        this.shift_down();
                        cx.notify();
                    })),
            )
            .child(div().w(px(1.0)).h(px(24.0)).bg(rgb(0x323232)))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .child(gameboy_icon_button("btn-flip-h", "flipH.png", false, cx, |this, _, _, cx| {
                        this.flip_vertically();
                        cx.notify();
                    }))
                    .child(gameboy_icon_button("btn-flip-v", "flipV.png", false, cx, |this, _, _, cx| {
                        this.flip_horizontally();
                        cx.notify();
                    }))
                    .child(gameboy_icon_button("btn-rotate", "rotate.png", false, cx, |this, _, _, cx| {
                        this.rotate_90_clockwise();
                        cx.notify();
                    })),
            )
    }

    pub fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let shade = self.active_shade();
        let active_shade_color = shade_color(shade, &self.palette);

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .pl_5()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(div().text_color(rgb(0x86C06C)).child("L"))
                    .child(
                        div()
                            .size(px(20.0))
                            .relative()
                            .when(!cfg!(target_os = "windows"), |el| el.pt(px(2.0)))
                            .bg(active_shade_color)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_color(if shade <= 1 { rgb(0x000000) } else { rgb(0xffffff) })
                                    .child(shade.to_string()),
                            )
                            .child(corner_notch(rgb(0x1a1a1a), true, true, 1.0))
                            .child(corner_notch(rgb(0x1a1a1a), true, false, 1.0))
                            .child(corner_notch(rgb(0x1a1a1a), false, true, 1.0))
                            .child(corner_notch(rgb(0x1a1a1a), false, false, 1.0)),
                    ),
            )
            .child(div().w(px(1.0)).h(px(20.0)).bg(rgb(0x323232)))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .children((0u8..4).map(|s| {
                        let color = shade_color(s, &self.palette);
                        let selected = self.current_shade == s;
                        let text_color = if s <= 1 { rgb(0x000000) } else { rgb(0xffffff) };
                        let behind_bg = rgb(0x1a1a1a);
                        let border_color = if selected {
                            shade_color(if s < 1 { 1 } else { 0 }, &self.palette)
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
                                el.child(corner_notch(behind_bg, true, true, 1.0))
                                    .child(corner_notch(behind_bg, true, false, 1.0))
                                    .child(corner_notch(behind_bg, false, true, 1.0))
                                    .child(corner_notch(behind_bg, false, false, 1.0))
                            })
                    })),
            )
            .child(div().w(px(1.0)).h(px(20.0)).bg(rgb(0x323232)))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .child(gameboy_button("btn-palette-gray", "Gray", cx, |this, _event, cx| {
                        this.palette = Palette::Grayscale;
                        cx.notify();
                    }))
                    .child(gameboy_button("btn-palette-green", "Green", cx, |this, _event, cx| {
                        this.palette = Palette::ClassicGreen;
                        cx.notify();
                    })),
            )
    }

    pub fn render_tile_list(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let row_height = px(40.0);
        let thumb_size = px(24.0);

        uniform_list(
            "tile-list",
            MAX_TILES,
            cx.processor(move |this, range: Range<usize>, _window, cx| {
                range.map(|i| {
                    let selected = this.current_tile == i;
                    let image = get_or_build_tile_image(
                        &mut this.tile_thumb_cache[i],
                        this.tiles[i],
                        this.palette,
                        2,
                        false, // w/o corners
                    );
                    div()
                        .id(("tile-thumb", i))
                        .w_full()
                        .h(row_height)
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_1()
                        .border_2()
                        .border_color(if selected { rgb(0x88C070) } else { rgba(0x00000000) })
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                            if this.current_tile != i {
                                this.current_tile = i;
                                cx.notify();
                            }
                        }))
                        .child(
                            div()
                                .w(px(32.0))
                                .flex_shrink_0()
                                .flex()
                                .justify_end()
                                .text_color(if selected { rgb(0xffffff) } else { rgb(0x88c070) })
                                .child(format!("{}", i)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_shrink_0()
                                .items_center()
                                .justify_center()
                                .child(img(image).w(thumb_size).h(thumb_size)),
                        )
                        .into_any_element()
                }).collect()
            }),
        )
        .w(px(80.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(rgb(0x08171C))
    }

    pub fn render_pattern_chunk(&mut self) -> impl IntoElement {
        let tile_px = px(GRID_SIZE as f32 * PATTERN_PIXEL_SIZE);
        let image = get_or_build_tile_image(
            &mut self.pattern_image_cache,
            self.tiles[self.current_tile],
            self.palette,
            PATTERN_PIXEL_SIZE as u32,
            false,
        );

        div()
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

    pub fn render_preview_panel(&mut self) -> impl IntoElement {
        let preview_size = px(GRID_SIZE as f32 * PREVIEW_PIXEL_SIZE);
        let bg_color = rgb(0x08171c);
        let border_color = rgb(0x88C070);

        let preview_image = get_or_build_tile_image(
            &mut self.preview_image_cache,
            self.tiles[self.current_tile],
            self.palette,
            PREVIEW_PIXEL_SIZE as u32,
            false,
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
                    .child(corner_notch(bg_color, true, true, 2.0))
                    .child(corner_notch(bg_color, true, false, 2.0))
                    .child(corner_notch(bg_color, false, true, 2.0))
                    .child(corner_notch(bg_color, false, false, 2.0)),
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
                    .child(corner_notch(bg_color, true, true, 2.0))
                    .child(corner_notch(bg_color, true, false, 2.0))
                    .child(corner_notch(bg_color, false, true, 2.0))
                    .child(corner_notch(bg_color, false, false, 2.0)),
            )
    }

    pub fn set_toast(&mut self, message: String, is_error: bool, cx: &mut Context<Self>) {
        self.toast = Some(Toast {
            message,
            is_error,
            spawn_time: Instant::now(),
        });
        cx.notify();
    }

    pub fn render_toast(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        let toast = self.toast.as_ref()?;

        if toast.spawn_time.elapsed().as_secs_f32() > 3.0 {
            self.toast = None;
            return None;
        }

        cx.on_next_frame(window, move |_: &mut Self, _: &mut Window, cx: &mut Context<Self>| {
            cx.notify();
        });

        let text_color = if toast.is_error {
            rgb(0xFF5555)
        } else {
            rgb(0x88C070)
        };

        let bg_color = rgb(0x08171C);

        Some(
            div()
                .absolute()
                .top(px(16.0))
                .right(px(16.0))
                .tab_index(100)
                .p_3()
                .bg(bg_color)
                .border_2()
                .border_color(text_color)
                .text_color(text_color)
                .child(toast.message.clone())
                .child(corner_notch(bg_color, true, true, 2.0))
                .child(corner_notch(bg_color, true, false, 2.0))
                .child(corner_notch(bg_color, false, true, 2.0))
                .child(corner_notch(bg_color, false, false, 2.0)),
        )
    }
}
