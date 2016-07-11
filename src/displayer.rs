extern crate sdl2;
extern crate sdl2_ttf;
use sdl2::render::{Renderer, BlendMode};
use sdl2::pixels::Color;
use std::env;
use font::*;
use display::{self, Display};
use subtitles::Subtitles;
use utils::*;

pub struct Displayer<'a> {
    fonts: FontList,
    renderer: Renderer<'a>,
    #[allow(dead_code)]
    ttf_context: sdl2_ttf::Sdl2TtfContext,
    subtitles: Option<Subtitles>,
}

impl<'a> Displayer<'a> {
    pub fn new(mut renderer: Renderer<'a>,
               subtitles: Option<Subtitles>)
               -> Result<Displayer<'a>, ()> {
        renderer.set_blend_mode(BlendMode::Blend);
        let ttf_context = sdl2_ttf::init().unwrap();
        let mut font_path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
        font_path.push("res/DejaVuSansMono-Bold.ttf");
        let font_path = font_path.as_path();
        let font_list = FontList::new(font_path, &ttf_context).unwrap();
        let displayer = Displayer {
            fonts: font_list,
            ttf_context: ttf_context,
            renderer: renderer,
            subtitles: subtitles,
        };
        Ok(displayer)
    }

    pub fn render_subtitles(&mut self, frame_number: u32) {
        const FADE_INTERVAL: u32 = 10;
        const TRANSITION_INTERVAL: u32 = FADE_INTERVAL + 5;
        let sub_colors = (Color::RGB(0, 0xFF, 0xFF),
                          Color::RGB(0xFF, 0, 0),
                          Color::RGB(0xFF, 0xFF, 0));
        let mut text2d_vec = vec![];
        match self.subtitles {
            Some(ref subtitles) => {
                let subtitles = subtitles.get_subtitles();
                let iter = subtitles.iter().enumerate().filter(|&(_, line)| {
                    match (line.first(), line.last()) {
                        (Some(&(_, (frame_begin, _))), Some(&(_, (_, frame_end)))) => {
                            frame_begin.saturating_sub(TRANSITION_INTERVAL) < frame_number &&
                            frame_end.saturating_add(TRANSITION_INTERVAL) > frame_number
                        }
                        _ => false,
                    }
                });
                for (i, line) in iter {
                    let text_pos_y = match i % 2 {
                        0 => display::PosY::FromTopPercent(0.02),
                        1 => display::PosY::FromTopPercent(0.12),
                        _ => unreachable!(),
                    };
                    let alpha: f32 = match (line.first(), line.last()) {
                        (Some(&(_, (frame_begin, _))), Some(&(_, (_, frame_end)))) => {
                            let finished_fade_interval = TRANSITION_INTERVAL - FADE_INTERVAL;
                            let begin_first_fade_frame =
                                frame_begin.saturating_sub(TRANSITION_INTERVAL);
                            let end_first_fade_frame =
                                frame_begin.saturating_sub(finished_fade_interval);
                            let begin_second_fade_frame =
                                frame_end.saturating_add(finished_fade_interval);
                            let end_second_fade_frame =
                                frame_end.saturating_add(TRANSITION_INTERVAL);
                            debug_assert_eq!(end_second_fade_frame - begin_second_fade_frame,
                                             FADE_INTERVAL);
                            if (end_first_fade_frame < frame_number &&
                                begin_second_fade_frame > frame_number) {
                                1.0
                            } else if begin_first_fade_frame <= frame_number &&
                               end_first_fade_frame >= frame_number {
                                (frame_number - begin_first_fade_frame) as f32 /
                                (end_first_fade_frame - begin_first_fade_frame) as f32
                            } else if begin_second_fade_frame <= frame_number &&
                               end_second_fade_frame >= frame_number {
                                1.0 -
                                ((frame_number - begin_second_fade_frame) as f32 /
                                 (end_second_fade_frame - begin_second_fade_frame) as f32)
                            } else {
                                0.0
                            }
                        }
                        _ => 0.0,
                    };
                    let text_elts = line.iter().fold(vec![], |mut accu_vec,
                                                      &(ref syllabus, (frame_begin, frame_end))| {
                        if (frame_number < frame_begin) {
                            // not yet

                            let text_2d = display::TextElement {
                                text: syllabus.clone(),
                                color: fade_color(sub_colors.2, alpha),
                                outline: Some(Color::RGB(0, 0, 0)),
                                shadow: None,
                            };
                            accu_vec.push(text_2d);

                        } else if (frame_begin <= frame_number) && (frame_number <= frame_end) {
                            let percent = (frame_number - frame_begin) as f32 /
                                          (frame_end - frame_begin) as f32;
                            // lets ease the percent a lil bits
                            let percent = 1.0 - (1.0 - percent).sqrt();
                            // let percent = (percent * consts::PI / 2.0).sin();
                            let transition_color = mix_colors(sub_colors.1, sub_colors.0, percent);
                            let text_2d = display::TextElement {
                                text: syllabus.clone(),
                                color: transition_color,
                                outline: Some(Color::RGB(0, 0, 0)),
                                shadow: None,
                            };
                            accu_vec.push(text_2d);
                        } else {
                            if (accu_vec.is_empty()) {
                                let text_2d = display::TextElement {
                                    text: syllabus.clone(),
                                    color: fade_color(sub_colors.0, alpha),
                                    outline: Some(Color::RGB(0, 0, 0)),
                                    shadow: None,
                                };
                                accu_vec.push(text_2d);
                            } else {
                                let mut text_2d = accu_vec.last_mut().unwrap();
                                text_2d.text.push_str(&syllabus);
                            }
                        }
                        accu_vec
                    });
                    let text_2d = display::Text2D {
                        text: text_elts,
                        size: display::Size::FitPercent(Some(0.95), Some(0.1)),
                        pos: (display::PosX::Centered, text_pos_y),
                        anchor: (0.5, 0.0),
                    };
                    text2d_vec.push(text_2d);
                }
            }
            None => {}
        };
        for text_2d in text2d_vec.into_iter() {
            text_2d.draw(self);
        }
    }

    // width and height must be between 0 and 1
    pub fn sub_screen_dims(&self,
                           width: Option<f32>,
                           height: Option<f32>)
                           -> (Option<u32>, Option<u32>) {
        let dims: (u32, u32) = self.renderer.window().unwrap().size();
        (width.and_then(|width| Some((width * (dims.0 as f32)) as u32)),
         height.and_then(|height| Some((height * (dims.1 as f32)) as u32)))
    }

    pub fn render(&mut self) {
        self.sdl_renderer_mut().window().unwrap().gl_swap_window();
    }

    pub fn sdl_renderer_mut(&mut self) -> &mut Renderer<'a> {
        &mut self.renderer
    }

    pub fn sdl_renderer(&self) -> &Renderer<'a> {
        &self.renderer
    }

    pub fn fonts(&self) -> &FontList {
        &self.fonts
    }
}
