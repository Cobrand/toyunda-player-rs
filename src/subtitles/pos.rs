extern crate serde;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::fmt::Debug;

#[derive(Debug,Copy,Clone,Serialize,Deserialize,PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy,Clone,Debug,PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RowPosition {
    Row(u8),
    ForcePos(Point),
}

#[derive(Copy,Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct Size {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Default for RowPosition {
    fn default() -> RowPosition {
        RowPosition::Row(0)
    }
}
