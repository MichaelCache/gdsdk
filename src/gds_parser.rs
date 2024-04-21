use super::gds_error::*;
use super::gds_model;
use super::gds_model::*;
use super::gds_record::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::slice::Iter;
use std::error::Error;

pub fn parse_gds(records: &[Record]) -> Result<Box<Lib>, Box<dyn Error>> {
    let mut lib: Box<Lib> = Box::new(Lib::default());
    let mut iter = records.iter();
    while let Some(record) = iter.next() {
        match record {
            Record::Header { version: _ } => {}
            Record::BgnLib(_) => lib = parse_lib(&mut iter)?,
            Record::EndLib => {}
            _ => return Err(Box::new(gds_err("not valid gds lib"))),
        }
    }

    Ok(lib)
}

fn parse_lib(iter: &mut Iter<'_, Record>) -> Result<Box<Lib>, Box<dyn Error>> {
    let mut lib = Box::new(Lib::default());
    let mut factor = 0.0;
    let mut name_cell_map = HashMap::new();
    let mut cell_ref_cellname_map =HashMap::<String, Vec::<(gds_model::Ref, String)>>::new();
    // step.1 parse all cell, save to name_cell_map
    while let Some(record) = iter.next() {
        match record {
            Record::BgnLib(date) => lib.date = date.clone(), //modification time of lib, and marks beginning of library
            Record::LibName(s) => lib.name = s.to_string(),
            Record::Units {
                unit_in_meter,
                precision,
            } => {
                lib.units = precision / unit_in_meter;
                if lib.units.is_infinite() {
                    return Err(Box::new(gds_err("Lib units is infinite")));
                }
                if lib.units.is_nan() {
                    return Err(Box::new(gds_err("Lib units is nan")));
                }

                lib.precision = *precision;
                factor = *unit_in_meter;
            }
            Record::BgnStr(date) => {
                let (cell, cell_refs) = parse_cell(iter, factor)?;
                cell.borrow_mut().date = date.clone();
                let cell_name = cell.borrow().name.clone();
                if name_cell_map.contains_key(&cell_name) {
                    return Err(Box::new(gds_err(&std::format!(
                        "Duplicated Cell \"{}\" found",
                        &cell_name
                    ))));
                }
                let cell_name_cp = cell_name.clone();
                name_cell_map.insert(cell_name, cell);

                cell_ref_cellname_map.insert(cell_name_cp, cell_refs);
            }
            Record::EndLib => {
                break;
            }
            other => {
                println!("get record from lib {:#?}", other);
            }
        }
    }

    // step.2 connect reference to cell,
    for c in cell_ref_cellname_map {
        let cur_cell_name = &c.0;
        let cur_cell = name_cell_map.get(cur_cell_name).unwrap().clone();
        let mut mut_cur_cell = cur_cell.borrow_mut();
        for r in c.1{
            // ref refer to cell
            let mut cell_ref = r.0;
            let ref_cell_name = &r.1;
            let refed_cell = name_cell_map.get(ref_cell_name).unwrap().clone();
            cell_ref.refed_cell = refed_cell;
            // current cell add refs
            mut_cur_cell.refs.push(cell_ref);
        }
    }

    // step.3 add all cell to lib
    for c in name_cell_map{
        lib.cells.push(c.1);
    }

    Ok(lib)
}

fn parse_cell(
    iter: &mut Iter<'_, Record>,
    factor: f64
) -> Result<(Rc<RefCell<Cell>>,Vec::<(gds_model::Ref, String)>), Box<dyn Error>> {
    let cell = Rc::new(RefCell::new(Cell::default()));
    let mut mut_cell = cell.borrow_mut();
    let mut ref_refname = Vec::<(gds_model::Ref, String)>::new();
    while let Some(record) = iter.next() {
        match record {
            Record::BgnStr(date) => mut_cell.date = date.clone(), // last modification time of a structure and marks the beginning of a structure
            Record::StrName(s) => mut_cell.name = s.to_string(),
            Record::Boundary | Record::Box => {
                let polygon = parse_polygon(iter, factor)?;
                mut_cell.polygons.push(polygon);
            }
            Record::Path => {
                let path = parse_path(iter, factor)?;
                mut_cell.paths.push(path);
            }
            Record::StrRef => {
                let (sref, ref_cellname) = parse_sref(iter, factor)?;
                ref_refname.push((sref, ref_cellname));
            }
            Record::Text => {
                let text = parse_text(iter, factor)?;
                mut_cell.label.push(text)
            }
            Record::AryRef => {
                let (aref, ref_cellname)  = parse_aref(iter, factor)?;
                ref_refname.push((aref, ref_cellname));
            }
            Record::EndStr => {
                break;
            }
            other => {
                println!("get record from cell {:#?}", other);
            }
        }
    }
    std::mem::drop(mut_cell);

    Ok((cell, ref_refname))
}

fn parse_text(iter: &mut Iter<'_, Record>, factor: f64) -> Result<Text, Box<dyn Error>> {
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
                    text.property.insert(key, value.to_string());   
                }else{
                    return Err(Box::new(gds_err(&std::format!(
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

fn parse_polygon(iter: &mut Iter<'_, Record>, factor: f64) -> Result<Polygon, Box<dyn Error>> {
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
                    polygon.property.insert(key, value.to_string());   
                }else{
                    return Err(Box::new(gds_err(&std::format!(
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

fn parse_path(iter: &mut Iter<'_, Record>, factor: f64) -> Result<Path, Box<dyn Error>> {
    let mut path = Path::default();
    let mut cur_prokey : Option<i16>= None;
    while let Some(record) = iter.next() {
        match record {
            Record::Path => (), // marks the beginning of a path element
            Record::Layer(l) => path.layer = *l,
            Record::DataType(d) => path.datatype = *d,
            Record::Width(w) => path.width = *w as f64 * factor,
            Record::PathType(t) => path.end_type = *t,
            Record::Points(points) => {
                path.points = i32_vec_2_pointvec(points, factor);
            }
            Record::PropAttr(key)=>cur_prokey = Some(*key),
            Record::PropValue(value) => {
                if let Some(key) = cur_prokey{
                    path.property.insert(key, value.to_string());   
                }else{
                    return Err(Box::new(gds_err(&std::format!(
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

fn parse_sref(iter: &mut Iter<'_, Record>, factor: f64) -> Result<(Ref, String), Box<dyn Error>> {
    let mut sref = Ref::new();
    let mut ref_cell_name = String::new();
    let mut cur_prokey : Option<i16>= None;
    while let Some(record) = iter.next() {
        match record {
            Record::StrRef => (), // marks the beginning of an SREF(structure reference) element
            Record::StrRefName(s) => ref_cell_name = s.to_string(),
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
                    sref.property.insert(key, value.to_string());   
                }else{
                    return Err(Box::new(gds_err(&std::format!(
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
    // ref_map.insert(ref_cell_name, &mut sref);
    Ok((sref,ref_cell_name))
}

fn parse_aref(iter: &mut Iter<'_, Record>, factor: f64) -> Result<(Ref, String), Box<dyn Error>> {
    let mut aref = Ref::new();
    let mut ref_cellname = String::new();
    let mut cur_prokey : Option<i16>= None;
    while let Some(record) = iter.next() {
        match record {
            Record::AryRef => (), // marks the beginning of an SREF(structure reference) element
            Record::StrRefName(s) =>ref_cellname=s.to_string(),
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
                    aref.property.insert(key, value.to_string());   
                }else{
                    return Err(Box::new(gds_err(&std::format!(
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
    Ok((aref, ref_cellname))
}

fn i32_vec_2_pointvec(vec: &[(i32, i32)], factor: f64) -> Vec<Points> {
    vec.iter()
        .map(|&(x, y)| Points::new(x as f64 * factor, y as f64 * factor))
        .collect()
}
