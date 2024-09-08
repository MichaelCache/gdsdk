use std::sync::Arc;
use std::sync::RwLock;

use super::*;
use crate::gds_record;
use crate::gds_writer;

/// Gds ArrayRef or StructurRef
/// refer Gds Structure
#[derive(Debug)]
pub struct Ref {
    pub refed_struc: Arc<RwLock<Struc>>,
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
    pub property: Property,
}

impl Ref {
    pub fn new(refto: &Arc<RwLock<Struc>>) -> Self {
        Ref {
            refed_struc: refto.clone(),
            reflection_x: false,
            magnific: 1.0,
            angle: 0.0,
            origin: Points::new(0.0, 0.0),
            row: 0,
            column: 0,
            spaceing_row: Vector { x: 0.0, y: 0.0 },
            spaceing_col: Vector { x: 0.0, y: 0.0 },
            property: Property::default(),
        }
    }
}

impl GdsObject for Ref {
    fn to_gds(&self, scaling: f64) -> Result<Vec<u8>, Box<dyn Error>> {
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

        // refered gds structure name
        let mut struc_name = Vec::<u8>::new();
        struc_name.extend(gds_record::SNAME);

        let struc = &*(self.refed_struc.read().unwrap());
        let mut name = gds_writer::ascii_string_to_be_bytes(&struc.name);
        if !name.len().is_power_of_two() {
            name.push(0);
        }
        struc_name.extend(name);

        data.extend((struc_name.len() as i16 + 2_i16).to_be_bytes());
        data.extend(struc_name);

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
        data.extend(gds_writer::f64_to_gds_bytes(self.magnific));

        // rotate
        data.extend(12_u16.to_be_bytes());
        data.extend(gds_record::ANGLE);
        data.extend(gds_writer::f64_to_gds_bytes(self.angle));

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

        // properties
        data.extend(self.property.to_gds(scaling)?);

        // endel
        data.extend(4_u16.to_be_bytes());
        data.extend(gds_record::ENDEL);

        Ok(data)
    }
}

// FakeRef only used for gdsii file parse, cache Ref data
pub(crate) struct FakeRef {
    pub refed_struc_name: String,
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
    pub property: Property,
}

impl FakeRef {
    pub(crate) fn new() -> Self {
        FakeRef {
            refed_struc_name: String::new(),
            reflection_x: false,
            magnific: 1.0,
            angle: 0.0,
            origin: Points::new(0.0, 0.0),
            row: 0,
            column: 0,
            spaceing_row: Vector { x: 0.0, y: 0.0 },
            spaceing_col: Vector { x: 0.0, y: 0.0 },
            property: Property::default(),
        }
    }

    pub(crate) fn create_true_ref(self, struc: &Arc<RwLock<Struc>>) -> Ref {
        let mut struc_ref = Ref::new(struc);
        struc_ref.reflection_x = self.reflection_x;
        struc_ref.magnific = self.magnific;
        struc_ref.angle = self.angle;
        struc_ref.origin = self.origin;
        struc_ref.row = self.row;
        struc_ref.column = self.column;
        struc_ref.spaceing_row = self.spaceing_row;
        struc_ref.spaceing_col = self.spaceing_col;
        struc_ref.property = self.property;
        struc_ref
    }
}
