//! gds error type

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct GDSIIError {
    err: String,
}

impl GDSIIError {
    pub fn new(err: &str) -> GDSIIError {
        GDSIIError {
            err: err.to_string(),
        }
    }
}

impl Display for GDSIIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[GDSIIError]: {}", self.err)
    }
}

impl Error for GDSIIError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

#[macro_export]
macro_rules! gds_err {
    ( $x:expr ) => {{
        crate::gds_error::GDSIIError::new(
            format!("{}:{} : {}", file!(), line!(), $x).as_str(),
        )
    }};
}
