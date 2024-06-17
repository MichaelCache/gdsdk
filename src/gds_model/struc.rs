use super::*;
use crate::gds_record;
use crate::gds_writer;

/// Gds Structure
#[derive(Debug)]
pub struct Struc {
    pub name: String,
    pub polygons: Vec<Polygon>,
    pub paths: Vec<Path>,
    pub refs: Vec<Ref>,
    pub label: Vec<Text>,
    pub date: Date,
}

impl Struc {
    pub fn new(name: &str) -> Self {
        Struc {
            name: name.to_string(),
            polygons: Vec::<Polygon>::new(),
            paths: Vec::<Path>::new(),
            refs: Vec::<Ref>::new(),
            label: Vec::<Text>::new(),
            date: Date::now(),
        }
    }
}

impl GdsObject for Struc {
    fn to_gds(&self, scaling: f64) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut data = Vec::<u8>::new();
        // bgnstr and date
        let mut structure_data = Vec::<u8>::new();
        structure_data.extend(gds_record::BGNSTR);
        structure_data.extend(self.date.to_gds(scaling)?);

        data.extend((structure_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(structure_data);

        // gds struc name
        let mut struc_name = Vec::<u8>::new();
        struc_name.extend(gds_record::STRNAME);
        let mut name = gds_writer::ascii_string_to_be_bytes(&self.name);
        if !name.len().is_power_of_two() {
            name.push(0);
        }
        struc_name.extend(name);

        data.extend((struc_name.len() as i16 + 2_i16).to_be_bytes());
        data.extend(struc_name);

        for p in &self.polygons {
            let polygon_byte = p.to_gds(scaling)?;
            data.extend(polygon_byte);
        }

        for p in &self.paths {
            let path_byte = p.to_gds(scaling)?;
            data.extend(path_byte);
        }

        for r in &self.refs {
            let ref_byte = r.to_gds(scaling)?;
            data.extend(ref_byte);
        }

        for l in &self.label {
            let ref_byte = l.to_gds(scaling)?;
            data.extend(ref_byte);
        }

        // endstr
        let mut endstr_data = Vec::<u8>::new();
        endstr_data.extend(gds_record::ENDSTR);

        data.extend((endstr_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(endstr_data);

        Ok(data)
    }
}
