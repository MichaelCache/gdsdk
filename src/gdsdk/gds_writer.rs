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
// FIXME:not correct
pub fn f64_to_gds_bytes(v: f64) -> Vec<u8> {
    let mut be_bytes = Vec::<u8>::new();
    be_bytes.resize(8, 0);
    let v_bytes = v.to_be_bytes();
    // sign
    be_bytes[0] |= v_bytes[0] & 0x80;
    // exponent
    let mut exp =  (v_bytes[0] & 0x0f << 4 | (v_bytes[1] & 0xf0)) as u32;
    exp = exp + 1023 - 64;
    
    // mantissa
    be_bytes[1] |= v_bytes[1] & 0x0f << 4 | (v_bytes[2] & 0xf0);
    be_bytes[2] |= v_bytes[2] & 0x0f << 4 | (v_bytes[3] & 0xf0);
    be_bytes[3] |= v_bytes[3] & 0x0f << 4 | (v_bytes[4] & 0xf0);
    be_bytes[4] |= v_bytes[4] & 0x0f << 4 | (v_bytes[5] & 0xf0);
    be_bytes[5] |= v_bytes[5] & 0x0f << 4 | (v_bytes[6] & 0xf0);
    be_bytes[6] |= v_bytes[6] & 0x0f << 4 | (v_bytes[7] & 0xf0);
    be_bytes[7] |= v_bytes[7] & 0x0f << 4;

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
