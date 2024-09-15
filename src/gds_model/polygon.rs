use super::*;
use crate::gds_record;

#[derive(Default, Debug)]
pub struct Polygon {
    pub layer: i16,
    pub datatype: i16,
    pub points: Vec<Points>,
    /// gds property, key is int value, value is max 128 bytes length ASCII str
    pub property: Property,
}

impl GdsObject for Polygon {
    fn to_gds(&self, scaling: f64) -> Result<Vec<u8>, Box<dyn Error>> {
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
            gds_err!(&format!(
                "Gds polygons can not have points more than 8190 count:{:#?}",
                &self
            ));
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

        // properties
        data.extend(self.property.to_gds(scaling)?);

        // endelement
        data.extend(4_i16.to_be_bytes());
        data.extend(gds_record::ENDEL);
        Ok(data)
    }
}
