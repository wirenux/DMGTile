use gpui::prelude::FluentBuilder;
use gpui::*;

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

pub fn corner_notch(color: Rgba, top: bool, left: bool, multiplicator: f32) -> impl IntoElement {
    notch(color, PIXEL * multiplicator, PIXEL * multiplicator, top, left)
}

pub fn staircase_corner(color: Rgba, top: bool, left: bool) -> [AnyElement; 2] {
    [
        notch(color, PIXEL * 2.0, PIXEL, top, left).into_any_element(),
        notch(color, PIXEL, PIXEL * 2.0, top, left).into_any_element(),
    ]
}
