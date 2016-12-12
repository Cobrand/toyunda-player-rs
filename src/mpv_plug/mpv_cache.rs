use mpv::MpvHandler;
use std::cmp::max;

#[derive(Debug)]
pub struct MpvCache {
    time_pos: Option<f64>,
    width: Option<u32>,
    height: Option<u32>,
    percent_pos: Option<f64>,
}

impl MpvCache {
    pub fn new() -> MpvCache {
        MpvCache {
            time_pos: None,
            width: None,
            height: None,
            percent_pos: None,
        }
    }

    pub fn update(&mut self, mpv: &MpvHandler) {
        if let Ok(t) = mpv.get_property::<f64>("time-pos") {
            self.time_pos = Some(t.max(0.0));
        } else {
            self.time_pos = None;
        };
        if let Ok(w) = mpv.get_property::<i64>("width") {
            self.width = Some(max(w, 0) as u32);
        } else {
            self.width = None;
        };
        if let Ok(h) = mpv.get_property::<i64>("height") {
            self.height = Some(max(h, 0) as u32);
        } else {
            self.height = None;
        };
        if let Ok(t) = mpv.get_property::<f64>("percent-pos") {
            self.percent_pos = Some(t.max(0.0));
        } else {
            self.percent_pos = None;
        };
    }

    pub fn cached_width(&self) -> Option<u32> {
        self.width.clone()
    }

    pub fn cached_height(&self) -> Option<u32> {
        self.height.clone()
    }

    pub fn cached_time_pos(&self) -> Option<f64> {
        self.time_pos.clone()
    }

    pub fn cached_percent_pos(&self) -> Option<f64> {
        self.percent_pos.clone()
    }
}
