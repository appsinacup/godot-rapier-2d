use rapier2d::prelude::*;

#[derive(Clone)]
pub struct Vector {
    pub x: Real,
    pub y: Real,
}

impl Vector {
    pub fn new(x: Real, y: Real) -> Vector {
        Vector { x, y }
    }
}
