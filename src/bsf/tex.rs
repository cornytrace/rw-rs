use nom::{
    number::complete::{le_u16, le_u32, le_u8},
    IResult, Parser,
};
use nom_derive::{Nom, Parse};
use num_derive::FromPrimitive;
use num_traits::cast::FromPrimitive;

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

#[derive(Clone, Copy, Debug, Nom)]
#[repr(u8)]
pub enum TextureFilteringMode {
    FILTERNAFILTERMODE,     // filtering is disabled
    FILTERNEAREST,          // Point sampled
    FILTERLINEAR,           // Bilinear
    FILTERMIPNEAREST,       // Point sampled per pixel mip map
    FILTERMIPLINEAR,        // Bilinear per pixel mipmap
    FILTERLINEARMIPNEAREST, // MipMap interp point sampled
    FILTERLINEARMIPLINEAR,  // Trilinear
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u8)]
pub enum TextureAddressingMode {
    TEXTUREADDRESSNATEXTUREADDRESS, // no tiling
    TEXTUREADDRESSWRAP,             // tile in U or V direction
    TEXTUREADDRESSMIRROR,           // mirror in U or V direction
    TEXTUREADDRESSCLAMP,
    TEXTUREADDRESSBORDER,
}

pub struct RpTexture {
    pub filtering: TextureFilteringMode,
    pub addressing: [TextureAddressingMode; 2],
    pub has_mip: bool,
}

impl RpTexture {
    pub fn parse<'a>(&mut self, i: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, filtering) = TextureFilteringMode::parse_le(i)?;
        let (i, addr) = le_u8(i)?;
        let addr_h = TextureAddressingMode::from_u8((addr & 0b11110000) >> 4).unwrap();
        let addr_l = TextureAddressingMode::from_u8(addr & 0b00001111).unwrap();
        let addressing = [addr_h, addr_l];
        let (i, has_mip) = le_u16(i)?;
        let has_mip = has_mip != 0;

        Ok((
            i,
            Self {
                filtering,
                addressing,
                has_mip,
            },
        ))
    }
}
