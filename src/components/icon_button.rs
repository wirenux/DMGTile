use gpui::*;

use super::notch::corner_notch;
use crate::images::get_icon_image;

pub fn gameboy_icon_button<T: 'static>(
    id: impl Into<ElementId>,
    icon_filename: &'static str,
    selected: bool,
    cx: &mut Context<T>,
    on_click: impl Fn(&mut T, &MouseDownEvent, &mut Window, &mut Context<T>) + 'static,
) -> impl IntoElement {
    let bg_color = if selected { rgb(0x88C070) } else { rgb(0x08171C) };
    let behind_bg = rgb(0x08171C);

    let icon_image = get_icon_image(icon_filename);

    div()
        .id(id)
        .relative()
        .size(px(48.0))
        .flex()
        .items_center()
        .justify_center()
        .bg(bg_color)
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, cx.listener(move |this, event, window, cx| {
            on_click(this, event, window, cx);
        }))
        .child(img(icon_image).size(px(44.0)))
        .child(corner_notch(behind_bg, true, true, 2.0))
        .child(corner_notch(behind_bg, true, false, 2.0))
        .child(corner_notch(behind_bg, false, true, 2.0))
        .child(corner_notch(behind_bg, false, false, 2.0))
}
