//! Gdsii stream format develop kit
//!
//! Read and parse gds file
//!
//! Or create gds object and export to file

pub mod gds_error;
pub mod gds_model;
mod gds_parser;
mod gds_reader;
mod gds_record;
mod gds_writer;
mod singleton_threadpool;

use std::error::Error;
use std::fs::read;
use std::path;
use std::sync::{Arc, RwLock};

/// read gds file return gds lib
pub fn read_gdsii<T: AsRef<path::Path>>(
    gds_file: T,
) -> Result<Box<gds_model::Lib>, Box<dyn Error>> {
    let buff = read(gds_file)?;
    let byte_len = buff.len();
    if byte_len < 4usize {
        return Result::Err(Box::new(gds_error::gds_err(
            "not valid gds file, file size less than 4 byte",
        )));
    }

    // check valid gds file
    if let gds_record::HEADER = &buff[2..4] {
        // do nothing
    } else {
        return Result::Err(Box::new(gds_error::gds_err(
            "not valid gds file, no valid gds header section found",
        )));
    }

    // slice file content to gds records
    let mut idx: usize = 0;
    let mut record_len: usize;
    let records: Arc<RwLock<Vec<gds_record::Record>>> = Arc::new(RwLock::new(Vec::new()));
    while idx < byte_len {
        // each gds record first 2 byte stored record byte length
        record_len = u16::from_be_bytes(match buff[idx..idx + 2].try_into() {
            Ok(v) => v,
            Err(err) => {
                return Result::Err(Box::new(gds_error::gds_err(&format!(
                    "transfer gds record failed at {:#08x}:{:#08x}: {}",
                    idx,
                    idx + 2,
                    err
                ))))
            }
        }) as usize;

        if record_len == 0 {
            return Result::Err(Box::new(gds_error::gds_err(&format!(
                "not valid gds record length at {:#08x}:{:#08x}: zero length",
                idx,
                idx + 2
            ))));
        }

        match gds_reader::record_type(&buff[idx..idx + record_len]) {
            Ok(r) => records.write().unwrap().push(r),
            Err(err) => {
                return Err(Box::new(gds_error::gds_err(&format!(
                    "parse error at byte offset range {:#08x}:{:#08x}: {}",
                    idx,
                    idx + record_len,
                    err
                ))))
            }
        }

        // ENDLIB marks the end of a stream format file
        if let Some(gds_record::Record::EndLib) = records.read().unwrap().last() {
            break;
        }

        idx += record_len;
    }

    if records.read().unwrap().len() == 0 {
        return Result::Err(Box::new(gds_error::gds_err(
            "not valid gds file, no any valid records found",
        )));
    }

    // transfer gds record data to gds object
    Ok(gds_parser::parse_gds(records)?)
}
