use gpui::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Palette {
    Grayscale,
    ClassicGreen,
}

pub fn shade_color(shade: u8, palette: &Palette) -> Rgba {
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

pub fn shade_rgb_bytes(shade: u8, palette: &Palette) -> (u8, u8, u8) {
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
