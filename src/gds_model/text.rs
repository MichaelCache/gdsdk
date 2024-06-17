use super::*;
use crate::gds_record;
use crate::gds_writer;

#[derive(Debug)]
pub enum TextAnchor {
    NW, // NorthWest
    N,
    NE, // NorthEast
    W,
    O, // Origin
    E,
    SW, // SouthWest
    S,
    SE,
}

impl Default for TextAnchor {
    fn default() -> TextAnchor {
        TextAnchor::O
    }
}

#[derive(Debug)]
pub enum TextFont {
    Fonts0,
    Fonts1,
    Fonts2,
    Fonts3,
}

impl Default for TextFont {
    fn default() -> Self {
        TextFont::Fonts0
    }
}

#[derive(Default, Debug)]
pub struct Text {
    pub layer: i16,
    pub datatype: i16,
    pub font: TextFont,
    pub text: String,
    pub position: Points,
    pub anchor: TextAnchor,
    pub rotation: f64, // in radians
    pub magnification: f64,
    pub x_reflection: bool,
    pub repetition: Repetition,
    pub property: Property,
}

impl GdsObject for Text {
    fn to_gds(&self, scaling: f64) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut data = Vec::<u8>::new();

        data.extend(4_i16.to_be_bytes());
        data.extend(gds_record::TEXT);
        data.extend(6_i16.to_be_bytes());
        data.extend(gds_record::LAYER);
        data.extend((self.layer as u16).to_be_bytes());
        data.extend(6_u16.to_be_bytes());
        data.extend(gds_record::TEXTTYPE);
        data.extend((self.datatype as u16).to_be_bytes());
        data.extend(6_u16.to_be_bytes());
        data.extend(gds_record::PRESENTATION);
        data.extend((gds_writer::text_anchor_to_gds_num(&self.anchor) as u16).to_be_bytes());

        let is_transform = self.rotation != 0.0 || self.magnification != 1.0 || self.x_reflection;

        if is_transform {
            data.extend(6_u16.to_be_bytes());
            data.extend(gds_record::STRANS);
            if self.x_reflection {
                data.extend((0x8000 as u16).to_be_bytes());
            } else {
                data.extend((0 as u16).to_be_bytes());
            }
            data.extend(12_u16.to_be_bytes());
            data.extend(gds_record::MAG);
            data.extend(gds_writer::f64_to_gds_bytes(self.magnification));
            data.extend(12_u16.to_be_bytes());
            data.extend(gds_record::ANGLE);
            data.extend(gds_writer::f64_to_gds_bytes(self.rotation));
        }
        // XY
        data.extend(12_u16.to_be_bytes());
        data.extend(gds_record::XY);
        data.extend((f64::round(self.position.x * scaling) as i32).to_be_bytes());
        data.extend((f64::round(self.position.y * scaling) as i32).to_be_bytes());

        // STRING
        let mut text_data = gds_writer::ascii_string_to_be_bytes(&self.text);
        if !text_data.len().is_power_of_two() {
            text_data.push(0);
        }

        data.extend((text_data.len() as u16 + 4_u16).to_be_bytes());
        data.extend(gds_record::STRING);
        data.extend(text_data);

        // properties
        data.extend(self.property.to_gds(scaling)?);

        data.extend(4_u16.to_be_bytes());
        data.extend(gds_record::ENDEL);

        Ok(data)
    }
}

#[derive(Default, Debug)]
pub struct Repetition {
    pub count_1: u64,
    pub count_2: u64,
    pub vec_1: Vector,
    pub vec_2: Vector,
}
