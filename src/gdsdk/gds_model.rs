use super::gds_record::{self, Date};
use super::gds_writer::*;
use std::{collections::HashMap, rc::Rc};

#[derive(Default, Debug)]
pub struct Lib {
    pub name: String,
    pub units: f64, //in meter
    pub precision: f64,
    pub cells: Vec<Rc<std::cell::RefCell<Cell>>>,
    pub date: Date,
}

fn get_cell_from_ref(refer: &Ref, uniqcells: &mut HashMap<String, Rc<std::cell::RefCell<Cell>>>) {
    if let RefCell::Cell(c) = &refer.refed_cell {
        let cell = (*(*c)).borrow();
        if !uniqcells.contains_key(&cell.name) {
            uniqcells.insert(cell.name.clone(), c.clone());
        }
        for r in &cell.refs {
            get_cell_from_ref(r, uniqcells);
        }
    } else {
        panic!("Reference should not refer cell by name");
    }
}

impl Lib {
    fn all_cells(&self) -> Vec<Rc<std::cell::RefCell<Cell>>> {
        let mut uniqcells = HashMap::<String, Rc<std::cell::RefCell<Cell>>>::new();
        let mut cells_vec = Vec::<Rc<std::cell::RefCell<Cell>>>::new();
        for c in &self.cells {
            let cell = (*(*c)).borrow();
            if !uniqcells.contains_key(&cell.name) {
                uniqcells.insert(cell.name.clone(), c.clone());
            }
            for r in &cell.refs {
                get_cell_from_ref(r, &mut uniqcells);
            }
        }

        for c in uniqcells {
            cells_vec.push(c.1.clone());
        }
        cells_vec
    }

    pub fn gds_bytes(&self) -> Vec<u8> {
        self.to_gds(0.0)
    }
}

const GDS_VERSIOIN: i16 = 600;

impl GdsObject for Lib {
    fn to_gds(&self, _: f64) -> Vec<u8> {
        let mut data = Vec::<u8>::new();

        // gds data binary format is in big endian
        // header
        let mut header_data = Vec::<u8>::new();

        header_data.extend(gds_record::HEADER);
        header_data.extend(GDS_VERSIOIN.to_be_bytes());

        data.extend((header_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(header_data);

        // date
        let mut date_data = Vec::<u8>::new();
        date_data.extend(gds_record::BGNLIB);
        date_data.extend(self.date.mod_year.to_be_bytes());
        date_data.extend(self.date.mod_month.to_be_bytes());
        date_data.extend(self.date.mod_day.to_be_bytes());
        date_data.extend(self.date.mod_hour.to_be_bytes());
        date_data.extend(self.date.mod_minute.to_be_bytes());
        date_data.extend(self.date.mod_second.to_be_bytes());
        date_data.extend(self.date.acc_year.to_be_bytes());
        date_data.extend(self.date.acc_month.to_be_bytes());
        date_data.extend(self.date.acc_day.to_be_bytes());
        date_data.extend(self.date.acc_hour.to_be_bytes());
        date_data.extend(self.date.acc_minute.to_be_bytes());
        date_data.extend(self.date.acc_second.to_be_bytes());

        data.extend((date_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(date_data);

        // lib name
        let mut lib_name = Vec::<u8>::new();
        lib_name.extend(gds_record::LIBNAME);
        let mut name = ascii_string_to_be_bytes(&self.name);
        if !name.len().is_power_of_two() {
            name.push(0);
        }
        lib_name.extend(name);

        data.extend((lib_name.len() as i16 + 2_i16).to_be_bytes());
        data.extend(lib_name);

        // unit
        let mut unit_data = Vec::<u8>::new();
        unit_data.extend(gds_record::UNITS);
        unit_data.extend(f64_to_gds_bytes(self.precision / self.units));
        unit_data.extend(f64_to_gds_bytes(self.precision));

        data.extend((unit_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(unit_data);

        let scaling = self.units / self.precision;

        let all_cells = self.all_cells();
        all_cells.iter().for_each(|c| {
            let cell = (*c).borrow();
            data.extend(cell.to_gds(scaling));
        });

        // endlib
        let mut endlib_data = Vec::<u8>::new();
        endlib_data.extend(gds_record::ENDLIB);

        data.extend((endlib_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(endlib_data);
        data
    }
}

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

impl GdsObject for Cell {
    fn to_gds(&self, scaling: f64) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data
    }
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
    Cell(Rc<std::cell::RefCell<Cell>>),
    CellName(String),
}

impl Default for RefCell {
    fn default() -> Self {
        RefCell::CellName("".to_string())
    }
}

#[derive(Default, Debug)]
pub struct Ref {
    pub(crate) refed_cell: RefCell,
    pub reflection_x: bool,
    // pub abs_magnific: bool,
    pub magnific: f64,
    // pub abs_angel: bool,
    pub angle: f64, //measured in degrees and in the counterclockwise direction
    pub origin: Points,
    pub row: i16,
    pub column: i16,
    pub spaceing_row: Points,
    pub spaceing_col: Points,
}

impl Ref {
    pub fn new() -> Self {
        Ref {
            reflection_x: false,
            magnific: 1.0,
            ..Default::default()
        }
    }
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

trait GdsObject {
    fn to_gds(&self, scaling: f64) -> Vec<u8>;
}
