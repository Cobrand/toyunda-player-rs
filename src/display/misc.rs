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
