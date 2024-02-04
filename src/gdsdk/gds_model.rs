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
pub struct Points {
    pub x: f64,
    pub y: f64,
}

impl Points {
    pub fn new(x: f64, y: f64) -> Self {
        Points { x, y }
    }
}

#[derive(Default, Debug)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64) -> Self {
        Vector { x, y }
    }
}

#[derive(Default, Debug)]
pub struct Cell {
    pub name: String,
    pub polygons: Vec<Polygon>,
    pub paths: Vec<Path>,
    pub refs: Vec<Ref>,
    pub label: Vec<Text>,
    pub date: Date,
}

#[derive(Default, Debug)]
pub struct Polygon {
    pub layer: i16,
    pub datatype: i16,
    pub points: Vec<Points>,
}

#[derive(Default, Debug)]
pub struct Path {
    pub layer: i16,
    pub datatype: i16,
    pub width: f64,
    pub end_type: i16,
    pub points: Vec<Points>,
}

#[derive(Debug)]
pub enum RefCell {
    Cell(Cell),
    CellName(String),
}

impl Default for RefCell {
    fn default() -> Self {
        RefCell::CellName("".to_string())
    }
}

#[derive(Default, Debug)]
pub struct Ref {
    pub refed_cell: RefCell,
    pub reflection_x: bool,
    pub abs_magnific: bool,
    pub magnific: f64,
    pub abs_angel: bool,
    pub angle: f64, //measured in degrees and in the counterclockwise direction
    pub origin: Points,
    pub row: i16,
    pub column: i16,
    pub spaceing_row: Points,
    pub spaceing_col: Points,
}

#[derive(Debug)]
pub enum TextAnchor {
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

#[derive(Debug)]
pub enum TextFont {
    Fonts0,
    Fonts1,
    Fonts2,
    Fonts3,
}

impl Default for TextFont {
    fn default() -> Self {
        TextFont::Fonts0
    }
}

#[derive(Default, Debug)]
pub struct Text {
    pub layer: i16,
    pub datatype: i16,
    pub font: TextFont,
    pub text: String,
    pub position: Points,
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
    pub vec_1: Vector,
    pub vec_2: Vector,
}
