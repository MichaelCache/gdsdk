use super::gds_err;

use super::gds_model;
use super::gds_model::*;
use super::gds_record::*;

use rayon::prelude::*;
use std::collections::HashMap;
use std::slice::Iter;
use std::error::Error;
use std::sync::{Arc,RwLock};

pub fn parse_gds(records: Vec<Record>) -> Result<Box<Lib>, Box<dyn Error+Send+Sync>> {
    // first record should be gds header with version info
    if let Some(Record::Header{version:ver}) =records.first() {
        println!("read GDSII version: {}", ver);
    }
    else {
        return Err(Box::new(gds_err!("GDSII version not found")));
    }

    // second record should be BgnLib, all data between BgnLib and EndLib is belong to this lib
    // which EndLib should be the last record
    if let Some(Record::BgnLib(_)) = records.get(1){}
    else {
        return Err( Box::new(gds_err!("no valid gds lib found")));
    }
    if let Record::EndLib = records.last().unwrap(){}else{
        return Err( Box::new(gds_err!("no valid gds lib found")));
    }
    
    let lib = parse_lib(records)?;
    return Ok(lib);
    
}

fn parse_lib(records: Vec<Record>) -> Result<Box<Lib>, Box<dyn Error+Send+Sync>> {
    let mut lib = Box::new(Lib::new(""));
    let mut factor = 0.0;
    let mut strucs_id_range =Vec::<(Option<usize>, Option<usize>)>::new();
    // 
    for (idx, rec) in records.iter().enumerate() {
        match rec {
            Record::BgnLib(date) => lib.date = date.clone(), //modification time of lib, and marks beginning of library
            Record::LibName(s) => lib.name = s.to_string(),
            Record::Units {
                unit_in_meter,
                precision,
            } => {
                lib.units = precision / unit_in_meter;
                if lib.units.is_infinite() {
                    return Err(Box::new(gds_err!("Lib units is infinite")));
                }
                if lib.units.is_nan() {
                    return Err(Box::new(gds_err!("Lib units is nan")));
                }

                lib.precision = *precision;
                factor = *unit_in_meter;
            }
            // record gds structure start and end range
            Record::BgnStr(_) => {
                strucs_id_range.push((None, None));
                strucs_id_range.last_mut().unwrap().0 = Some(idx);
            }
            Record::EndStr => {  
                strucs_id_range.last_mut().unwrap().1 = Some(idx); 
            }
            Record::EndLib => {
                break;
            }
            _ => {}
        }
    }

    // check gds structure range valid
    // which means: 1. all range idx is Some(i64). 2. range not overlap
    for i in 0..strucs_id_range.len() {
        let (start, end) = strucs_id_range[i];
        if start.is_none() || end.is_none() {
            return Err(Box::new(gds_err!("Invalid gds structure range found")));
        }

        if i>0 {
             if let (Some(pre_start), Some(pre_end)) = strucs_id_range[i-1]{
                if pre_start.max(start.unwrap()) < pre_end.min(end.unwrap()) {
                    return Err(Box::new(gds_err!(
                        "Gds structure range overlap"
                    )));
                }
             }   
        }
    }

    
   // step.1 parse all stuc, save to name_stuc_map
    let name_struc_map = Arc::new(RwLock::new( HashMap::<String, Arc<RwLock<Struc>>>::new()));
    let struc_ref_strucname_map =Arc::new(RwLock::new(HashMap::<String, Vec::<gds_model::FakeRef>>::new()));
   

    strucs_id_range.par_iter().for_each(|(start, end)|{
        let mut write_name_struc_map = name_struc_map.write().unwrap();
        let mut write_struc_ref_strucname_map = struc_ref_strucname_map.write().unwrap();
        let mut iter = records[start.unwrap() as usize..end.unwrap() as usize].iter();
        let (struc, fack_refs) = parse_struc( &mut iter, factor).unwrap();
        let struc_name = struc.read().unwrap().name.clone();
        if write_name_struc_map.contains_key(&struc_name) {
                           return;
                        }
        write_name_struc_map.insert(struc_name.clone(), struc);
        write_struc_ref_strucname_map.insert(struc_name, fack_refs);
    });
    

    // step.2 connect reference to struc
    let mut write_struc_ref_strucname_map = struc_ref_strucname_map.write().unwrap();
    let read_name_struc_map = name_struc_map.read().unwrap();
    write_struc_ref_strucname_map.par_drain().for_each(|(struc_name, fack_refs)|{
        let cur_struc = read_name_struc_map.get(&struc_name).unwrap().clone();
        let mut mut_cur_struc = cur_struc.write().unwrap();
        for fack_ref in fack_refs{
            let ref_struc = read_name_struc_map.get(&fack_ref.refed_struc_name).unwrap().clone();
            let struc_ref = fack_ref.create_true_ref(&ref_struc);
            mut_cur_struc.refs.push(struc_ref);
        }
    });

    // step.3 add all struc to lib
    let read_name_struc_map = name_struc_map.read().unwrap();
    for c in read_name_struc_map.iter(){
        lib.add_struc(&c.1)?;
    }

    Ok(lib)
}

fn parse_struc(
    iter: &mut Iter<'_, Record>,
    factor: f64
) -> Result<(Arc<RwLock<Struc>>,Vec::<gds_model::FakeRef>), Box<dyn Error+Send+Sync>> {
    let struc_ptr = Arc::new(RwLock::new(Struc::new("")));
    let mut ref_refname = Vec::<gds_model::FakeRef>::new();
    let mut struc = struc_ptr.write().unwrap();
    while let Some(record) = iter.next() {
        match record {
            Record::BgnStr(date) => struc.date = date.clone(), // last modification time of a structure and marks the beginning of a structure
            Record::StrName(s) => struc.name = s.to_string(),
            Record::Boundary | Record::Box => {
                let polygon = parse_polygon(iter, factor)?;
                struc.polygons.push(polygon);
            }
            Record::Path => {
                let path = parse_path(iter, factor)?;
                struc.paths.push(path);
            }
            Record::StrRef => {
                let sref = parse_sref(iter, factor)?;
                ref_refname.push(sref);
            }
            Record::Text => {
                let text = parse_text(iter, factor)?;
                struc.label.push(text)
            }
            Record::AryRef => {
                let aref  = parse_aref(iter, factor)?;
                ref_refname.push(aref );
            }
            Record::EndStr => {
                break;
            }
            other => {
                println!("get record from struc {:#?}", other);
            }
        }
    }
    drop(struc);

    Ok((struc_ptr, ref_refname))
}

fn parse_text(iter: &mut Iter<'_, Record>, factor: f64) -> Result<Text, Box<dyn Error+Send+Sync>> {
    let mut text = Text::default();
    let mut cur_prokey : Option<i16>= None;
    while let Some(record) = iter.next() {
        match record {
            Record::Text => (), //marks the beginning of a text element
            Record::Layer(l) => text.layer = *l,
            Record::TextType(d) => text.datatype = *d,
            Record::Presentation {
                font_num,
                vertival_justfication,
                horizontal_justfication,
            } => {
                match font_num {
                    PresentationFont::Fonts0 => text.font = TextFont::Fonts0,
                    PresentationFont::Fonts1 => text.font = TextFont::Fonts1,
                    PresentationFont::Fonts2 => text.font = TextFont::Fonts2,
                    PresentationFont::Fonts3 => text.font = TextFont::Fonts3,
                };
                match vertival_justfication {
                    PresentationVerticalPos::Top => match horizontal_justfication {
                        PresentationHorizontalPos::Left => text.anchor = TextAnchor::NW,
                        PresentationHorizontalPos::Center => text.anchor = TextAnchor::N,
                        PresentationHorizontalPos::Right => text.anchor = TextAnchor::NE,
                    },
                    PresentationVerticalPos::Middle => match horizontal_justfication {
                        PresentationHorizontalPos::Left => text.anchor = TextAnchor::W,
                        PresentationHorizontalPos::Center => text.anchor = TextAnchor::O,
                        PresentationHorizontalPos::Right => text.anchor = TextAnchor::E,
                    },
                    PresentationVerticalPos::Bottom => match horizontal_justfication {
                        PresentationHorizontalPos::Left => text.anchor = TextAnchor::SW,
                        PresentationHorizontalPos::Center => text.anchor = TextAnchor::S,
                        PresentationHorizontalPos::Right => text.anchor = TextAnchor::SE,
                    },
                }
            }
            Record::String(content) => text.text = content.clone(),
            Record::Mag(mag) => text.magnification = *mag,
            Record::Angle(angle) => text.rotation = std::f64::consts::PI / 180.0 * angle,
            // TODO:
            Record::RefTrans {
                reflection_x,
                ..
                // absolute_magnification,
                // absolute_angle,
            } => text.x_reflection = *reflection_x,
            Record::Points(points) => {
                text.position =
                    Points::new(points[0].0 as f64 * factor, points[0].1 as f64 * factor)
            }
            Record::PropAttr(key)=>cur_prokey = Some(*key),
            Record::PropValue(value) => {
                if let Some(key) = cur_prokey{
                    text.property.0.insert(key, value.to_string());   
                }else{
                    return Err(Box::new(gds_err!(&std::format!(
                        "Text Property value \"{}\" have no key",
                        &value))));
                }
            }
            Record::EndElem => break,
            other => {
                println!("get record from Text {:#?}", other);
            }
        }
    }
    Ok(text)
}

fn parse_polygon(iter: &mut Iter<'_, Record>, factor: f64) -> Result<Polygon, Box<dyn Error+Send+Sync>> {
    let mut polygon = Polygon::default();
    let mut cur_prokey : Option<i16>= None;
    while let Some(record) = iter.next() {
        match record {
            Record::Boundary => (), //marks the beginning of a boundary element
            Record::Layer(l) => polygon.layer = *l,
            Record::DataType(d) | Record::BoxType(d)=> polygon.datatype = *d,
            Record::Points(points) => {
                if let Some((_, elements)) = points.split_last() {
                    // gds polygon last points is same with first one, so slice it
                    polygon.points = i32_vec_2_pointvec(elements, factor);
                }                
            }
            Record::PropAttr(key)=>cur_prokey = Some(*key),
            Record::PropValue(value) => {
                if let Some(key) = cur_prokey{
                    polygon.property.0.insert(key, value.to_string());   
                }else{
                    return Err(Box::new(gds_err!(&std::format!(
                        "Polygon Property value \"{}\" have no key",
                        &value))));
                }
            }
            Record::EndElem => break,
            other => {
                println!("get record from polygon {:#?}", other);
            }
        }
    }
    Ok(polygon)
}

fn parse_path(iter: &mut Iter<'_, Record>, factor: f64) -> Result<Path, Box<dyn Error+Send+Sync>> {
    let mut path = Path::default();
    let mut cur_prokey : Option<i16>= None;
    while let Some(record) = iter.next() {
        match record {
            Record::Path => (), // marks the beginning of a path element
            Record::Layer(l) => path.layer = *l,
            Record::DataType(d) => path.datatype = *d,
            Record::Width(w) => path.width = *w as f64 * factor,
            Record::PathType(t) => path.end_type = t.try_into()?,
            Record::Points(points) => {
                path.points = i32_vec_2_pointvec(points, factor);
            }
            Record::PropAttr(key)=>cur_prokey = Some(*key),
            Record::PropValue(value) => {
                if let Some(key) = cur_prokey{
                    path.property.0.insert(key, value.to_string());   
                }else{
                    return Err(Box::new(gds_err!(&std::format!(
                        "Path Property value \"{}\" have no key",
                        &value))));
                }
            }
            Record::EndElem => break,
            other => {
                println!("get record from path {:#?}", other);
            }
        }
    }
    Ok(path)
}

fn parse_sref(iter: &mut Iter<'_, Record>, factor: f64) -> Result<FakeRef, Box<dyn Error+Send+Sync>> {
    let mut sref = FakeRef::new();
    let mut cur_prokey : Option<i16>= None;
    while let Some(record) = iter.next() {
        match record {
            Record::StrRef => (), // marks the beginning of an SREF(structure reference) element
            Record::StrRefName(s) => sref.refed_struc_name = s.to_string(),
            Record::RefTrans {
                reflection_x,
                ..
                // absolute_magnification,
                // absolute_angle,
            } => {
                sref.reflection_x = *reflection_x;
                // sref.abs_magnific = *absolute_magnification;
                // sref.abs_angel = *absolute_angle;
            }
            Record::Mag(mag) => sref.magnific = *mag,
            Record::Angle(angle) => sref.angle = std::f64::consts::PI / 180.0 * angle,
            Record::Points(points) => {
                sref.origin = Points::new(points[0].0 as f64 * factor, points[0].1 as f64 * factor)
            }
            Record::PropAttr(key)=>cur_prokey = Some(*key),
            Record::PropValue(value) => {
                if let Some(key) = cur_prokey{
                    sref.property.0.insert(key, value.to_string());   
                }else{
                    return Err(Box::new(gds_err!(&std::format!(
                        "Ref Property value \"{}\" have no key",
                        &value))));
                }
            }
            Record::EndElem => break,
            other => {
                println!("get record from ref {:#?}", other);
            }
        }
    }
    Ok(sref)
}

fn parse_aref(iter: &mut Iter<'_, Record>, factor: f64) -> Result<FakeRef, Box<dyn Error+Send+Sync>> {
    let mut aref = FakeRef::new();
    let mut cur_prokey : Option<i16>= None;
    while let Some(record) = iter.next() {
        match record {
            Record::AryRef => (), // marks the beginning of an SREF(structure reference) element
            Record::StrRefName(s) =>aref.refed_struc_name=s.to_string(),
            Record::RefTrans {
                reflection_x,
                ..
                // absolute_magnification,
                // absolute_angle,
            } => {
                aref.reflection_x = *reflection_x;
                // aref.abs_magnific = *absolute_magnification;
                // aref.abs_angel = *absolute_angle;
            }
            Record::Mag(mag) => aref.magnific = *mag,
            Record::Angle(angle) => aref.angle = std::f64::consts::PI / 180.0 * angle,
            Record::ColRow { column, row } => {
                aref.column = *column;
                aref.row = *row;
            }
            Record::Points(points) => {
                aref.origin = Points::new(points[0].0 as f64 * factor, points[0].1 as f64 * factor);
                aref.spaceing_row =
                    Vector::new((points[2].0 as f64 * factor - aref.origin.x)/aref.row as f64, 
                    (points[2].1 as f64 * factor-aref.origin.y)/aref.row as f64);
                aref.spaceing_col =
                    Vector::new((points[1].0 as f64 * factor - aref.origin.x)/aref.column as f64, 
                    (points[1].1 as f64 * factor- aref.origin.y)/aref.column as f64);
            }
            Record::PropAttr(key)=>cur_prokey = Some(*key),
            Record::PropValue(value) => {
                if let Some(key) = cur_prokey{
                    aref.property.0.insert(key, value.to_string());   
                }else{
                    return Err(Box::new(gds_err!(&std::format!(
                        "Ref Property value \"{}\" have no key",
                        &value))));
                }
            }
            Record::EndElem => break,
            other => {
                println!("get record from ref {:#?}", other);
            }
        }
    }
    Ok(aref)
}

fn i32_vec_2_pointvec(vec: &[(i32, i32)], factor: f64) -> Vec<Points> {
    vec.iter()
        .map(|&(x, y)| Points::new(x as f64 * factor, y as f64 * factor))
        .collect()
}
