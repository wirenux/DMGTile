use gpui::*;
use gpui::prelude::FluentBuilder;

const PIXEL: f32 = 2.0;

fn notch(color: Rgba, w: f32, h: f32, top: bool, left: bool) -> impl IntoElement {
    div()
        .absolute()
        .w(px(w))
        .h(px(h))
        .bg(color)
        .when(top, |s| s.top(px(0.)))
        .when(!top, |s| s.bottom(px(0.)))
        .when(left, |s| s.left(px(0.)))
        .when(!left, |s| s.right(px(0.)))
}

fn staircase_corner(color: Rgba, top: bool, left: bool) -> [AnyElement; 2] {
    [
        notch(color, PIXEL * 2.0, PIXEL, top, left).into_any_element(),
        notch(color, PIXEL, PIXEL * 2.0, top, left).into_any_element(),
    ]
}

pub fn gameboy_button<V: Render>(
    id: &'static str,
    label: &'static str,
    cx: &mut Context<V>,
    on_click: impl Fn(&mut V, &MouseDownEvent, &mut Context<V>) + 'static,
) -> impl IntoElement {
    let gb_light_green = rgb(0x88c070);
    let gb_dark_text = rgb(0x081820);
    let gb_hover_green = rgb(0x306850);
    let gb_dark_green = rgb(0x071821);

    div()
        .id(id)
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .px_3()
        .py_1()
        .bg(gb_light_green)
        .text_color(gb_dark_text)
        .font_family("Pixter-Display")
        .line_height(relative(1.0))
        .rounded(px(0.0))
        .cursor_pointer()
        .hover(|style| style.bg(gb_hover_green).text_color(gb_light_green))
        .active(|style| style.bg(gb_dark_green))
        .on_mouse_down(MouseButton::Left, cx.listener(move |this, event, _window, cx| {
            on_click(this, event, cx);
        }))
        .children(staircase_corner(gb_dark_green, true, true))
        .children(staircase_corner(gb_dark_green, true, false))
        .children(staircase_corner(gb_dark_green, false, true))
        .children(staircase_corner(gb_dark_green, false, false))
        .child(label)
}
