use super::gds_record::Date;

#[derive(Default, Debug)]
pub struct Lib {
    pub name: String,
    pub units: f64, //in meter
    pub precision: f64,
    pub cells: Vec<Box<Cell>>,
    pub date: Date,
}

impl Lib {}

#[derive(Default, Debug)]
pub struct Cell {
    pub name: String,
    pub polygons: Vec<Box<Polygon>>,
    pub paths: Vec<Box<Path>>,
    pub refs: Vec<Box<Ref>>,
    pub label: Vec<Box<Text>>,
    pub date: Date,
}

#[derive(Default, Debug)]
pub struct Polygon {
    pub layer: i16,
    pub datatype: i16,
    pub points: Vec<(f64, f64)>,
}

#[derive(Default, Debug)]
pub struct Path {
    pub layer: i16,
    pub datatype: i16,
    pub width: i32,
    pub end_type: i16,
    pub points: Vec<(i32, i32)>,
}

#[derive(Default, Debug)]
pub struct Ref {
    pub ref_cell_name: String,
    pub reflection_x: bool,
    pub abs_magnific: bool,
    pub abs_angel: bool,
    pub angle: f64, //measured in degrees and in the counterclockwise direction
    pub origin: (i32, i32),
}

#[derive(Debug)]
enum TextAnchor {
    NW, // NorthWest
    N,
    NE, // NorthEast
    W,
    O, // Origin
    E,
    SW, // SouthWest
    S,
    SE,
}

impl Default for TextAnchor {
    fn default() -> TextAnchor {
        TextAnchor::O
    }
}
#[derive(Default, Debug)]
pub struct Text {
    pub layer: i16,
    pub datatype: i16,
    pub text: String,
    pub position: (f64, f64),
    pub anchor: TextAnchor,
    pub rotation: f64,      // in radians
    pub magnification: f64, // (not supported by OASIS)
    pub x_reflection: bool, // (not supported by OASIS)
    pub repetition: Repetition,
}

#[derive(Default, Debug)]
pub struct Repetition {
    pub count_1: u64,
    pub count_2: u64,
    pub vec_1: (f64, f64),
    pub vec_2: (f64, f64),
}
