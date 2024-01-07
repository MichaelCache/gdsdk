
use super::gds_error::*;
use super::gds_record::Record;
use super::gds_model::*;

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
            // Record::Text => {

            // }
            other => {
                println!("get record from cell {:#?}", other);
            }
        }
        i += 1;
    }

    Ok((cell, i))
}

fn parse_polygon(records: &[Record]) -> Result<(Box<Polygon>, usize), Box<dyn Error>> {
    let mut polygon = Box::new(Polygon::default());
    let rec_len = records.len();
    let mut i = 0;
    while i < rec_len {
        match &records[i] {
            Record::Boundary => (), //marks the beginning of a boundary element
            Record::Layer(l) => polygon.layer = *l,
            Record::DataType(d) => polygon.datatype = *d,
            Record::Points(points) => polygon.points = points.clone(),
            Record::EndElem => break,
            other => {
                println!("get record from polygon {:#?}", other);
            }
        }
        i += 1;
    }
    Ok((polygon, i))
}

fn parse_path(records: &[Record]) -> Result<(Box<Path>, usize), Box<dyn Error>> {
    let mut path = Box::new(Path::default());
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

fn parse_sref(records: &[Record]) -> Result<(Box<Ref>, usize), Box<dyn Error>> {
    let mut sref = Box::new(Ref::default());
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
            Record::Angle(a) => sref.angle = *a.first().unwrap(),
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
