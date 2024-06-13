#![allow(unused)]
// gdsii record format
//
//  Bit 0                                   16
//      |  Total record length(in bytes)    |
//      |  Record Type     |    Data Type   |
//      |       Data content....            |
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::{error::Error, time::SystemTime};

use super::gds_error;
use super::gds_model;

// header
pub const HEADER: &[u8] = &[0x00, 0x02];
pub const BGNLIB: &[u8] = &[0x01, 0x02];
pub const LIBNAME: &[u8] = &[0x02, 0x06];
pub const UNITS: &[u8] = &[0x03, 0x05];
pub const ENDLIB: &[u8] = &[0x04, 0x00];
pub const BGNSTR: &[u8] = &[0x05, 0x02];
pub const STRNAME: &[u8] = &[0x06, 0x06];
pub const ENDSTR: &[u8] = &[0x07, 0x00];
pub const BOUNDARY: &[u8] = &[0x08, 0x00];
pub const PATH: &[u8] = &[0x09, 0x00];
pub const SREF: &[u8] = &[0x0a, 0x00];
pub const AREF: &[u8] = &[0x0b, 0x00];
pub const TEXT: &[u8] = &[0x0c, 0x00];
pub const LAYER: &[u8] = &[0x0d, 0x02];
pub const DATATYPE: &[u8] = &[0x0e, 0x02];
pub const WIDTH: &[u8] = &[0x0f, 0x03];
pub const XY: &[u8] = &[0x10, 0x03];
pub const ENDEL: &[u8] = &[0x11, 0x00];
pub const SNAME: &[u8] = &[0x12, 0x06];
pub const COLROW: &[u8] = &[0x13, 0x02];
pub const TEXTNODE: &[u8] = &[0x14, 0x00]; // No Data Present (Not currently used)
pub const NODE: &[u8] = &[0x15, 0x00]; // No Data Present
pub const TEXTTYPE: &[u8] = &[0x16, 0x02];
pub const PRESENTATION: &[u8] = &[0x17, 0x01];
pub const SPACING: Option<&[u8]> = None; // Not currently used
pub const STRING: &[u8] = &[0x19, 0x06];
pub const STRANS: &[u8] = &[0x1a, 0x01];
pub const MAG: &[u8] = &[0x1b, 0x05];
pub const ANGLE: &[u8] = &[0x1c, 0x05];
pub const UINTEGER: Option<&[u8]> = None; // Not currently used, User Integer data was used in GDSII Release 2.0 only
pub const USTRING: Option<&[u8]> = None; // Not currently used, User String data, formerly called character string data (CSD), was used in GDSII Releases 1.0 and 2.0
pub const REFLIBS: &[u8] = &[0x1f, 0x06];
pub const FONTS: &[u8] = &[0x20, 0x06];
pub const PATHTYPE: &[u8] = &[0x21, 0x02];
pub const GENERATIONS: &[u8] = &[0x22, 0x02];
pub const ATTRTABLE: &[u8] = &[0x23, 0x06];
pub const STYPTABLE: &[u8] = &[0x24, 0x06]; // Unreleased feature
pub const STRTYPE: &[u8] = &[0x25, 0x02]; // Unreleased feature
pub const ELFLAGS: &[u8] = &[0x26, 0x01];
pub const ELKEY: &[u8] = &[0x27, 0x03]; // Unreleased feature
pub const LINKTYPE: &[u8] = &[0x28]; // Unreleased feature
pub const LINKKEYS: &[u8] = &[0x29]; // Unreleased feature
pub const NODETYPE: &[u8] = &[0x2a, 0x02];
pub const PROPATTR: &[u8] = &[0x2b, 0x02];
pub const PROPVALUE: &[u8] = &[0x2c, 0x06];
pub const BOX: &[u8] = &[0x2d, 0x00];
pub const BOXTYPE: &[u8] = &[0x2e, 0x02];
pub const PLEX: &[u8] = &[0x2f, 0x03];
pub const BGNEXTN: &[u8] = &[0x30, 0x03]; //TODO:
pub const ENDEXTN: &[u8] = &[0x31, 0x03]; // TODO:
pub const TAPENUM: &[u8] = &[0x32, 0x03];
pub const TAPECODE: &[u8] = &[0x33, 0x02];
pub const STRCLASS: &[u8] = &[0x34, 0x01];
pub const RESERVED: &[u8] = &[0x35, 0x03]; // Not currently used, This record type was used for NUMTYPES but was not required.
pub const FORMAT: &[u8] = &[0x36, 0x02];
pub const MASK: &[u8] = &[0x37, 0x06];
pub const ENDMASKS: &[u8] = &[0x38, 0x00];
pub const LIBDIRSIZE: &[u8] = &[0x39, 0x02];
pub const SRFNAME: &[u8] = &[0x3a, 0x06];
pub const LIBSECUR: &[u8] = &[0x3b, 0x02];
pub const BORDER: &[u8] = &[0x3c, 0x00];
pub const SOFTFENCE: &[u8] = &[0x3d, 0x00];
pub const HARDFENCE: &[u8] = &[0x3e, 0x00];
pub const SOFTWIRE: &[u8] = &[0x3f, 0x00];
pub const HARDWIRE: &[u8] = &[0x40, 0x00];
pub const PATHPORT: &[u8] = &[0x41, 0x00];
pub const NODEPORT: &[u8] = &[0x42, 0x00];
pub const USERCONSTRAINT: &[u8] = &[0x43, 0x00];
pub const SPACERERROR: &[u8] = &[0x44, 0x00];
pub const CONTACT: &[u8] = &[0x45, 0x00];

#[derive(Debug)]
pub enum PresentationFont {
    Fonts0,
    Fonts1,
    Fonts2,
    Fonts3,
}

#[derive(Debug)]
pub enum PresentationVerticalPos {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug)]
pub enum PresentationHorizontalPos {
    Left,
    Center,
    Right,
}

#[derive(Debug)]
pub enum Record {
    Header {
        version: i16,
    },
    BgnLib(gds_model::Date),
    LibName(String),
    Units {
        unit_in_meter: f64,
        precision: f64,
    },
    EndLib,
    BgnStr(gds_model::Date),
    StrName(String),
    EndStr,
    Boundary,
    Path,
    StrRef,
    AryRef,
    Text,
    Layer(i16),
    DataType(i16),
    Width(i32),
    Points(Vec<(i32, i32)>),
    EndElem,
    StrRefName(String),
    ColRow {
        column: i16,
        row: i16,
    },
    // TEXTNODE,
    // NODE,
    TextType(i16),
    Presentation {
        font_num: PresentationFont,
        vertival_justfication: PresentationVerticalPos,
        horizontal_justfication: PresentationHorizontalPos,
    },
    // SPACING,
    String(String),
    RefTrans {
        reflection_x: bool,
        absolute_magnification: bool,
        absolute_angle: bool,
    },
    Mag(f64),
    Angle(f64),
    // UINTEGER,
    // USTRING,
    // REFLIBS,
    // FONTS,
    PathType(i16),
    // GENERATIONS,
    // ATTRTABLE,
    // STYPTABLE,
    // STRTYPE,
    // ELFLAGS,
    // ELKEY,
    // LINKTYPE,
    // LINKKEYS,
    // NODETYPE,
    PropAttr(i16),
    PropValue(String),
    Box,
    BoxType(i16),
    // PLEX,
    // BGNEXTN,
    // ENDEXTN,
    // TAPENUM,
    // TAPECODE,
    // STRCLASS,
    // RESERVED,
    // FORMAT,
    // MASK,
    // ENDMASKS,
    // LIBDIRSIZE,
    // SRFNAME,
    // LIBSECUR,
    // BORDER,
    // SOFTFENCE,
    // HARDFENCE,
    // SOFTWIRE,
    // HARDWIRE,
    // PATHPORT,
    // NODEPORT,
    // USERCONSTRAINT,
    // SPACERERROR,
    // CONTACT,
}
