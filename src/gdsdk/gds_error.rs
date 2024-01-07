pub use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct GDSIIError {
    err: String,
}

pub fn gds_err(err: &str) -> GDSIIError {
    GDSIIError {
        err: err.to_string(),
    }
}

impl Display for GDSIIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GDSIIError: {}", self.err)
    }
}

impl Error for GDSIIError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}
