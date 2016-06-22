extern crate sdl2;
use sdl2::pixels::Color;
use std::cmp::{max, min};

/// will always return a flat color regardless or alpha
pub fn mix_colors(color1: Color, color2: Color, mix_ratio: f32) -> Color {
    let (r1, g1, b1) = color1.rgb();
    let (r2, g2, b2) = color2.rgb();
    let (r1, g1, b1) = (r1 as i16, g1 as i16, b1 as i16);
    let (r2, g2, b2) = (r2 as i16, g2 as i16, b2 as i16);
    let (r, g, b): (i16, i16, i16) = (((r2 - r1) as f32 * mix_ratio) as i16 + r1,
                                      ((g2 - g1) as f32 * mix_ratio) as i16 + g1,
                                      ((b2 - b1) as f32 * mix_ratio) as i16 + b1);

    let (r, g, b) = (max(min(r, 0xFF), 0) as u8,
                     max(min(g, 0xFF), 0) as u8,
                     max(min(b, 0xFF), 0) as u8);
    Color::RGB(r, g, b)
}

pub fn fade_color(color1: Color, fade_ratio: f32) -> Color {
    let (r, g, b, a): (u8, u8, u8, u8) = match color1 {
        Color::RGB(r, g, b) => (r, g, b, 255),
        Color::RGBA(r, g, b, a) => (r, g, b, a),
    };
    Color::RGBA(r, g, b, (a as f32 * fade_ratio) as u8)
}
