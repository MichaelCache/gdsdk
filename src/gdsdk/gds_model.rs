use super::gds_record::{self, Date};
use super::gds_writer::{self, *};
use std::cell::RefCell;
use std::{collections::HashMap, rc::Rc};

#[derive(Default, Debug)]
pub struct Lib {
    pub name: String,
    pub units: f64, //in meter
    pub precision: f64,
    pub cells: Vec<Rc<RefCell<Cell>>>,
    pub date: Date,
}

fn get_cell_from_ref(refer: &Ref, uniqcells: &mut HashMap<String, Rc<RefCell<Cell>>>, depth: i64) {
    let cell = refer.refed_cell.borrow();
    if !uniqcells.contains_key(&cell.name) {
        uniqcells.insert(cell.name.clone(), refer.refed_cell.clone());
    }
    for r in &cell.refs {
        get_cell_from_ref(r, uniqcells, if depth > 0 { depth - 1 } else { depth });
    }
}

impl Lib {
    pub fn gds_bytes(&self) -> Vec<u8> {
        self.to_gds(0.0)
    }

    pub fn top_cells(&self) -> Vec<Rc<RefCell<Cell>>> {
        let mut top_cell = Vec::<Rc<RefCell<Cell>>>::new(); // self.cells.clone();
        let mut refed_cells = HashMap::<String, Rc<RefCell<Cell>>>::new();
        for c in &self.cells[..] {
            for refer in &c.borrow().refs[..] {
                get_cell_from_ref(refer, &mut refed_cells, -1)
            }
        }
        for ref c in &self.cells[..] {
            if !refed_cells.contains_key(&c.borrow().name) {
                top_cell.push((*c).clone());
            }
        }

        top_cell
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

        // bgnlib and date
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

        // cells
        self.cells.iter().for_each(|c| {
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
        // bgnstr and date
        let mut structure_data = Vec::<u8>::new();
        structure_data.extend(gds_record::BGNSTR);
        structure_data.extend(self.date.mod_year.to_be_bytes());
        structure_data.extend(self.date.mod_month.to_be_bytes());
        structure_data.extend(self.date.mod_day.to_be_bytes());
        structure_data.extend(self.date.mod_hour.to_be_bytes());
        structure_data.extend(self.date.mod_minute.to_be_bytes());
        structure_data.extend(self.date.mod_second.to_be_bytes());
        structure_data.extend(self.date.acc_year.to_be_bytes());
        structure_data.extend(self.date.acc_month.to_be_bytes());
        structure_data.extend(self.date.acc_day.to_be_bytes());
        structure_data.extend(self.date.acc_hour.to_be_bytes());
        structure_data.extend(self.date.acc_minute.to_be_bytes());
        structure_data.extend(self.date.acc_second.to_be_bytes());

        data.extend((structure_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(structure_data);

        // cell name
        let mut cell_name = Vec::<u8>::new();
        cell_name.extend(gds_record::STRNAME);
        let mut name = ascii_string_to_be_bytes(&self.name);
        if !name.len().is_power_of_two() {
            name.push(0);
        }
        cell_name.extend(name);

        data.extend((cell_name.len() as i16 + 2_i16).to_be_bytes());
        data.extend(cell_name);

        self.polygons.iter().for_each(|p| {
            data.extend(p.to_gds(scaling));
        });

        self.paths.iter().for_each(|p| {
            data.extend(p.to_gds(scaling));
        });

        self.refs.iter().for_each(|r| {
            data.extend(r.to_gds(scaling));
        });

        self.label.iter().for_each(|l| {
            data.extend(l.to_gds(scaling));
        });

        // endstr
        let mut endstr_data = Vec::<u8>::new();
        endstr_data.extend(gds_record::ENDSTR);

        data.extend((endstr_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(endstr_data);

        data
    }
}

#[derive(Default, Debug)]
pub struct Polygon {
    pub layer: i16,
    pub datatype: i16,
    pub points: Vec<Points>,
}

impl GdsObject for Polygon {
    fn to_gds(&self, scaling: f64) -> Vec<u8> {
        let mut data = Vec::<u8>::new();

        // boundary
        data.extend(4_i16.to_be_bytes());
        data.extend(gds_record::BOUNDARY);

        // layer
        data.extend(6_i16.to_be_bytes());
        data.extend(gds_record::LAYER);
        data.extend((self.layer as i16).to_be_bytes());

        // datatype
        data.extend(6_i16.to_be_bytes());
        data.extend(gds_record::DATATYPE);
        data.extend((self.datatype as i16).to_be_bytes());

        // points
        if self.points.len() > 8190 {
            panic!("Gds polygons can not have points more than 8190");
        }
        // gds polygon points front is same as end
        data.extend((4_i16 + 8 * (self.points.len() + 1) as i16).to_be_bytes());
        data.extend(gds_record::XY);
        self.points.iter().for_each(|point| {
            let x = point.x * scaling;
            let y = point.y * scaling;
            data.extend((f64::round(x) as i32).to_be_bytes());
            data.extend((f64::round(y) as i32).to_be_bytes());
        });
        if !self.points.is_empty() {
            data.extend((f64::round(self.points[0].x * scaling) as i32).to_be_bytes());
            data.extend((f64::round(self.points[0].y * scaling) as i32).to_be_bytes());
        }

        // endelement
        data.extend(4_i16.to_be_bytes());
        data.extend(gds_record::ENDEL);
        data
    }
}

#[derive(Default, Debug)]
pub struct Path {
    pub layer: i16,
    pub datatype: i16,
    pub width: f64,
    pub end_type: i16,
    pub points: Vec<Points>,
}

impl GdsObject for Path {
    fn to_gds(&self, scaling: f64) -> Vec<u8> {
        let mut data = Vec::<u8>::new();

        // path
        data.extend(4_i16.to_be_bytes());
        data.extend(gds_record::PATH);

        // layer
        data.extend(6_i16.to_be_bytes());
        data.extend(gds_record::LAYER);
        data.extend((self.layer as i16).to_be_bytes());

        // datatype
        data.extend(6_i16.to_be_bytes());
        data.extend(gds_record::DATATYPE);
        data.extend((self.datatype as i16).to_be_bytes());

        // endtype
        data.extend(6_i16.to_be_bytes());
        data.extend(gds_record::PATHTYPE);
        data.extend((self.end_type as u16).to_be_bytes());

        // width
        data.extend(8_i16.to_be_bytes());
        data.extend(gds_record::WIDTH);
        data.extend((f64::round(self.width * scaling) as u32).to_be_bytes());
        // TODO: if end_type == 4, which means path end is in extend mode, need to export extend data

        // points
        data.extend((4_i16 + 8 * self.points.len() as i16).to_be_bytes());
        data.extend(gds_record::XY);
        self.points.iter().for_each(|point| {
            let x = point.x * scaling;
            let y = point.y * scaling;
            data.extend((f64::round(x) as i32).to_be_bytes());
            data.extend((f64::round(y) as i32).to_be_bytes());
        });

        // endel
        data.extend(4_i16.to_be_bytes());
        data.extend(gds_record::ENDEL);

        data
    }
}

#[derive(Default, Debug)]
pub struct Ref {
    pub(crate) refed_cell: Rc<RefCell<Cell>>,
    pub reflection_x: bool,
    // pub abs_magnific: bool,
    pub magnific: f64,
    // pub abs_angel: bool,
    pub angle: f64, //measured in degrees and in the counterclockwise direction
    pub origin: Points,
    pub row: i16,
    pub column: i16,
    pub spaceing_row: Vector,
    pub spaceing_col: Vector,
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

impl GdsObject for Ref {
    fn to_gds(&self, scaling: f64) -> Vec<u8> {
        let mut data = Vec::<u8>::new();

        // sref or aref
        data.extend(4_i16.to_be_bytes());
        let mut is_array = false;
        if self.row != 0 || self.column != 0 {
            is_array = true;
        }

        if is_array {
            data.extend(gds_record::AREF);
        } else {
            data.extend(gds_record::SREF);
        }

        // refered cell name
        let mut cell_name = Vec::<u8>::new();
        cell_name.extend(gds_record::SNAME);

        let cell = &*(self.refed_cell.borrow());
        let mut name = ascii_string_to_be_bytes(&cell.name);
        if !name.len().is_power_of_two() {
            name.push(0);
        }
        cell_name.extend(name);

        data.extend((cell_name.len() as i16 + 2_i16).to_be_bytes());
        data.extend(cell_name);

        // strans
        data.extend(6_i16.to_be_bytes());
        data.extend(gds_record::STRANS);

        let mut flag: u16 = 0;
        if self.reflection_x {
            flag |= 0x8000
        }
        data.extend(flag.to_be_bytes());

        // magnification
        data.extend(12_u16.to_be_bytes());
        data.extend(gds_record::MAG);
        data.extend(f64_to_gds_bytes(self.magnific));

        // rotate
        data.extend(12_u16.to_be_bytes());
        data.extend(gds_record::ANGLE);
        data.extend(f64_to_gds_bytes(self.angle));

        if is_array {
            // colrow
            data.extend(8_u16.to_be_bytes());
            data.extend(gds_record::COLROW);
            data.extend((self.column as u16).to_be_bytes());
            data.extend((self.row as u16).to_be_bytes());
            // xy
            data.extend(28_u16.to_be_bytes());
            data.extend(gds_record::XY);
            data.extend((f64::round(self.origin.x * scaling) as i32).to_be_bytes());
            data.extend((f64::round(self.origin.y * scaling) as i32).to_be_bytes());
            // spaceing
            data.extend(
                (f64::round((self.spaceing_col.x * self.column as f64 + self.origin.x) * scaling)
                    as i32)
                    .to_be_bytes(),
            );
            data.extend(
                (f64::round((self.spaceing_col.y * self.column as f64 + self.origin.y) * scaling)
                    as i32)
                    .to_be_bytes(),
            );
            data.extend(
                (f64::round((self.spaceing_row.x * self.row as f64 + self.origin.x) * scaling)
                    as i32)
                    .to_be_bytes(),
            );
            data.extend(
                (f64::round((self.spaceing_row.y * self.row as f64 + self.origin.y) * scaling)
                    as i32)
                    .to_be_bytes(),
            );
        } else {
            data.extend(12_u16.to_be_bytes());
            data.extend(gds_record::XY);
            data.extend((f64::round(self.origin.x * scaling) as i32).to_be_bytes());
            data.extend((f64::round(self.origin.y * scaling) as i32).to_be_bytes());
        }

        // endel
        data.extend(4_u16.to_be_bytes());
        data.extend(gds_record::ENDEL);

        data
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

impl GdsObject for Text {
    fn to_gds(&self, scaling: f64) -> Vec<u8> {
        let mut data = Vec::<u8>::new();

        data.extend(4_i16.to_be_bytes());
        data.extend(gds_record::TEXT);
        data.extend(6_i16.to_be_bytes());
        data.extend(gds_record::LAYER);
        data.extend((self.layer as u16).to_be_bytes());
        data.extend(6_u16.to_be_bytes());
        data.extend(gds_record::TEXTTYPE);
        data.extend((self.datatype as u16).to_be_bytes());
        data.extend(6_u16.to_be_bytes());
        data.extend(gds_record::PRESENTATION);
        data.extend((gds_writer::text_anchor_to_gds_num(&self.anchor) as u16).to_be_bytes());

        let is_transform = self.rotation != 0.0 || self.magnification != 1.0 || self.x_reflection;

        if is_transform {
            data.extend(6_u16.to_be_bytes());
            data.extend(gds_record::STRANS);
            if self.x_reflection {
                data.extend((0x8000 as u16).to_be_bytes());
            } else {
                data.extend((0 as u16).to_be_bytes());
            }
            data.extend(12_u16.to_be_bytes());
            data.extend(gds_record::MAG);
            data.extend(gds_writer::f64_to_gds_bytes(self.magnification));
            data.extend(12_u16.to_be_bytes());
            data.extend(gds_record::ANGLE);
            data.extend(gds_writer::f64_to_gds_bytes(self.rotation));
        }
        // XY
        data.extend(12_u16.to_be_bytes());
        data.extend(gds_record::XY);
        data.extend((f64::round(self.position.x * scaling) as i32).to_be_bytes());
        data.extend((f64::round(self.position.y * scaling) as i32).to_be_bytes());

        // STRING
        let mut text_data = gds_writer::ascii_string_to_be_bytes(&self.text);
        if !text_data.len().is_power_of_two() {
            text_data.push(0);
        }

        data.extend((text_data.len() as u16 + 4_u16).to_be_bytes());
        data.extend(gds_record::STRING);
        data.extend(text_data);

        data.extend(4_u16.to_be_bytes());
        data.extend(gds_record::ENDEL);

        data
    }
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

#[cfg(test)]
mod test_gds_model {
    use std::borrow::Borrow;

    use super::*;
    #[test]
    fn test_lib_top_cell() {
        let mut gds_lib = Lib::default();
        let cell1 = Rc::new(RefCell::new(Cell::default()));
        let cell2 = Rc::new(RefCell::new(Cell::default()));
        let cell3 = Rc::new(RefCell::new(Cell::default()));
        cell1.borrow_mut().name = String::from("cell1");
        cell2.borrow_mut().name = String::from("cell2");
        cell3.borrow_mut().name = String::from("cell3");
        let mut ref3 = Ref::default();
        let mut ref2 = Ref::default();
        ref3.refed_cell = cell3.clone();
        ref2.refed_cell = cell2.clone();
        cell2.borrow_mut().refs.push(ref3);
        cell1.borrow_mut().refs.push(ref2);
        gds_lib.cells.push(cell1.clone());
        gds_lib.cells.push(cell2);
        gds_lib.cells.push(cell3);

        let top_cell = gds_lib.top_cells();
        assert_eq!(top_cell.len(), 1);
        assert!(Rc::ptr_eq(&top_cell[0], &cell1));
    }
}
