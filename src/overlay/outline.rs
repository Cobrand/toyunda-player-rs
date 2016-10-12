use super::Color;
use std::cmp::Ordering;

#[derive(Debug,Clone,Copy)]
pub enum Outline {
    None,
    Light(Color),
    Bold(Color)
}

impl Outline {
    pub fn to_size(&self) -> usize {
        match self {
            &Outline::None => 0,
            &Outline::Light(_) => 1,
            &Outline::Bold(_) => 2
        }
    }
}

impl PartialEq for Outline {
    fn eq(&self,other: &Outline) -> bool {
        use self::Outline::*;
        match (self,other) {
            (&None, &None) |
            (&Light(_), &Light(_)) |
            (&Bold(_), &Bold(_)) => true,
            _ => false
        }
    }
}

impl Eq for Outline {}

impl Ord for Outline {
    fn cmp(&self,other: &Outline) -> Ordering {
        use self::Outline::*;
        match (self,other) {
            (&None, &None) |
            (&Light(_), &Light(_)) |
            (&Bold(_), &Bold(_)) => Ordering::Equal,
            (&None,_) => Ordering::Less,
            (&Light(_),&Bold(_)) => Ordering::Less,
            _ => Ordering::Greater
        }
    }
}

impl PartialOrd for Outline {
    fn partial_cmp(&self,other: &Outline) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
