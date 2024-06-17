use super::*;
use crate::gds_error;
use crate::gds_record;
use crate::gds_writer;

use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Property(pub HashMap<i16, String>);

impl GdsObject for Property {
    fn to_gds(&self, _: f64) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut data = Vec::<u8>::new();
        // properties
        for prop in &self.0 {
            data.extend(6_i16.to_be_bytes());
            data.extend(gds_record::PROPATTR);
            if *prop.0 < 1 || *prop.0 > 126 {
                return Err(Box::new(gds_error::gds_err(&format!(
                    "Gds property attribute value can not be out of range [1:126]:{:#?}",
                    &self
                ))));
            }
            data.extend(prop.0.to_be_bytes());

            let mut prop_value = Vec::<u8>::new();
            prop_value.extend(gds_record::PROPVALUE);
            let mut value = gds_writer::ascii_string_to_be_bytes(&prop.1);
            if !value.len().is_power_of_two() {
                value.push(0);
            }
            if value.len() > 128 {
                return Err(Box::new(gds_error::gds_err(&format!(
                    "Gds property value can not have ascii char more than 128 count:{:#?}",
                    &self
                ))));
            }
            prop_value.extend(value);

            data.extend((prop_value.len() as i16 + 2_i16).to_be_bytes());
            data.extend(prop_value);
        }
        Ok(data)
    }
}
