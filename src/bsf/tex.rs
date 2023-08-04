use nom::{number::complete::le_u32, IResult};
use nom_derive::{Nom, Parse};

#[derive(Clone, Copy, Debug, Nom)]
pub struct RwRGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Copy, Debug, Nom)]
pub struct RwTexCoords {
    pub u: f32,
    pub v: f32,
}

impl RwTexCoords {
    pub fn as_arr(&self) -> [f32; 2] {
        [self.u, self.v]
    }
}

#[derive(Clone, Copy, Debug, Nom)]
pub struct RpSurfProp {
    pub ambient: f32,
    pub specular: f32,
    pub diffuse: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct RpMaterial {
    pub color: RwRGBA,
    pub surface_prop: Option<RpSurfProp>,
}
impl RpMaterial {
    pub fn parse(i: &[u8], version: u32) -> IResult<&[u8], Self> {
        let (i, _flags) = le_u32(i)?;
        let (i, color) = RwRGBA::parse_le(i)?;
        let (i, _unused) = le_u32(i)?;
        let (mut i, _is_textured) = le_u32(i)?;

        let mut surface_prop = None;
        if version > 0x30400 {
            let s;
            (i, s) = RpSurfProp::parse_le(i)?;
            surface_prop = Some(s);
        }

        Ok((
            i,
            Self {
                color,
                surface_prop,
            },
        ))
    }
}
