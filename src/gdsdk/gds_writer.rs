use std::vec::Vec;

pub fn ascii_string_to_be_bytes(s: &str) -> Vec<u8> {
    let mut be_bytes = Vec::<u8>::new();
    if !s.is_ascii() {
        panic!("{} is not ascii", s);
    }
    for c in s.chars() {
        be_bytes.push(c as u8);
    }

    be_bytes
}

pub fn f64_to_gds_bytes(v: f64) -> Vec<u8> {
    let mut be_bytes = Vec::<u8>::new();
    be_bytes.resize(1, 0);

    // sign
    be_bytes[0] |= (v.is_sign_negative() as u8).to_be_bytes()[0];
    // exponent
    let fexp = 0.25 * v.log2();
    let mut exponent = fexp.ceil();
    if exponent == fexp {
        exponent = exponent + 1_f64;
    }

    // mantissa
    let mantissa = v * 16_f64.powf(14_f64 - exponent);
    let mantissa_byte = (mantissa as u64).to_be_bytes();

    // assemble binary
    be_bytes[0] |= ((exponent + 64_f64) as u8).to_be_bytes()[0];
    be_bytes.extend(&mantissa_byte[1..]);
    

    be_bytes
}

#[cfg(test)]
mod test_gds_writer {
    use crate::gdsdk::gds_reader;

    use super::*;
    use float_cmp::{ApproxEq, F64Margin};
    #[test]
    fn test_f64_to_gds_bytes() {
        let v = 1.0e-9;

        let gds_be_bytes = f64_to_gds_bytes(v);
        let fv = gds_reader::gdsii_eight_byte_real(&gds_be_bytes).unwrap();
        assert!(v.approx_eq(fv, F64Margin::default()));
    }
}
