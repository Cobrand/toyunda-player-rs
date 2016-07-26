#[allow(dead_code)]
#[derive(Copy,Debug,Clone)]
pub enum PosX {
    Centered,
    FromLeft(u32),
    FromRight(u32),
    FromLeftPercent(f32),
    FromRightPercent(f32),
}

#[allow(dead_code)]
#[derive(Copy,Debug,Clone)]
pub enum PosY {
    Centered,
    FromTop(u32),
    FromBottom(u32),
    FromTopPercent(f32),
    FromBottomPercent(f32),
}

#[allow(dead_code)]
#[derive(Copy,Debug,Clone)]
pub enum Size {
    // Percentage(f64),
    FitPercent(Option<f32>, Option<f32>),
    Fit(Option<u32>, Option<u32>),
}

pub fn real_position((displayer_width,displayer_height):(u32,u32),
                     (pos_x,pos_y):(PosX,PosY),
                     (anchor_x,anchor_y): (f32,f32),
                     (object_width,object_height):(u32,u32)) -> (i32,i32) {
    let mut object_pos :(i32,i32) = (0,0);
    let delta_anchor_x = (anchor_x * object_width as f32) as i32;
    let delta_anchor_y = (anchor_y * object_height as f32) as i32;
    match pos_x {
        PosX::Centered => {
            object_pos.0 = (displayer_width / 2) as i32 - delta_anchor_x
        },
        PosX::FromLeft(value) => {
            object_pos.0 = value as i32 - delta_anchor_x
        },
        PosX::FromLeftPercent(percent) => {
            object_pos.0 = (percent * (displayer_width as f32)) as i32 - delta_anchor_x
        }
        PosX::FromRight(value) => {
            object_pos.0 = displayer_width as i32 - value as i32 - delta_anchor_x
        }
        PosX::FromRightPercent(percent) => {
            object_pos.0 = displayer_width as i32 - (percent * (displayer_width as f32)) as i32 -
                           delta_anchor_x
        }
    };
    match pos_y {
        PosY::Centered => {
            object_pos.1 = (displayer_height / 2) as i32 - delta_anchor_y
        },
        PosY::FromTop(value) => {
            object_pos.1 = value as i32 - delta_anchor_y
        },
        PosY::FromTopPercent(percent) => {
            object_pos.1 = (percent * (displayer_height as f32)) as i32 - delta_anchor_y
        }
        PosY::FromBottom(value) => {
            object_pos.1 = displayer_height as i32 - value as i32 - delta_anchor_y
        }
        PosY::FromBottomPercent(percent) => {
            object_pos.1 = displayer_height as i32 - (percent * (displayer_height as f32)) as i32 -
                           delta_anchor_y
        }
    };
    object_pos
}
