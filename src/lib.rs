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
mod singleton_threadpool;

use std::collections::VecDeque;
use std::error::Error;
use std::fs::read;
use std::path;
use std::sync::{Arc, RwLock};

use singleton_threadpool::get_thread_pool;

fn to_gds_record(
    buff: &[u8],
    range: (usize, usize),
) -> Result<Vec<gds_record::Record>, Box<dyn Error>> {
    let byte_len = buff.len();
    let mut records = Vec::with_capacity(byte_len);
    let (start, end) = range;

    let mut idx = start; //    let mut idx = 0;
    while idx < end && idx < byte_len {
        let record_len = u16::from_be_bytes(match buff[idx..idx + 2].try_into() {
            Ok(v) => v,
            Err(err) => {
                return Result::Err(Box::new(gds_err!(&format!(
                    "transfer gds record failed {}",
                    err
                ))))
            }
        }) as usize;
        
        records.push(gds_reader::record_type(&buff[idx..idx + record_len]).unwrap());
        idx += record_len;
    }

    Ok(records)
}

/// read gds file return gds lib
pub fn read_gdsii<T: AsRef<path::Path>>(
    gds_file: T,
) -> Result<Box<gds_model::Lib>, Box<dyn Error>> {
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

    // slice file content to gds records
    let thread_num = get_thread_pool().read().unwrap().max_count() + 1;

    let thread_buff_len = if byte_len % thread_num == 0 {
        byte_len / thread_num
    } else {
        byte_len / thread_num + thread_num
    };

    let mut thread_buff_range = VecDeque::new();

    let mut idx: usize = 0;
    let mut thread_start_idx: usize = 0;
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
            // return Result::Err(Box::new(gds_err!(
            // "not valid gds record length, zero length"
            // )));
            thread_buff_range.push_back((thread_start_idx, idx));
            break;
        }
        idx += record_len;
        if idx - thread_start_idx >= thread_buff_len {
            thread_buff_range.push_back((thread_start_idx, idx));
            thread_start_idx = idx;
        }
    }

    if thread_buff_range.back().unwrap().1 != buff.len() {
        thread_buff_range.push_back((thread_buff_range.back().unwrap().1, byte_len));
    }

    let gds_records: Arc<RwLock<Vec<gds_record::Record>>> = Arc::new(RwLock::new(Vec::new()));
    let thread_buff = Arc::new(RwLock::new(buff));

    let cur_thread = Arc::new(RwLock::new(-1));
    for i in 0..thread_buff_range.len() - 1 {
        let range = thread_buff_range.pop_front().unwrap();
        let save_recodes = gds_records.clone();
        let buff = thread_buff.clone();
        let thread_id = cur_thread.clone();
        get_thread_pool().read().unwrap().execute(move || {
            let thread_records = to_gds_record(&buff.read().unwrap(), range).unwrap();
            loop {
                if *thread_id.read().unwrap() + 1 == i as i32 {
                    save_recodes.write().unwrap().extend(thread_records);
                    let mut cur_thread_id = thread_id.write().unwrap();
                    *cur_thread_id = i as i32;
                    break;
                }
            }
        })
    }

    let last_records = to_gds_record(
        &thread_buff.read().unwrap(),
        thread_buff_range.pop_front().unwrap(),
    )
    .unwrap();

    get_thread_pool().read().unwrap().join();

    gds_records.write().unwrap().extend(last_records);

    if gds_records.read().unwrap().len() == 0 {
        return Result::Err(Box::new(gds_err!(
            "not valid gds file, no any valid records found"
        )));
    }

    // transfer gds record data to gds object
    Ok(gds_parser::parse_gds(gds_records)?)
}
