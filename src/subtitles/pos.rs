use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::fmt::Debug;

#[derive(Debug,Copy,Clone,Serialize,Deserialize,PartialEq)]
pub struct Point<T: Debug + Copy + Clone + Serialize + Deserialize + PartialEq> {
    pub x: T,
    pub y: T,
}

impl Deserialize for RowPosition {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        struct Visitor;
        impl de::Visitor for Visitor {
            type Value = RowPosition;
            fn visit_u8<E>(&mut self, value: u8) -> Result<RowPosition, E>
                where E: de::Error
            {
                Ok(RowPosition::Row(value))
            }

            fn visit_u64<E>(&mut self, value: u64) -> Result<RowPosition, E>
                where E: de::Error
            {
                Ok(RowPosition::Row(value as u8))
            }

            fn visit_map<M>(&mut self, visitor: M) -> Result<RowPosition, M::Error>
                where M: de::MapVisitor
            {
                let mut mvd = de::value::MapVisitorDeserializer::new(visitor);
                let res: Result<Point<f32>, _> = Deserialize::deserialize(&mut mvd);
                res.map(|v| RowPosition::ForcePos(v))
            }
        }
        deserializer.deserialize(Visitor)
    }
}

impl Serialize for RowPosition {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match *self {
            RowPosition::Row(r) => serializer.serialize_u64(r as u64),
            RowPosition::ForcePos(p) => p.serialize(serializer),
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum RowPosition {
    Row(u8),
    ForcePos(Point<f32>),
}

#[derive(Copy,Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct Size<T: Copy + Clone + Debug + PartialEq + Serialize + Deserialize> {
    pub width: T,
    pub height: T,
}

impl Default for RowPosition {
    fn default() -> RowPosition {
        RowPosition::Row(0)
    }
}
