pub mod geo;
pub mod tex;

use nom::bytes::complete::take;
use nom::multi::many0;
use nom::number::complete::le_u32;
use nom::IResult;
use nom_derive::*;

use self::geo::RpGeometry;
use self::tex::{RpMaterial, RpMaterialList, RpRasterPC, RpTexture};

macro_rules! parse_children {
    ($i:ident, $enum:path) => {{
        let (i, children) = many0(Chunk::parse)($i)?;
        Ok((i, ($enum, Some(children))))
    }};
}

macro_rules! parse_struct_and_children {
    ($i:ident, $version:ident, $enum:path, $struc:ty) => {{
        let (i, mut children) = many0(Chunk::parse)($i)?;
        let mut struc = None;
        children.retain(|e| match &e.content {
            Self::Struct(vec) => {
                if let Ok(s) = <$struc>::parse(&vec[..], $version) {
                    struc = Some(s.1);
                    return false;
                }
                true
            }
            _ => true,
        });

        // TODO: proper error handling if struc is None
        Ok((i, ($enum(struc.unwrap()), Some(children))))
    }};
}

#[derive(Clone, Debug)]
#[repr(u32)]
pub enum ChunkContent {
    Section((u32, Vec<u8>)), // For sections we can't yet parse
    Struct(Vec<u8>), // The contents of a known section will be in that enum variant, this is only for child Struct sections of unknown sections
    String(String),
    Extension,
    Camera,
    Texture(RpTexture),
    Material(RpMaterial),
    MaterialList(RpMaterialList),
    FrameList,
    Geometry(RpGeometry),
    Clump,
    Atomic,
    Raster(RpRasterPC),
    TextureDictionary,
    GeometryList,
}
impl ChunkContent {
    fn parse(
        i: &[u8],
        ty: u32,
        version: u32,
    ) -> IResult<&[u8], (ChunkContent, Option<Vec<Chunk>>)> {
        match ty {
            0x00000001 => Ok((&[] as &[u8], (Self::Struct(i.to_vec()), None))),
            0x00000002 => Ok((
                &[] as &[u8],
                (
                    Self::String(
                        std::str::from_utf8(i)
                            .unwrap_or("")
                            .trim_matches('\0')
                            .to_owned(),
                    ),
                    None,
                ),
            )),
            0x00000003 => parse_children!(i, Self::Extension),
            0x00000005 => parse_children!(i, Self::Camera),
            0x00000006 => parse_struct_and_children!(i, version, Self::Texture, RpTexture),
            0x00000007 => parse_struct_and_children!(i, version, Self::Material, RpMaterial),
            0x00000008 => {
                parse_struct_and_children!(i, version, Self::MaterialList, RpMaterialList)
            }
            0x0000000E => parse_children!(i, Self::FrameList),
            0x0000000F => parse_struct_and_children!(i, version, Self::Geometry, RpGeometry),
            0x00000010 => parse_children!(i, Self::Clump),
            0x00000014 => parse_children!(i, Self::Atomic),
            0x00000015 => parse_struct_and_children!(i, version, Self::Raster, RpRasterPC),
            0x00000016 => parse_children!(i, Self::TextureDictionary),
            0x0000001A => parse_children!(i, Self::GeometryList),

            _ => Ok((&[] as &[u8], (Self::Section((ty, i.to_vec())), None))),
        }
    }
}

#[derive(Copy, Clone, Debug, Nom)]
pub struct ChunkHeader {
    pub version: u32,
    pub build: u32,
}

impl ChunkHeader {
    pub fn parse(i: &[u8]) -> IResult<&[u8], ChunkHeader> {
        let (i, lib_id) = le_u32(i)?;

        Ok((
            i,
            ChunkHeader {
                version: get_chunk_version(lib_id),
                build: get_chunk_build(lib_id),
            },
        ))
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub header: ChunkHeader,
    pub content: ChunkContent,
    pub children: Option<Vec<Chunk>>,
}

impl Chunk {
    pub fn parse(i: &[u8]) -> IResult<&[u8], Chunk> {
        let (i, ty) = le_u32(i)?;
        let (i, size) = le_u32(i)?;
        let (i, header) = ChunkHeader::parse(i)?;
        let (i, data) = take(size)(i)?;
        let (_, (content, children)) = ChunkContent::parse(data, ty, header.version)?;

        Ok((
            i,
            Chunk {
                header,
                content,
                children,
            },
        ))
    }

    pub fn get_children(&self) -> &[Chunk] {
        if let Some(children) = &self.children {
            children
        } else {
            &[]
        }
    }
}

pub fn get_chunk_version(lib_id: u32) -> u32 {
    if lib_id & 0xFFFF0000 != 0 {
        return ((lib_id >> 14 & 0x3FF00) + 0x30000) | (lib_id >> 16 & 0x3F);
    }
    lib_id << 8
}

pub fn get_chunk_build(lib_id: u32) -> u32 {
    if lib_id & 0xFFFF0000 != 0 {
        return lib_id & 0xFFFF;
    }
    0
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let file = fs::read("player.dff")?;
        let (_, dff) = Chunk::parse(&file).unwrap();
        dbg!(dff);
        Ok(())
    }
}
