use gpui::*;
use std::borrow::Cow;

mod actions;
mod components;
mod dmgtile;
mod images;
mod palette;
mod panels;
mod tools;

use actions::{
    set_app_menus, Brush, Bucket, Copy, Cut, Eraser, NewFile, OpenFile, Paste,
    Quit, Redo, Save, ShiftDown, ShiftLeft, ShiftRight, ShiftUp, ShowAbout, ToastDev, ToastShiftDev,
    Undo,
};
use dmgtile::DMGTile;

use crate::actions::EraseTile;

fn main() {
    Application::with_platform(gpui_platform::current_platform(false)).run(|cx: &mut App| {
        cx.activate(true);
        cx.set_cursor_hide_mode(CursorHideMode::Never);

        let font_bytes = include_bytes!("../font/Pixter-Display.ttf").to_vec();

        cx.text_system()
            .add_fonts(vec![Cow::Owned(font_bytes)])
            .expect("Failed to load custom font");

        cx.on_action(|_: &Quit, cx| { cx.quit(); });
        cx.on_action(|_: &NewFile, _cx| { println!("New"); });
        cx.on_action(|_: &OpenFile, _cx| { println!("Open"); });
        cx.on_action(|_: &Save, _cx| { println!("Save"); });
        cx.on_action(|_: &ShowAbout, _cx| { println!("About"); });

        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-n", NewFile, None),
            KeyBinding::new("cmd-o", OpenFile, None),
            KeyBinding::new("cmd-s", Save, None),
            KeyBinding::new("cmd-z", Undo, None),
            KeyBinding::new("cmd-shift-z", Redo, None),
            KeyBinding::new("cmd-c", Copy, None),
            KeyBinding::new("cmd-x", Cut, None),
            KeyBinding::new("cmd-v", Paste, None),
            KeyBinding::new("cmd-backspace", EraseTile, None),

            KeyBinding::new("cmd-t", ToastDev, None),
            KeyBinding::new("cmd-shift-t", ToastShiftDev, None),

            KeyBinding::new("e", Eraser, Some("DMGTile")),
            KeyBinding::new("b", Brush, Some("DMGTile")),
            KeyBinding::new("g", Bucket, Some("DMGTile")),
            KeyBinding::new("up", ShiftUp, Some("DMGTile")),
            KeyBinding::new("down", ShiftDown, Some("DMGTile")),
            KeyBinding::new("left", ShiftLeft, Some("DMGTile")),
            KeyBinding::new("right", ShiftRight, Some("DMGTile")),
        ]);

        set_app_menus(cx);

        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(800.0), px(600.0)),
                    cx,
                ))),
                window_min_size: Some(size(px(750.0), px(525.0))),
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
