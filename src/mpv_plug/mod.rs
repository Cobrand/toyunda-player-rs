
extern crate mpv;
extern crate ffmpeg;

mod mpv_cache;
pub use self::mpv_cache::*;

pub fn video_duration(file:&str) -> Result<u32,String> {
    match ffmpeg::format::input(&file) {
        Ok(context) => {
            let dur_sec : f64 = context.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64;
            let dur_ms = (dur_sec * 1000.0).max(0.0) as u32;
            Ok(dur_ms)
        },
        Err(error) => {
            Err(format!("ffmpeg error :{}", error))
        }
    }
}
