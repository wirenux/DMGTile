use gpui::*;
use image::Rgba as ImageRgba;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::palette::{shade_rgb_bytes, Palette};
use crate::dmgtile::GRID_SIZE;

#[derive(Clone)]
pub struct TileImageCache {
    key: ([u8; 64], Palette, u32, bool),
    image: Arc<RenderImage>,
}

pub fn get_or_build_tile_image(
    cache: &mut Option<TileImageCache>,
    tile: [u8; 64],
    palette: Palette,
    pixel_scale: u32,
    with_corners: bool,
) -> Arc<RenderImage> {
    let key = (tile, palette, pixel_scale, with_corners);
    if let Some(existing) = cache && existing.key == key {
        return existing.image.clone();
    }
    let image = build_tile_render_image(tile, palette, pixel_scale, with_corners);
    *cache = Some(TileImageCache { key, image: image.clone() });
    image
}

fn build_tile_render_image(
    tile: [u8; 64],
    palette: Palette,
    pixel_scale: u32,
    with_corners: bool,
) -> Arc<RenderImage> {
    let dim = GRID_SIZE as u32 * pixel_scale;
    let mut buffer = image::RgbaImage::new(dim, dim);

    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let shade = tile[row * GRID_SIZE + col];
            let (r, g, b) = shade_rgb_bytes(shade, &palette);
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

    if with_corners {
        let bg_color = [0x08, 0x17, 0x1c, 0xff];
        let corner_size = pixel_scale;
        let corners = [
            (0, 0),
            (dim - corner_size, 0),
            (0, dim - corner_size),
            (dim - corner_size, dim - corner_size),
        ];
        for &(cx, cy) in &corners {
            for dy in 0..corner_size {
                for dx in 0..corner_size {
                    buffer.put_pixel(cx + dx, cy + dy, ImageRgba(bg_color));
                }
            }
        }
    }

    let frame = image::Frame::new(buffer);
    Arc::new(RenderImage::new(vec![frame]))
}

pub fn get_icon_image(icon_name: &'static str) -> Arc<RenderImage> {
    static CACHE: OnceLock<HashMap<&'static str, Arc<RenderImage>>> = OnceLock::new();

    let cache = CACHE.get_or_init(|| {
        let icons = [
            "pen.png", "eraser.png", "bucket.png",
            "left.png", "right.png", "up.png", "down.png",
            "flipH.png", "flipV.png", "rotate.png",
        ];

        let mut map = HashMap::new();
        let scale = 4u32;

        for name in icons {
            let full_path = format!("{}/assets/aseprite/gpui/{}", env!("CARGO_MANIFEST_DIR"), name); // TODO: Maybe change the folder when release build
            if let Ok(img_bytes) = std::fs::read(&full_path)
                && let Ok(decoded) = image::load_from_memory(&img_bytes)
            {
                let rgba = decoded.to_rgba8();
                let orig_w = rgba.width();
                let orig_h = rgba.height();

                let scaled_w = orig_w * scale;
                let scaled_h = orig_h * scale;
                let mut scaled_buffer = vec![0u8; (scaled_w * scaled_h * 4) as usize];

                for y in 0..orig_h {
                    for x in 0..orig_w {
                        let pixel = rgba.get_pixel(x, y);
                        let r = pixel[0];
                        let g = pixel[1];
                        let b = pixel[2];
                        let a = pixel[3];

                        for sy in 0..scale {
                            for sx in 0..scale {
                                let out_x = x * scale + sx;
                                let out_y = y * scale + sy;
                                let idx = ((out_y * scaled_w + out_x) * 4) as usize;

                                scaled_buffer[idx] = b;
                                scaled_buffer[idx + 1] = g;
                                scaled_buffer[idx + 2] = r;
                                scaled_buffer[idx + 3] = a;
                            }
                        }
                    }
                }

                if let Some(img_buffer) =
                    image::RgbaImage::from_raw(scaled_w, scaled_h, scaled_buffer)
                {
                    let frame = image::Frame::new(img_buffer);
                    map.insert(name, Arc::new(RenderImage::new(vec![frame])));
                }
            }
        }
        map
    });

    cache.get(icon_name).cloned().expect("Icon missing from cache")
}
