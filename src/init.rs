extern crate sdl2;
extern crate mpv;
extern crate sdl2_sys;
use mpv::MpvHandler;
use sdl2::render::Renderer;
use sdl2::video::Window;
use sdl2::Sdl;
use std::os::raw::{c_void,c_char};
use std::ffi::CStr;

pub unsafe extern "C" fn get_proc_address(arg: *mut c_void,
                                      name: *const c_char) -> *mut c_void {
    let arg: &sdl2::VideoSubsystem = &*(arg as *mut sdl2::VideoSubsystem);
    let name = CStr::from_ptr(name).to_str().unwrap();
    arg.gl_get_proc_address(name) as *mut c_void
}

pub fn find_sdl_gl_driver() -> Option<u32>{
    let mut opengl_driver : Option<u32> = None ;
    info!("Detecting drivers ...");
    // SDL drivers are counted from 0
    // Typically here if we want to draw with SDL on mpv we must use the "opengl" driver,
    // and for instance not the direct3d driver (on windows), nor the opengles driver, ...
    let mut driver_index : i32 = -1 ;
    for item in sdl2::render::drivers() {
        driver_index = driver_index + 1 ;
        info!("* Found driver '{}'",item.name);
        if item.name == "opengl"{
            opengl_driver = Some(driver_index as u32);
        }
    }
    opengl_driver
}

pub fn init_mpv()  {
}

pub fn init_sdl<'a>(video_subsystem:&mut sdl2::VideoSubsystem,opengl_driver:u32) -> Renderer<'a> {
    let window = video_subsystem.window("Toyunda Player", 960, 540)
        .resizable()
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut renderer = window.renderer()
        .present_vsync()
        .index(opengl_driver)
        .build()
        .expect("Failed to create renderer with given parameters");
    renderer.window()
            .expect("Failed to extract window from displayer")
            .gl_set_context_to_current()
            .unwrap();
    renderer
}
