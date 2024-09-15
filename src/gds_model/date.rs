use chrono::{DateTime, Datelike, Timelike, Utc};
use std::{error::Error, time::SystemTime};

use super::*;

#[derive(Debug, Default, Clone)]
pub struct Date {
    pub mod_year: i16,
    pub mod_month: i16,
    pub mod_day: i16,
    pub mod_hour: i16,
    pub mod_minute: i16,
    pub mod_second: i16,
    pub acc_year: i16,
    pub acc_month: i16,
    pub acc_day: i16,
    pub acc_hour: i16,
    pub acc_minute: i16,
    pub acc_second: i16,
}

impl Date {
    pub fn new() -> Self {
        Date {
            mod_year: i16::default(),
            mod_month: i16::default(),
            mod_day: i16::default(),
            mod_hour: i16::default(),
            mod_minute: i16::default(),
            mod_second: i16::default(),
            acc_year: i16::default(),
            acc_month: i16::default(),
            acc_day: i16::default(),
            acc_hour: i16::default(),
            acc_minute: i16::default(),
            acc_second: i16::default(),
        }
    }

    pub fn now() -> Self {
        let now = SystemTime::now();
        let utc: DateTime<Utc> = now.into();

        // year, month, day
        let year = utc.year() as i16;
        let month = utc.month() as i16;
        let day = utc.day() as i16;

        // hour, minute, second
        let hour = utc.hour() as i16;
        let minute = utc.minute() as i16;
        let second = utc.second() as i16;

        Date {
            mod_year: year,
            mod_month: month,
            mod_day: day,
            mod_hour: hour,
            mod_minute: minute,
            mod_second: second,
            acc_year: year,
            acc_month: month,
            acc_day: day,
            acc_hour: hour,
            acc_minute: minute,
            acc_second: second,
        }
    }

    pub fn from_i16_array(date: &[i16]) -> Result<Date, Box<dyn Error>> {
        if date.len() < 12 {
            return Err(Box::new(gds_err!(
                "Can't create gds Date for data length less than 12"
            )));
        }
        let mut it = date.iter();
        Ok(Date {
            mod_year: *it.next().unwrap(),
            mod_month: *it.next().unwrap(),
            mod_day: *it.next().unwrap(),
            mod_hour: *it.next().unwrap(),
            mod_minute: *it.next().unwrap(),
            mod_second: *it.next().unwrap(),
            acc_year: *it.next().unwrap(),
            acc_month: *it.next().unwrap(),
            acc_day: *it.next().unwrap(),
            acc_hour: *it.next().unwrap(),
            acc_minute: *it.next().unwrap(),
            acc_second: *it.next().unwrap(),
        })
    }
}

impl GdsObject for Date {
    fn to_gds(&self, _: f64) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut date_data = Vec::<u8>::new();
        date_data.extend(self.mod_year.to_be_bytes());
        date_data.extend(self.mod_month.to_be_bytes());
        date_data.extend(self.mod_day.to_be_bytes());
        date_data.extend(self.mod_hour.to_be_bytes());
        date_data.extend(self.mod_minute.to_be_bytes());
        date_data.extend(self.mod_second.to_be_bytes());
        date_data.extend(self.acc_year.to_be_bytes());
        date_data.extend(self.acc_month.to_be_bytes());
        date_data.extend(self.acc_day.to_be_bytes());
        date_data.extend(self.acc_hour.to_be_bytes());
        date_data.extend(self.acc_minute.to_be_bytes());
        date_data.extend(self.acc_second.to_be_bytes());
        Ok(date_data)
    }
}
