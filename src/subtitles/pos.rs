#[derive(Copy,Clone,Debug,Serialize,Deserialize,PartialEq)]
pub enum RowPosition {
    #[serde(rename="row")]
    Row(u8),
    #[serde(rename="force_pos")]
    ForcePos { x: f32, y: f32 },
}

impl Default for RowPosition {
    fn default() -> RowPosition {
        RowPosition::Row(0)
    }
}
