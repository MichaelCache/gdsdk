use std::collections::HashMap;

use super::*;
use crate::gds_error;
use crate::gds_record;
use crate::gds_writer;

#[repr(i16)]
#[derive(Debug)]
pub enum PathEndType {
    Square = 0,
    Round = 1,
    SquareHalfWidth = 2,
    SquareExtend = 4,
}

impl Default for PathEndType {
    fn default() -> Self {
        PathEndType::Square
    }
}

impl TryFrom<&i16> for PathEndType {
    type Error = gds_error::GDSIIError;
    fn try_from(value: &i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PathEndType::Square),
            1 => Ok(PathEndType::Round),
            2 => Ok(PathEndType::SquareHalfWidth),
            4 => Ok(PathEndType::SquareExtend),
            _ => Err(gds_error::gds_err(&format!(
                "not valid path end type value: {}",
                value
            ))),
        }
    }
}

impl From<&PathEndType> for i16 {
    fn from(value: &PathEndType) -> Self {
        match value {
            PathEndType::Square => 0,
            PathEndType::Round => 1,
            PathEndType::SquareHalfWidth => 2,
            PathEndType::SquareExtend => 4,
        }
    }
}

/// Gds Path
#[derive(Default, Debug)]
pub struct Path {
    pub layer: i16,
    pub datatype: i16,
    pub width: f64,
    pub end_type: PathEndType,
    pub points: Vec<Points>,
    pub property: HashMap<i16, String>,
}

impl GdsObject for Path {
    fn to_gds(&self, scaling: f64) -> Result<Vec<u8>, Box<dyn Error>> {
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
        data.extend(i16::from(&self.end_type).to_be_bytes());

        // width
        data.extend(8_i16.to_be_bytes());
        data.extend(gds_record::WIDTH);
        data.extend((f64::round(self.width * scaling) as u32).to_be_bytes());
        // TODO: if end_type == 4, which means path end is in extend mode, need to export extend data
        if let PathEndType::SquareExtend = self.end_type {
            gds_error::gds_err(&format!(
                "end_type == 4 is not support for path now: {:#?}",
                &self
            ));
        }

        // points
        data.extend((4_i16 + 8 * self.points.len() as i16).to_be_bytes());
        data.extend(gds_record::XY);
        self.points.iter().for_each(|point| {
            let x = point.x * scaling;
            let y = point.y * scaling;
            data.extend((f64::round(x) as i32).to_be_bytes());
            data.extend((f64::round(y) as i32).to_be_bytes());
        });

        // properties
        for prop in &self.property {
            data.extend(6_i16.to_be_bytes());
            data.extend(gds_record::PROPATTR);
            data.extend(prop.0.to_be_bytes());

            let mut prop_value = Vec::<u8>::new();
            prop_value.extend(gds_record::PROPVALUE);
            let mut value = gds_writer::ascii_string_to_be_bytes(&prop.1);
            if !value.len().is_power_of_two() {
                value.push(0);
            }
            if value.len() > 128 {
                gds_error::gds_err(&format!(
                    "Gds Path property can not have ascii char more than 128 count:{:#?}",
                    &self
                ));
            }
            prop_value.extend(value);

            data.extend((prop_value.len() as i16 + 2_i16).to_be_bytes());
            data.extend(prop_value);
        }

        // endel
        data.extend(4_i16.to_be_bytes());
        data.extend(gds_record::ENDEL);

        Ok(data)
    }
}
