use super::gds_error::*;
use super::gds_record::*;

fn two_byte_int(byte: &[u8]) -> Result<Vec<i16>, Box<dyn Error>> {
    let byte_len = byte.len();
    if byte_len % 2 != 0 {
        return Err(Box::new(gds_err(
            "transfer two byte int failed: byte length % 2 != 0",
        )));
    }
    let mut value: Vec<i16> = Vec::new();
    for i in (0..byte_len).step_by(2) {
        value.push(i16::from_be_bytes(byte[i..i + 2].try_into()?));
    }
    Ok(value)
}

fn four_byte_int(byte: &[u8]) -> Result<Vec<i32>, Box<dyn Error>> {
    let byte_len = byte.len();
    if byte_len % 4 != 0 {
        return Err(Box::new(gds_err(
            "transfer four byte int failed: byte length % 4 != 0",
        )));
    }
    let mut value: Vec<i32> = Vec::new();
    for i in (0..byte_len).step_by(4) {
        value.push(i32::from_be_bytes(byte[i..i + 4].try_into()?));
    }
    Ok(value)
}

/// convert gdsii eight byte real to IEEE 754 f64
///
/// in gdsii stream file, eight byte real is defined as
/// SEEEEEEE MMMMMMMM MMMMMMMM MMMMMMMM
/// MMMMMMMM MMMMMMMM MMMMMMMM MMMMMMMM
/// and value = (-1)^S*16^(E as u32 - 64)*(M as u64 /2^56)
/// for short value = (-1)^S*2^(4*E as u32 - 312)*(M as u64)
///
/// by the way double of IEEE 754 is defined as
/// SEEEEEEE EEEEMMMM MMMMMMMM MMMMMMMM
/// MMMMMMMM MMMMMMMM MMMMMMMM MMMMMMMM
/// and value = (-1)^S*2^(E as u32 -1023)*(1+M as u64/2^52)
pub(crate) fn gdsii_eight_byte_real(byte: &[u8]) -> Result<f64, Box<dyn Error>> {
    if byte.len() != 8 {
        return Err(Box::new(gds_err(
            "transfer eight byte real failed: byte length != 8",
        )));
    }
    // 0x7F is 0b0111_1111, get all E bit, convert to i32
    let exponent = (byte[0] & 0x7F) as i32;
    let mantissa = u64::from_be_bytes(byte.try_into()?) & 0x00FFFFFFFFFFFFFF;
    let result = mantissa as f64 * 2_f64.powi(4_i32 * exponent - 312_i32);
    let sign = (byte[0] & 0x80) != 0;

    if sign {
        Ok(-result)
    } else {
        Ok(result)
    }
}

fn eight_byte_real(byte: &[u8]) -> Result<Vec<f64>, Box<dyn Error>> {
    let byte_len = byte.len();
    if byte_len % 8 != 0 {
        return Err(Box::new(gds_err(
            "transfer eight byte real failed: byte length % 8 != 0",
        )));
    }
    let mut value: Vec<f64> = Vec::new();
    for i in (0..byte_len).step_by(8) {
        value.push(gdsii_eight_byte_real(byte[i..i + 8].try_into()?)?);
    }
    Ok(value)
}

fn ascii_string(byte: &[u8]) -> Result<String, Box<dyn Error>> {
    let strip_none = if *(byte.last().unwrap()) == 0 {
        &byte[..byte.len() - 1]
    } else {
        byte
    };
    let s = String::from_utf8(strip_none.to_vec())?;
    if s.is_ascii() {
        Ok(s)
    } else {
        Err(Box::new(gds_err(&format!(
            "{} contains char not in ascii charset",
            s
        ))))
    }
}

pub fn record_type(bytes: &[u8]) -> Result<Record, Box<dyn Error>> {
    if bytes.len() < 4 {
        return Err(Box::new(gds_err("gds record length less than 4 bytes")));
    }
    let record = &bytes[2..4];
    let data = &bytes[4..];
    match record {
        HEADER => {
            let version = two_byte_int(data)?;
            Ok(Record::Header {
                version: version[0],
            })
        }
        BGNLIB => {
            let date = two_byte_int(data)?;
            Ok(Record::BgnLib(Date::from_i16_array(&date)?))
        }
        // TODO:
        // manual require libname follow UNIX filename conventions for length and valid characters. 1023
        // which is 1023 characters including alphanumeric characters (A-Z, a-z, and 0-9), blanks,
        // mathematical symbols (+ - = | ~ ( ) < > { } \), punctuation marks (? , . ! ; : ' " / [ ]),
        // and the following special characters: &, %, $, #, @, ^, *, and _
        LIBNAME => Ok(Record::LibName(ascii_string(data)?)),
        UNITS => {
            let unit = eight_byte_real(data)?;
            Ok(Record::Units {
                unit_in_meter: unit[0],
                precision: unit[1],
            })
        }
        ENDLIB => Ok(Record::EndLib),
        BGNSTR => {
            let date = two_byte_int(data)?;
            Ok(Record::BgnStr(Date::from_i16_array(&date)?))
        }
        // TODO:
        // manual require strname can be up to 32 characters
        // including alphanumeric characters (A-Z, a-z, and 0-9)
        // Underscore (_), Question mark (?) and Dollar sign ($)
        STRNAME => Ok(Record::StrName(ascii_string(data)?)),
        ENDSTR => Ok(Record::EndStr),
        BOUNDARY => Ok(Record::Boundary),
        PATH => Ok(Record::Path),
        SREF => Ok(Record::StrRef),
        AREF => Ok(Record::AryRef),
        TEXT => Ok(Record::Text),
        LAYER => {
            let layer = two_byte_int(data)?[0];
            // TODO:
            // manual require layer in range [0..255]
            // assert!(layer >= 0 && layer <= 255);
            Ok(Record::Layer(layer))
        }
        DATATYPE => {
            let datatype = two_byte_int(data)?[0];
            // TODO:
            // manual require datatype in range [0..255]
            // assert!(datatype >= 0 && datatype <= 255);
            Ok(Record::DataType(datatype))
        }
        WIDTH => {
            let width = four_byte_int(data)?[0];
            Ok(Record::Width(width))
        }
        XY => {
            // let data = ;
            let xy: Vec<(i32, i32)> = four_byte_int(data)?
                .chunks(2)
                .map(|p| (p[0], p[1]))
                .collect();
            Ok(Record::Points(xy))
        }
        ENDEL => Ok(Record::EndElem),
        // TODO:
        // follow STRNAME rule
        SNAME => Ok(Record::StrRefName(ascii_string(data)?)),
        COLROW => {
            let nums = two_byte_int(data)?;
            Ok(Record::ColRow {
                column: nums[0],
                row: nums[1],
            })
        }
        // TEXTNODE => Record::TEXTNODE,
        // NODE => Record::NODE,
        TEXTTYPE => Ok(Record::TextType(two_byte_int(data)?[0])),
        PRESENTATION => {
            let font_tag = data[1] & 0b0011_0000;
            let ver_tag = data[1] & 0b0000_1100;
            let hor_tag = data[1] & 0b0000_0011;
            Ok(Record::Presentation {
                font_num: if font_tag == 0b0000_0000 {
                    PresentationFont::Fonts0
                } else if font_tag == 0b0001_0000 {
                    PresentationFont::Fonts1
                } else if font_tag == 0b0010_0000 {
                    PresentationFont::Fonts2
                } else if font_tag == 0b0011_0000 {
                    PresentationFont::Fonts3
                } else {
                    return Err(Box::new(gds_err("Unknown font type")));
                },
                vertival_justfication: if ver_tag == 0b0000_0000 {
                    PresentationVerticalPos::Top
                } else if ver_tag == 0b0000_0100 {
                    PresentationVerticalPos::Middle
                } else if ver_tag == 0b0000_1000 {
                    PresentationVerticalPos::Bottom
                } else {
                    return Err(Box::new(gds_err("Unknown vertical type")));
                },
                horizontal_justfication: if hor_tag == 0b0000_0000 {
                    PresentationHorizontalPos::Left
                } else if hor_tag == 0b0000_0001 {
                    PresentationHorizontalPos::Center
                } else if hor_tag == 0b0000_0010 {
                    PresentationHorizontalPos::Right
                } else {
                    return Err(Box::new(gds_err("Unknown horizontal type")));
                },
            })
        }
        // SPACING => Record::SPACING,
        STRING => {
            let s = ascii_string(data)?;
            if s.len() > 512 {
                return Err(Box::new(gds_err("Lib string exceed 512 chars")));
            }
            Ok(Record::String(s))
        }
        STRANS => Ok(Record::RefTrans {
            // test bit 0
            reflection_x: if data[0] & 0x80 != 0 { true } else { false },
            // test bit 13
            absolute_magnification: if data[1] & 0x04 != 0 { true } else { false },
            // test bit 14
            absolute_angle: if data[1] & 0x02 != 0 { true } else { false },
        }),
        MAG => Ok(Record::Mag(eight_byte_real(data)?[0])),
        ANGLE => Ok(Record::Angle(eight_byte_real(data)?[0])),
        // UINTEGER => Record::UINTEGER,
        // USTRING => Record::USTRING,
        // REFLIBS => Record::REFLIBS,
        // FONTS => Record::FONTS,
        PATHTYPE => Ok(Record::PathType(two_byte_int(data)?[0])),
        // GENERATIONS => Record::GENERATIONS,
        // ATTRTABLE => Record::ATTRTABLE,
        // STYPTABLE => Record::STYPTABLE,
        // STRTYPE => Record::STRTYPE,
        // ELFLAGS => Record::ELFLAGS,
        // ELKEY => Record::ELKEY,
        // LINKTYPE => Record::LINKTYPE,
        // LINKKEYS => Record::LINKKEYS,
        // NODETYPE => Record::NODETYPE,
        PROPATTR => {
            let v = two_byte_int(data)?[0];
            // TODO:
            // manual require number is an integer from 1 to 127. Attribute numbers 126 and 127 are reserved
            // assert!(v>=1 && v<= 127);
            Ok(Record::PropAttr(v))
        }
        PROPVALUE => {
            let s = ascii_string(data)?;
            if s.len() > 126 {
                return Err(Box::new(gds_err("Property value record exceed 126 chars")));
            }
            Ok(Record::PropValue(s))
        }
        BOX => Ok(Record::Box),
        BOXTYPE => {
            let boxtype = two_byte_int(data)?[0];
            Ok(Record::BoxType(boxtype))
        }
        // PLEX => Record::PLEX,
        // BGNEXTN => Record::BGNEXTN,
        // ENDEXTN => Record::ENDEXTN,
        // TAPENUM => Record::TAPENUM,
        // TAPECODE => Record::TAPECODE,
        // STRCLASS => Record::STRCLASS,
        // RESERVED => Record::RESERVED,
        // FORMAT => Record::FORMAT,
        // MASK => Record::MASK,
        // ENDMASKS => Record::ENDMASKS,
        // LIBDIRSIZE => Record::LIBDIRSIZE,
        // SRFNAME => Record::SRFNAME,
        // LIBSECUR => Record::LIBSECUR,
        // BORDER => Record::BORDER,
        // SOFTFENCE => Record::SOFTFENCE,
        // HARDFENCE => Record::HARDFENCE,
        // SOFTWIRE => Record::SOFTWIRE,
        // HARDWIRE => Record::HARDWIRE,
        // PATHPORT => Record::PATHPORT,
        // NODEPORT => Record::NODEPORT,
        // USERCONSTRAINT => Record::USERCONSTRAINT,
        // SPACERERROR => Record::SPACERERROR,
        // CONTACT => Record::CONTACT,
        _ => Err(Box::new(gds_err(&format!(
            "Error: unkonw record {:#02x?}",
            bytes
        )))),
    }
}

#[cfg(test)]
mod test_gds_reader {
    use super::*;
    use float_cmp::{ApproxEq, F64Margin};
    #[test]
    fn test_gdsii_eight_byte_real() {
        let mut byte = vec![
            0b01000001_u8,
            0b00010000_u8,
            0b00000000_u8,
            0b00000000_u8,
            0b00000000_u8,
            0b00000000_u8,
            0b00000000_u8,
            0b00000000_u8,
        ];

        assert!(1.0.approx_eq(gdsii_eight_byte_real(&byte).unwrap(), F64Margin::default()));

        byte[0] = 0b01000010_u8;
        assert!(16.0.approx_eq(gdsii_eight_byte_real(&byte).unwrap(), F64Margin::default()));

        byte[0] = 0b01000001_u8;
        byte[1] = 0b00001000_u8;
        assert!(0.5.approx_eq(gdsii_eight_byte_real(&byte).unwrap(), F64Margin::default()));
    }
}
