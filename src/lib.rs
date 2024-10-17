//! Gdsii stream format develop kit
//!
//! Read and parse gds file
//!
//! Or create gds object and export to file

#[macro_use]
pub mod gds_error;
pub mod gds_model;
mod gds_parser;
mod gds_reader;
mod gds_record;
mod gds_writer;

use rayon::{prelude::*, ThreadPoolBuilder};
use std::error::Error;
use std::fs::read;
use std::path;

fn to_gds_record(
    buff: &[u8],
    (start, end): &(usize, usize),
) -> Result<gds_record::Record, Box<dyn Error>> {
    Ok(gds_reader::record_type(&buff[*start..*end]).unwrap())
}

/// read gds file return gds lib
pub fn read_gdsii<T: AsRef<path::Path>>(
    gds_file: T,
) -> Result<Box<gds_model::Lib>, Box<dyn Error + Sync + Send>> {
    let _ = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get_physical())
        .build_global();
    let buff = read(gds_file)?;
    let byte_len = buff.len();
    if byte_len < 4usize {
        return Result::Err(Box::new(gds_err!(
            "not valid gds file, file size less than 4 byte"
        )));
    }

    // check valid gds file
    if let gds_record::HEADER = &buff[2..4] {
        // do nothing
    } else {
        return Result::Err(Box::new(gds_err!(
            "not valid gds file, no valid gds header section found"
        )));
    }

    let mut idx: usize = 0;
    let mut record_ranges = Vec::new();
    while idx < buff.len() {
        // each gds record first 2 byte stored record byte length
        let record_len = u16::from_be_bytes(match buff[idx..idx + 2].try_into() {
            Ok(v) => v,
            Err(err) => {
                return Result::Err(Box::new(gds_err!(&format!(
                    "transfer gds record failed {}",
                    err
                ))))
            }
        }) as usize;

        if record_len == 0 {
            return Result::Err(Box::new(gds_err!(
                "not valid gds record length, zero length"
            )));
        }
        record_ranges.push((idx, idx + record_len));
        idx += record_len;
    }

    let gds_records: Vec<gds_record::Record> = record_ranges
        .par_iter()
        .map(|range| to_gds_record(&buff, range).unwrap())
        .collect();

    if gds_records.len() == 0 {
        return Result::Err(Box::new(gds_err!(
            "not valid gds file, no any valid records found"
        )));
    }

    // transfer gds record data to gds object
    Ok(gds_parser::parse_gds(gds_records)?)
    // Ok(Box::new(gds_model::Lib::new("")))
}
