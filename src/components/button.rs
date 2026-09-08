use gpui::*;

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
        .flex()
        .items_center()
        .justify_center()
        .px_3()
        .py_1()
        .bg(gb_light_green)
        .text_color(gb_dark_text)
        .font_family("Pixter-Display")
        .line_height(relative(1.0))
        .rounded(px(6.0))
        .cursor_pointer()
        .hover(|style| style.bg(gb_hover_green).text_color(gb_light_green))
        .active(|style| style.bg(gb_dark_green))
        .on_mouse_down(MouseButton::Left, cx.listener(move |this, event, _window, cx| {
            on_click(this, event, cx);
        }))
        .child(label)
}
