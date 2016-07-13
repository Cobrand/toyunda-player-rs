extern crate sdl2;
extern crate sdl2_ttf;
use sdl2::render::{Renderer, BlendMode};
use sdl2::pixels::Color;
use std::env;
use font::*;
// use display::{self, Display};
// use utils::*;

pub struct Displayer<'a> {
    fonts: FontList,
    renderer: Renderer<'a>,
    #[allow(dead_code)]
    ttf_context: sdl2_ttf::Sdl2TtfContext,
}
/*
mod subtitles_display {
    use sdl2::pixels::Color;
    pub const DEFAULT_SUB_COLORS : (Color,Color,Color) = (Color::RGB(0, 0xFF, 0xFF),
                                              Color::RGB(0xFF, 0, 0),
                                              Color::RGB(0xFF, 0xFF, 0));
    pub const DEFAULT_FADE_INTERVAL: u32 = 10;
    pub const DEFAULT_TRANSITION_INTERVAL: u32 = DEFAULT_FADE_INTERVAL + 5;

    #[derive(Copy,Clone)]
    pub struct Options {
        pub fade_interval:u32,
        pub transition_interval:u32,
        pub sub_colors:(Color,Color,Color)
    }

    impl Options {
        pub fn new() -> Options {
            Options {
                fade_interval:DEFAULT_FADE_INTERVAL,
                transition_interval:DEFAULT_TRANSITION_INTERVAL,
                sub_colors:DEFAULT_SUB_COLORS
            }
        }

        pub fn with_transition_interval(mut self,value:u32) -> Options {
            self.transition_interval = value;
            self
        }

        pub fn with_fade_interval(mut self,value:u32) -> Options {
            self.fade_interval =value;
            self
        }

        /// alive : not yet displayed;
        /// active: being displayed;
        /// dead: finished being displayed
        pub fn with_sub_colors(mut self,alive:Color,active:Color,dead:Color) -> Options {
            self.sub_colors = (alive,active,dead);
            self
        }
    }
}
*/
impl<'a> Displayer<'a> {
    pub fn new(mut renderer: Renderer<'a>)
               -> Result<Displayer<'a>, ()> {
        renderer.set_blend_mode(BlendMode::Blend);
        let ttf_context = sdl2_ttf::init().unwrap();
        let font_list = FontList::new(&ttf_context).unwrap();
        let displayer = Displayer {
            fonts: font_list,
            ttf_context: ttf_context,
            renderer: renderer,
        };
        Ok(displayer)
    }
    /*
    fn add_syllabus(mut text_elts : &mut Vec<display::TextElement>,
                    syllabus:&str,
                    current_frame:u32,
                    frame_begin:u32,
                    frame_end:u32,
                    alpha:f32,
                    sub_options:subtitles_display::Options) {
        if (current_frame < frame_begin) {
            // not yet

            let text_2d = display::TextElement {
                text: String::from(syllabus),
                color: fade_color(sub_options.sub_colors.2, alpha),
                outline: Some(Color::RGB(0, 0, 0)),
                shadow: None,
            };
            text_elts.push(text_2d);

        } else if (frame_begin <= current_frame) && (current_frame <= frame_end) {
            let percent = (current_frame - frame_begin) as f32 /
                          (frame_end - frame_begin) as f32;
            // lets ease the percent a lil bits
            let percent = 1.0 - (1.0 - percent*percent).sqrt();
            // let percent = (percent * consts::PI / 2.0).sin();
            let transition_color = mix_colors(sub_options.sub_colors.1, sub_options.sub_colors.0, percent);
            let text_2d = display::TextElement {
                text: String::from(syllabus),
                color: transition_color,
                outline: Some(Color::RGB(0, 0, 0)),
                shadow: None,
            };
            text_elts.push(text_2d);
        } else {
            if (text_elts.is_empty()) {
                let text_2d = display::TextElement {
                    text: String::from(syllabus),
                    color: fade_color(sub_options.sub_colors.0, alpha),
                    outline: Some(Color::RGB(0, 0, 0)),
                    shadow: None,
                };
                text_elts.push(text_2d);
            } else {
                let mut text_2d = text_elts.last_mut().unwrap();
                text_2d.text.push_str(&syllabus);
            }
        }
    }
    */
    pub fn fatal_error_message(&self,title:&str,info:&str) {
        ::sdl2::messagebox::show_simple_message_box(::sdl2::messagebox::MESSAGEBOX_ERROR,
                                                    title,
                                                    info,
                                                    self.sdl_renderer().window());
    }
    /*
    pub fn render_subtitles(&mut self, frame_number: u32) {
        let (w,h) = self.renderer.output_size().expect("unable to get renderer size");
        let mut target_texture = self.renderer.create_texture_target(sdl2::pixels::PixelFormatEnum::ARGB8888,
                                                                 w,
                                                                 h).expect("Unable to create texture");
        target_texture.set_blend_mode(BlendMode::Blend);
        let original_texture : Option<_> = self.renderer.render_target()
                                                        .expect("Unsupported graphic card")
                                                        .set(target_texture)
                                                        .unwrap();
        let sub_colors = (Color::RGB(0, 0xFF, 0xFF),
                          Color::RGB(0xFF, 0, 0),
                          Color::RGB(0xFF, 0xFF, 0));
        let mut text2d_vec = vec![];
        let sub_options = subtitles_display::Options::new();
        let finished_fade_interval = sub_options.transition_interval - sub_options.fade_interval;
        match self.subtitles {
            Some(ref subtitles) => {
                let subtitles = subtitles.get_subtitles();
                let iter = subtitles.iter().enumerate().filter(|&(_, line)| {
                    match (line.first(), line.last()) {
                        (Some(&(_, (frame_begin, _))), Some(&(_, (_, frame_end)))) => {
                            frame_begin.saturating_sub(sub_options.transition_interval) < frame_number &&
                            frame_end.saturating_add(sub_options.transition_interval) > frame_number
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

                            let begin_first_fade_frame =
                                frame_begin.saturating_sub(sub_options.transition_interval);
                            let end_first_fade_frame =
                                frame_begin.saturating_sub(finished_fade_interval);
                            let begin_second_fade_frame =
                                frame_end.saturating_add(finished_fade_interval);
                            let end_second_fade_frame =
                                frame_end.saturating_add(sub_options.transition_interval);
                            debug_assert_eq!(end_second_fade_frame - begin_second_fade_frame,
                                             sub_options.fade_interval);
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
                    let mut text_elts = Vec::new();
                    for &(ref syllabus,(frame_begin,frame_end)) in line.iter() {
                        Self::add_syllabus(&mut text_elts,syllabus,frame_number,frame_begin,frame_end,alpha,sub_options);
                    }
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
        };
        // TODO : restore first texture instead of asssuming it's window every time
        let target_texture = self.renderer.render_target().unwrap().reset().unwrap().unwrap();
        self.renderer.copy(&target_texture,None,None);
    }
    */
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
