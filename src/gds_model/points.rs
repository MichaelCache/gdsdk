
/// geometry coord, in Lib units
#[derive(Default, Debug)]
pub struct Points {
    pub x: f64,
    pub y: f64,
}

impl Points {
    pub fn new(x: f64, y: f64) -> Self {
        Points { x, y }
    }
}