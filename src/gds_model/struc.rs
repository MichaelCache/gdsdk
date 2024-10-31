use rayon::prelude::*;

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
        if name.len() % 2 != 0 {
            name.push(0);
        }
        struc_name.extend(name);

        data.extend((struc_name.len() as i16 + 2_i16).to_be_bytes());
        data.extend(struc_name);

        for d in self
            .polygons
            .par_iter()
            .map(|p| p.to_gds(scaling).unwrap())
            .collect::<Vec<_>>()
        {
            data.extend(d);
        }

        for d in self
            .paths
            .par_iter()
            .map(|p| p.to_gds(scaling).unwrap())
            .collect::<Vec<_>>()
        {
            data.extend(d);
        }

        for d in self
            .refs
            .par_iter()
            .map(|p| p.to_gds(scaling).unwrap())
            .collect::<Vec<_>>()
        {
            data.extend(d);
        }

        for d in self
            .label
            .par_iter()
            .map(|p| p.to_gds(scaling).unwrap())
            .collect::<Vec<_>>()
        {
            data.extend(d);
        }

        // endstr
        let mut endstr_data = Vec::<u8>::new();
        endstr_data.extend(gds_record::ENDSTR);

        data.extend((endstr_data.len() as i16 + 2_i16).to_be_bytes());
        data.extend(endstr_data);

        Ok(data)
    }
}
