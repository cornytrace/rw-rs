use std::ffi::{c_char, CStr};

use nom::{
    bytes,
    character::is_alphanumeric,
    number::complete::{le_u16, le_u32, le_u8},
    IResult,
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

impl RwRGBA {
    pub fn as_arr(&self) -> [f32; 4] {
        [self.r.into(), self.g.into(), self.b.into(), self.a.into()]
    }
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

#[derive(Clone, Copy, Debug, Nom, FromPrimitive)]
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

#[derive(Clone, Copy, Debug, Nom, FromPrimitive)]
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
    pub fn parse<'a>(i: &'a [u8]) -> IResult<&'a [u8], Self> {
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

#[derive(Debug, Nom, FromPrimitive)]
#[repr(u32)]
pub enum RasterFormat {
    FormatDefault = 0x0000,
    Format1555 = 0x0100, //(1 bit alpha, RGB 5 bits each; also used for DXT1 with alpha)
    Format565 = 0x0200,  //(5 bits red, 6 bits green, 5 bits blue; also used for DXT1 without alpha)
    Format4444 = 0x0300, //(RGBA 4 bits each; also used for DXT3)
    FormatLum8 = 0x0400, //(gray scale, D3DFMT_L8)
    Format8888 = 0x0500, //(RGBA 8 bits each)
    Format888 = 0x0600,  //(RGB 8 bits each, D3DFMT_X8R8G8B8)
    Format555 = 0x0A00,  //(RGB 5 bits each - rare, use 565 instead, D3DFMT_X1R5G5B5)

    FormatExtAutoMipmap = 0x1000, //(RW generates mipmaps, see special section below)
    FormatExtPal8 = 0x2000,       //(2^8 = 256 palette colors)
    FormatExtPal4 = 0x4000,       //(2^4 = 16 palette colors)
    FormatExtMipmap = 0x8000,     //(mipmaps included)
}

#[derive(Debug)]
pub struct RpRasterPC {
    pub platform_id: u32,
    pub filtering: TextureFilteringMode,
    pub addressing: [TextureAddressingMode; 2],
    pub name: String,
    pub mask_name: String,
    pub raster_format: u32,
    pub d3d_format: u32,
    pub width: u16,
    pub height: u16,
    pub depth: u8,
    pub num_levels: u8,
    pub raster_type: u8,
    pub compression: u8,
    pub has_alpha: bool,
    pub cube_texture: bool,
    pub auto_mipmaps: bool,
    pub compressed: bool,
    pub data: Vec<u8>,
}

impl RpRasterPC {
    pub fn parse(i: &[u8], version: u32) -> IResult<&[u8], Self> {
        let (i, platform_id) = le_u32(i)?;
        let (i, lump) = le_u32(i)?;
        let filtering = TextureFilteringMode::from_u8((lump >> 24) as u8).unwrap();
        let addr = ((lump >> 16) & 0b000000011111111) as u16;
        let addr_h = TextureAddressingMode::from_u8(((addr & 0b11110000) >> 4) as u8).unwrap();
        let addr_l = TextureAddressingMode::from_u8((addr & 0b00001111) as u8).unwrap();
        let addressing = [addr_h, addr_l];
        let (i, name) = bytes::complete::take(32usize)(i)?;
        let name = String::from_utf8_lossy(name).to_string();
        let (i, mask_name) = bytes::complete::take(32usize)(i)?;
        let mask_name = String::from_utf8_lossy(mask_name).to_string();
        let (i, raster_format) = le_u32(i)?;

        let mut has_alpha = false;
        let mut d3d_format = 0;
        let (i, temp0) = le_u32(i)?;
        if version < 0x36003 {
            // III & VC
            has_alpha = temp0 > 0;
        } else {
            // SA
            d3d_format = temp0;
        }

        let (i, width) = le_u16(i)?;
        let (i, height) = le_u16(i)?;
        let (i, depth) = le_u8(i)?;
        let (i, num_levels) = le_u8(i)?;
        let (i, raster_type) = le_u8(i)?;

        let mut compression = 0;
        let mut cube_texture = false;
        let mut auto_mipmaps = false;
        let mut compressed = false;
        let (i, temp0) = le_u8(i)?;
        if version < 0x36003 {
            // III & VC
            compression = temp0;
        } else {
            // SA
            has_alpha = ((temp0 >> 7) & 1) > 0;
            cube_texture = ((temp0 >> 6) & 1) > 0;
            auto_mipmaps = ((temp0 >> 5) & 1) > 0;
            compressed = ((temp0 >> 4) & 1) > 0;
        }

        let data = i.to_vec();

        Ok((
            &[],
            RpRasterPC {
                platform_id,
                filtering,
                addressing,
                name,
                mask_name,
                raster_format,
                d3d_format,
                width,
                height,
                depth,
                num_levels,
                raster_type,
                compression,
                has_alpha,
                cube_texture,
                auto_mipmaps,
                compressed,
                data,
            },
        ))
    }
}
