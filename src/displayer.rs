extern crate sdl2;
extern crate sdl2_ttf;
use sdl2::render::{Renderer, TextureQuery, BlendMode};
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use std::vec::Vec;
use std::path::Path;
use font::*;
use std::ops::DerefMut;
use display::{self,Display};
use subtitles::{self,Subtitles};

pub struct Displayer<'a> {
    fonts: FontList,
    renderer: Renderer<'a>,
    #[allow(dead_code)]
    ttf_context: sdl2_ttf::Sdl2TtfContext,
    subtitles:Option<Subtitles>
}

impl<'a> Displayer<'a> {
    pub fn new(mut renderer: Renderer<'a>,subtitles:Option<Subtitles>) -> Result<Displayer<'a>, ()> {
        renderer.set_blend_mode(BlendMode::Blend);
        let ttf_context = sdl2_ttf::init().unwrap();
        let font_list = FontList::new(Path::new("./res/DejaVuSansMono-Bold.ttf"),
                                      &ttf_context)
                            .unwrap();
        let displayer = Displayer {
            fonts: font_list,
            ttf_context: ttf_context,
            renderer: renderer,
            subtitles: subtitles
        };
        Ok(displayer)
    }

    pub fn render_subtitles(&mut self,frame_number:u32){
        let mut text2d_vec = vec![];
        match self.subtitles {
            Some(ref subtitles) => {
                let subtitles = subtitles.get_subtitles();
                let iter = subtitles.iter().enumerate().filter(|&(i,line)|{
                    line.iter().any(|syllabus|{
                        let &(ref syllabus,(frame_begin,frame_end)) = syllabus;
                        (frame_begin <= frame_number) && (frame_number <= frame_end)
                    })
                });
                for (i,line) in iter {
                    let text_pos_y = match i%2 {
                        0 => {display::PosY::FromTopPercent(0.02)},
                        1 => {display::PosY::FromTopPercent(0.12)},
                        _ => unreachable!()
                    };
                    let text_elts = line.iter().fold(vec![],|mut accu_vec,&(ref syllabus,(frame_begin,frame_end))|{
                        if (frame_number < frame_begin){ // not yet
                            if (accu_vec.is_empty()){
                                let text_2d = display::TextElement {
                                    text:syllabus.clone(),
                                    color:Color::RGBA(128,255,0,255),
                                    outline:Some(Color::RGB(0,0,0)),
                                    shadow:None
                                };
                                accu_vec.push(text_2d);
                            } else {
                                let mut text_2d = accu_vec.last_mut().unwrap();
                                text_2d.text.push_str(&syllabus) ;
                            }
                        } else if (frame_begin <= frame_number) && (frame_number <= frame_end) {
                            let text_2d = display::TextElement {
                                text:syllabus.clone(),
                                color:Color::RGBA(255,0,0,255),
                                outline:Some(Color::RGB(0,0,0)),
                                shadow:None
                            };
                            accu_vec.push(text_2d);
                        } else {
                            let text_2d = display::TextElement {
                                text:syllabus.clone(),
                                color:Color::RGBA(128,128,255,255),
                                outline:Some(Color::RGB(0,0,0)),
                                shadow:None
                            };
                            accu_vec.push(text_2d);
                        }
                        accu_vec
                    });
                    let text_2d = display::Text2D {
                        text:text_elts,
                        size:display::Size::FitPercent(Some(0.95),Some(0.1)),
                        pos:(display::PosX::Centered,text_pos_y),
                        anchor:(0.5,0.0)
                    };
                    text2d_vec.push(text_2d);
                }
            },
            None => {}
        };
        for text2D in text2d_vec.into_iter() {
            text2D.draw(self);
        }
    }

    pub fn display(&mut self, text: &str) {
        let size: f32 = 0.039;
        let window_width = self.renderer.window().unwrap().size().0 ;
        let font_set = self.fonts.get_fittest_font_set(text, (Some(window_width),None),true).unwrap();
        let font = font_set.get_regular_font();
        let font_outline = font_set.get_outline_font();
        let surface = font.render(text)
                          .blended(Color::RGB(180, 180, 180))
                          .unwrap();
        let mut surface_outline = font_outline.render(text)
                                              .blended(Color::RGB(0, 0, 0))
                                              .unwrap();
        let outline_width: u32 = 2;
        let (width, height) = surface_outline.size();
        surface.blit(None,
                     surface_outline.deref_mut(),
                     Some(Rect::new(outline_width as i32,
                                    outline_width as i32,
                                    (width - outline_width),
                                    (height - outline_width)))).expect("Failed to blit texture");
        let mut texture = self.renderer.create_texture_from_surface(&surface_outline).unwrap();
        texture.set_blend_mode(BlendMode::Blend);
        texture.set_alpha_mod(128);
        let TextureQuery { width:texture_width, height:texture_height, format:texture_format,.. } = texture.query();
        self.renderer.copy(&mut texture,
                           None,
                           Some(Rect::new(3, 3, texture_width, texture_height)));
    }

    pub fn example(&mut self) {
        let text_element_1 = display::TextElement {
            text:"S".to_string(),
            color:Color::RGBA(0,0,0,255),
            outline:Some(Color::RGB(255,255,255)),
            shadow:None
        };
        let text_element_2 = display::TextElement {
            text:"L".to_string(),
            color:Color::RGBA(255,255,255,255),
            outline:Some(Color::RGB(0,0,0)),
            shadow:None
        };
        let text_element_3 = display::TextElement {
            text:"T".to_string(),
            color:Color::RGBA(255,0,0,200),
            outline:None,
            shadow:None
        };
        let text_2d : display::Text2D = display::Text2D {
            text:vec![text_element_1,text_element_2,text_element_3],
            size:display::Size::FitPercent(Some(0.9),Some(0.1)),
            pos:(display::PosX::Centered,display::PosY::FromTopPercent(0.50)),
            anchor:(0.5,0.5)
        };
        text_2d.draw(self);
    }

    // width and height must be between 0 and 1
    pub fn sub_screen_dims(&self,width:Option<f32>,height:Option<f32>) -> (Option<u32>,Option<u32>){
        let dims : (u32,u32)= self.renderer.window().unwrap().size();
        (
            width.and_then(|width|{
                Some((width *  (dims.0 as f32)) as u32)
            }) ,
            height.and_then(|height|{
                Some((height *  (dims.1 as f32)) as u32)
            })
        )
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
