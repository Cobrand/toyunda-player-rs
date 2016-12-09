extern crate sdl2;
extern crate sdl2_sys;
extern crate mpv;
use sdl2::render::Renderer;
use std::os::raw::{c_void, c_char};
use std::ffi::CStr;
use ::toyunda_player::{StartupParameters,ToyundaPlayer,ToyundaMode};
use sdl_displayer::SDLDisplayer;

unsafe extern "C" fn get_proc_address(arg: *mut c_void, name: *const c_char) -> *mut c_void {
    let arg: &sdl2::VideoSubsystem = &*(arg as *mut sdl2::VideoSubsystem);
    let name = CStr::from_ptr(name).to_str().unwrap();
    arg.gl_get_proc_address(name) as *mut c_void
}

pub fn player_start(startup_parameters: StartupParameters) {
    // INIT SDL
    let sdl_context = sdl2::init().unwrap();
    let mut video_subsystem = sdl_context.video().unwrap();
    let renderer = init_sdl(&mut video_subsystem, &startup_parameters);
    let video_subsystem_ptr = &mut video_subsystem as *mut _ as *mut c_void;
    // INIT MPV
    let mut mpv_builder = mpv::MpvHandlerBuilder::new().expect("Error while creating MPV builder");
    mpv_builder.set_option("sid", "no").unwrap(); // disables subtitles if any
    mpv_builder.set_option("softvol", "yes").unwrap(); // enables softvol so it can go higher than 100%
    mpv_builder.set_option("softvol-max", 250.0).unwrap(); // makes the max volume at 250%
    mpv_builder.set_option("aid",2i64).unwrap(); // aid to 2; normalized audio if there is one
    mpv_builder.try_hardware_decoding().unwrap(); // try hardware decoding instead of software decoding
    let mpv = mpv_builder.build_with_gl(Some(get_proc_address), video_subsystem_ptr)
        .expect("Error while initializing MPV");
    // BIND MPV WITH SDL

    let displayer = SDLDisplayer::new(renderer).expect("Failed to init displayer");

    if startup_parameters.mode == ToyundaMode::KaraokeMode {
        let mouse_utils = sdl_context.mouse();
        mouse_utils.show_cursor(false);
        // dont show cursor on top of player in karaoke mode
    }
    // Create a new displayer for the toyunda_player

    let mut toyunda_player = ToyundaPlayer::new(mpv, displayer);
    match toyunda_player.start(startup_parameters) {
        Err(e) => {
            error!("Failed to start player with given arguments, expect default parameters !\n\
                    '{}' ({:?})",
                   e,
                   e);
        }
        Ok(_) => {
            debug!("Parsed arguments successfully");
        }
    };
    let res = toyunda_player.main_loop(&sdl_context);
    match res {
        Ok(_) => {
            info!("Toyunda Player finished gracefully");
        }
        Err(e) => {
            error!("FATAL : {}", e);
        }
    };
}

fn find_sdl_gl_driver() -> Option<u32> {
    let mut opengl_driver: Option<u32> = None;
    debug!("Detecting drivers ...");
    // SDL drivers are counted from 0
    // Typically here if we want to draw with SDL on mpv we must use the "opengl" driver,
    // and for instance not the direct3d driver (on windows), nor the opengles driver, ...
    let mut driver_index: i32 = -1;
    for item in sdl2::render::drivers() {
        driver_index = driver_index + 1;
        debug!("* Found driver '{}'", item.name);
        if item.name == "opengl" {
            opengl_driver = Some(driver_index as u32);
        }
    }
    opengl_driver
}

fn init_sdl<'a>(video_subsystem: &mut sdl2::VideoSubsystem,
                    params: &StartupParameters)
                    -> Renderer<'a> {
    let opengl_driver_id = find_sdl_gl_driver().expect("Unable to find OpenGL video driver");

    let mut window_builder = if params.fullscreen {
        let mut builder = video_subsystem.window("Toyunda Player", 960, 540);
        builder.fullscreen_desktop();
        builder
    } else {
        video_subsystem.window("Toyunda Player", 960, 540)
    };
    let window = window_builder.resizable()
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let renderer = window.renderer()
        .present_vsync()
        .index(opengl_driver_id)
        .build()
        .expect("Failed to create renderer with given parameters");
    renderer.window()
        .expect("Failed to extract window from displayer")
        .gl_set_context_to_current()
        .unwrap();
    renderer
}
