use super::gds_error::*;
use super::gds_model;
use super::gds_model::*;
use super::gds_record::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::slice::Iter;

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
            Record::BgnStr(_) => {
                let cell = parse_cell(iter, factor)?;
                if name_cell_map.contains_key(&(*cell).borrow_mut().name) {
                    return Err(Box::new(gds_err(&std::format!(
                        "Duplicated Cell \"{}\" found",
                        &(*cell).borrow_mut().name
                    ))));
                }
                let cell_name = (*cell).borrow_mut().name.clone();
                name_cell_map.insert(cell_name, cell);
            }
            Record::EndLib => {
                break;
            }
            other => {
                println!("get record from lib {:#?}", other);
            }
        }
    }

    // step.2 connect reference to cell, only add cell not be refered to lib
    let mut not_refered_cell = name_cell_map.clone();
    for c in &name_cell_map {
        let mut cell = c.1.borrow_mut();
        for refer in cell.refs.iter_mut() {
            if let gds_model::RefCell::CellName(name) = &refer.refed_cell {
                let refered_cell = name_cell_map.get(name).unwrap();
                not_refered_cell.remove(name);
                refer.refed_cell = gds_model::RefCell::Cell(refered_cell.clone());
            }
        }
    }

    for c in not_refered_cell {
        lib.cells.push(c.1);
    }

    Ok(lib)
}

fn parse_cell(
    iter: &mut Iter<'_, Record>,
    factor: f64,
) -> Result<Rc<RefCell<Cell>>, Box<dyn Error>> {
    let cell = Rc::new(RefCell::new(Cell::default()));
    while let Some(record) = iter.next() {
        match record {
            Record::BgnStr(date) => (*cell).borrow_mut().date = date.clone(), // last modification time of a structure and marks the beginning of a structure
            Record::StrName(s) => (*cell).borrow_mut().name = s.to_string(),
            Record::Boundary => {
                let polygon = parse_polygon(iter, factor)?;
                (*cell).borrow_mut().polygons.push(polygon);
            }
            Record::Path => {
                let path = parse_path(iter, factor)?;
                (*cell).borrow_mut().paths.push(path);
            }
            Record::StrRef => {
                let sref = parse_sref(iter, factor)?;
                (*cell).borrow_mut().refs.push(sref);
            }
            Record::Text => {
                let text = parse_text(iter, factor)?;
                (*cell).borrow_mut().label.push(text)
            }
            Record::AryRef => {
                let aref = parse_aref(iter, factor)?;
                (*cell).borrow_mut().refs.push(aref);
            }
            Record::EndStr => {
                break;
            }
            other => {
                println!("get record from cell {:#?}", other);
            }
        }
    }

    Ok(cell)
}

fn parse_text(iter: &mut Iter<'_, Record>, factor: f64) -> Result<Text, Box<dyn Error>> {
    let mut text = Text::default();
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
            Record::MAG(mag) => text.magnification = *mag,
            Record::Angle(angle) => text.rotation = std::f64::consts::PI / 180.0 * angle,
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
    while let Some(record) = iter.next() {
        match record {
            Record::Boundary => (), //marks the beginning of a boundary element
            Record::Layer(l) => polygon.layer = *l,
            Record::DataType(d) => polygon.datatype = *d,
            Record::Points(points) => {
                polygon.points = i32_vec_2_pointvec(points, factor);
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
            Record::EndElem => break,
            other => {
                println!("get record from path {:#?}", other);
            }
        }
    }
    Ok(path)
}

fn parse_sref(iter: &mut Iter<'_, Record>, factor: f64) -> Result<Ref, Box<dyn Error>> {
    let mut sref = Ref::new();
    while let Some(record) = iter.next() {
        match record {
            Record::StrRef => (), // marks the beginning of an SREF(structure reference) element
            Record::StrRefName(s) => sref.refed_cell = gds_model::RefCell::CellName(s.to_string()),
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
            Record::MAG(mag) => sref.magnific = *mag,
            Record::Angle(angle) => sref.angle = std::f64::consts::PI / 180.0 * angle,
            Record::Points(points) => {
                sref.origin = Points::new(points[0].0 as f64 * factor, points[0].1 as f64 * factor)
            }
            Record::EndElem => break,
            other => {
                println!("get record from ref {:#?}", other);
            }
        }
    }
    Ok(sref)
}

fn parse_aref(iter: &mut Iter<'_, Record>, factor: f64) -> Result<Ref, Box<dyn Error>> {
    let mut aref = Ref::new();
    while let Some(record) = iter.next() {
        match record {
            Record::AryRef => (), // marks the beginning of an SREF(structure reference) element
            Record::StrRefName(s) => aref.refed_cell = gds_model::RefCell::CellName(s.to_string()),
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
            Record::MAG(mag) => aref.magnific = *mag,
            Record::Angle(angle) => aref.angle = std::f64::consts::PI / 180.0 * angle,
            Record::COLROW { column, row } => {
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
            Record::EndElem => break,
            other => {
                println!("get record from ref {:#?}", other);
            }
        }
    }
    Ok(aref)
}

fn i32_vec_2_pointvec(vec: &Vec<(i32, i32)>, factor: f64) -> Vec<Points> {
    vec.iter()
        .map(|&(x, y)| Points::new(x as f64 * factor, y as f64 * factor))
        .collect()
}
