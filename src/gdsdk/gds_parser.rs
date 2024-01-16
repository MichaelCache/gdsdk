use super::gds_error::*;
use super::gds_model::*;
use super::gds_record::*;

pub fn parse_gds(records: &[Record]) -> Result<Box<Lib>, Box<dyn Error>> {
    let mut lib: Box<Lib> = Box::new(Lib::default());
    let rec_len = records.len();
    // let mut next_i: usize;
    let mut i = 0;
    while i < rec_len {
        match records[i] {
            Record::Header { version: _ } => {
                i += 1;
                continue;
            }
            Record::BgnLib(_) => (lib, i) = parse_lib(&records[i..])?,
            Record::EndLib => {
                i += 1;
                continue;
            }
            _ => return Err(Box::new(gds_err("not valid gds lib"))),
        }
        i += 1;
    }

    Ok(lib)
}

fn parse_lib(records: &[Record]) -> Result<(Box<Lib>, usize), Box<dyn Error>> {
    let mut lib = Box::new(Lib::default());
    let rec_len = records.len();
    let mut i = 0;
    while i < rec_len {
        match &records[i] {
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
            }
            Record::BgnStr(_) => {
                let (cell, end_i) = parse_cell(&records[i..])?;
                lib.cells.push(cell);
                i += end_i;
            }
            Record::EndLib => {
                break;
            }
            other => {
                println!("get record from lib {:#?}", other);
            }
        }
        i += 1;
    }

    Ok((lib, i))
}

fn parse_cell(records: &[Record]) -> Result<(Box<Cell>, usize), Box<dyn Error>> {
    let mut cell = Box::new(Cell::default());
    let rec_len = records.len();
    let mut i = 0;
    while i < rec_len {
        match &records[i] {
            Record::BgnStr(data) => cell.date = data.clone(), // last modification time of a structure and marks the beginning of a structure
            Record::StrName(s) => cell.name = s.to_string(),
            Record::Boundary => {
                let (polygon, end_i) = parse_polygon(&records[i..])?;
                i += end_i;
                cell.polygons.push(polygon);
            }
            Record::Path => {
                let (path, end_i) = parse_path(&records[i..])?;
                i += end_i;
                cell.paths.push(path);
            }
            Record::StrRef => {
                let (sref, end_i) = parse_sref(&records[i..])?;
                i += end_i;
                cell.refs.push(sref);
            }
            Record::EndStr => {
                break;
            }
            Record::Text => {
                let (text, end_i) = parse_text(&records[i..])?;
                i += end_i;
                cell.label.push(text)
            }
            other => {
                println!("get record from cell {:#?}", other);
            }
        }
        i += 1;
    }

    Ok((cell, i))
}

fn parse_text(records: &[Record]) -> Result<(Text, usize), Box<dyn Error>> {
    let mut text = Text::default();
    let rec_len = records.len();
    let mut i = 0;
    while i < rec_len {
        match &records[i] {
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
                absolute_magnification,
                absolute_angle,
            } => text.x_reflection = *reflection_x,
            Record::Points(points) => {
                text.position = (
                    points.first().unwrap().0 as f64,
                    points.first().unwrap().1 as f64,
                )
            }
            Record::EndElem => break,
            other => {
                println!("get record from Text {:#?}", other);
            }
        }
        i += 1;
    }
    Ok((text, i))
}

fn parse_polygon(records: &[Record]) -> Result<(Polygon, usize), Box<dyn Error>> {
    let mut polygon = Polygon::default();
    let rec_len = records.len();
    let mut i = 0;
    while i < rec_len {
        match &records[i] {
            Record::Boundary => (), //marks the beginning of a boundary element
            Record::Layer(l) => polygon.layer = *l,
            Record::DataType(d) => polygon.datatype = *d,
            Record::Points(points) => {
                polygon.points = points.iter().map(|&(x, y)| (x as f64, y as f64)).collect()
            }
            Record::EndElem => break,
            other => {
                println!("get record from polygon {:#?}", other);
            }
        }
        i += 1;
    }
    Ok((polygon, i))
}

fn parse_path(records: &[Record]) -> Result<(Path, usize), Box<dyn Error>> {
    let mut path = Path::default();
    let rec_len = records.len();
    let mut i = 0;
    while i < rec_len {
        match &records[i] {
            Record::Path => (), // marks the beginning of a path element
            Record::Layer(l) => path.layer = *l,
            Record::DataType(d) => path.datatype = *d,
            Record::Width(w) => path.width = *w,
            Record::PathType(t) => path.end_type = *t,
            Record::Points(points) => path.points = points.clone(),
            Record::EndElem => break,
            other => {
                println!("get record from path {:#?}", other);
            }
        }
        i += 1;
    }
    Ok((path, i))
}

fn parse_sref(records: &[Record]) -> Result<(Ref, usize), Box<dyn Error>> {
    let mut sref = Ref::default();
    let rec_len = records.len();
    let mut i = 0;
    while i < rec_len {
        match &records[i] {
            Record::StrRef => (), // marks the beginning of an SREF(structure reference) element
            Record::StrRefName(s) => sref.ref_cell_name = s.to_string(),
            Record::RefTrans {
                reflection_x,
                absolute_magnification,
                absolute_angle,
            } => {
                sref.reflection_x = *reflection_x;
                sref.abs_magnific = *absolute_magnification;
                sref.abs_angel = *absolute_angle;
            }
            Record::Angle(angle) => sref.angle = std::f64::consts::PI / 180.0 * angle,
            Record::Points(points) => sref.origin = *points.first().unwrap(),
            Record::EndElem => break,
            other => {
                println!("get record from ref {:#?}", other);
            }
        }
        i += 1;
    }
    Ok((sref, i))
}
