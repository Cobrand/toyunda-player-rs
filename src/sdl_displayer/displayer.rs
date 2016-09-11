use sdl2::render::{Renderer, BlendMode, Texture};
use sdl2_image::{LoadTexture, INIT_PNG, INIT_JPG, init as image_init};
use sdl2_ttf::{Sdl2TtfContext, init as ttf_init};
use ::overlay::{Display,OverlayFrame,Rect} ;
use super::font::*;

pub struct SDLDisplayer<'a> {
    pub fonts: FontList,
    pub renderer: Renderer<'a>,
    #[allow(dead_code)]
    ttf_context: Sdl2TtfContext,
    pub lyrics_logo: Option<Texture>,
}

impl<'a> SDLDisplayer<'a> {
    pub fn new(mut renderer: Renderer<'a>) -> Result<SDLDisplayer<'a>, ()> {
        renderer.set_blend_mode(BlendMode::Blend);
        let ttf_context = ttf_init().unwrap();
        let font_list = FontList::new(&ttf_context).unwrap();
        let _image_context = image_init(INIT_PNG | INIT_JPG).unwrap();
        // we dont care if imag econtext dies, we only load images once (for now)
        let lyrics_logo: Option<Texture> = match ::std::env::current_exe() {
            Ok(current_exe_path) => {
                match renderer.load_texture(&*current_exe_path.with_file_name("logo_toyunda.png")) {
                    Ok(texture) => Some(texture),
                    Err(e) => {
                        error!("Failed to load logo_toyunda.png : error '{}' ({:?})", e, e);
                        None
                    }
                }
            }
            _ => {
                error!("Failed to open logo_toyunda.png : failed to find current executable");
                None
            }
        };
        let displayer = SDLDisplayer {
            fonts: font_list,
            ttf_context: ttf_context,
            renderer: renderer,
            lyrics_logo: lyrics_logo,
        };
        Ok(displayer)
    }

    pub fn copy_lyrics_logo(&mut self, rect: ::sdl2::rect::Rect) {
        match self.lyrics_logo {
            Some(ref texture) => {
                self.renderer.copy(texture, None, Some(rect));
            }
            None => {}
        };
    }

    // TODO use this somewhere ?
    #[allow(dead_code)]
    pub fn fatal_error_message(&self, title: &str, info: &str) {
        use ::sdl2::messagebox::ShowMessageError;
        let res =
            ::sdl2::messagebox::show_simple_message_box(::sdl2::messagebox::MESSAGEBOX_ERROR,
                                                        title,
                                                        info,
                                                        self.sdl_renderer().window());
        match res {
            Ok(_) => {}
            Err(ShowMessageError::SdlError(string)) => {
                error!("Unexpected SDL_ERROR {} when creating MessageBox", string);
            }
            Err(_) => error!("Unexpected Error when creating MessageBox"),
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
        self.renderer.window().unwrap().gl_swap_window();
    }

    #[inline]
    pub fn sdl_renderer_mut(&mut self) -> &mut Renderer<'a> {
        &mut self.renderer
    }

    #[inline]
    pub fn sdl_renderer(&self) -> &Renderer<'a> {
        &self.renderer
    }
}

impl<'a> Display for SDLDisplayer<'a> {
    fn display(&self,overlay_frame:&OverlayFrame) -> Vec<Rect> {
        vec!()
    }
}
