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

use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fs::read;
use std::path;
use std::sync::{mpsc, Arc, RwLock};

use singleton_threadpool::get_thread_pool;

fn to_gds_record(buffs: Vec<Vec<u8>>) -> Result<Vec<gds_record::Record>, Box<dyn Error>> {
    let byte_len = buffs.len();
    let mut records = Vec::with_capacity(byte_len);
    for buff in buffs {
        match gds_reader::record_type(&buff) {
            Ok(r) => records.push(r),
            Err(err) => {
                return Err(Box::new(gds_err!(&format!(
                    "parse error at byte offset range {}",
                    err
                ))))
            }
        }
        // ENDLIB marks the end of a stream format file
        if let Some(gds_record::Record::EndLib) = records.last() {
            break;
        }
    }

    Ok(records)
}

/// read gds file return gds lib
pub fn read_gdsii<T: AsRef<path::Path>>(
    gds_file: T,
) -> Result<Box<gds_model::Lib>, Box<dyn Error>> {
    let mut buff = read(gds_file)?;
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
    let mut records = Vec::<Vec<u8>>::new();

    while buff.len() > 0 {
        // each gds record first 2 byte stored record byte length
        let record_len = u16::from_be_bytes(match buff[0..0 + 2].try_into() {
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

        records.push(buff.drain(0..0 + record_len).collect());
    }

    let thread_num = get_thread_pool().read().unwrap().max_count() + 1;
    let thread_buff_len = records.len() / thread_num + thread_num;

    let mut thread_buffers: VecDeque<Vec<Vec<u8>>> = records
        .chunks(thread_buff_len)
        .map(|v| v.to_vec())
        .collect();

    let (tx, rx) = mpsc::channel();
    for i in 0..thread_num - 1 {
        let cur_buff = thread_buffers.pop_front().unwrap();
        let tx = tx.clone();
        get_thread_pool().read().unwrap().execute(move || {
            let recs = to_gds_record(cur_buff).unwrap();
            tx.send((i, recs)).unwrap();
        })
    }

    drop(tx);

    let main_thread_buff = to_gds_record(thread_buffers.pop_front().unwrap()).unwrap();

    let mut thread_rescs = HashMap::new();
    while let Ok((i, recs)) = rx.recv() {
        thread_rescs.insert(i, recs);
    }

    let gds_records: Arc<RwLock<Vec<gds_record::Record>>> = Arc::new(RwLock::new(Vec::new()));

    for i in 0..thread_num - 1 {
        gds_records
            .write()
            .unwrap()
            .extend(thread_rescs.remove(&i).unwrap());
    }

    gds_records.write().unwrap().extend(main_thread_buff);

    if gds_records.read().unwrap().len() == 0 {
        return Result::Err(Box::new(gds_err!(
            "not valid gds file, no any valid records found"
        )));
    }

    // transfer gds record data to gds object
    Ok(gds_parser::parse_gds(gds_records)?)
}
