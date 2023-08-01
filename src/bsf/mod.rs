mod geo;
mod tex;

use nom::bytes::complete::take;
use nom::multi::many0;
use nom::number::complete::le_u32;
use nom::IResult;
use nom_derive::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use self::geo::RpGeometry;
use self::tex::RpMaterial;

#[derive(FromPrimitive, Debug, PartialEq)]
#[repr(u32)]
pub enum ChunkType {
    Struct = 0x00000001,
    String = 0x00000002,
    Extension = 0x00000003,
    Camera = 0x00000005,
    Texture = 0x00000006,
    Material = 0x00000007,
    MaterialList = 0x00000008,
    FrameList = 0x0000000E,
    Geometry = 0x0000000F,
    Clump = 0x00000010,
    Atomic = 0x00000014,
    GeometryList = 0x0000001A,
    MaterialEffectsPLG = 0x00000120,
    BinMeshPLG = 0x0000050E,
    Frame = 0x0253F2FE,
}
impl ChunkType {
    fn has_children(&self) -> bool {
        !matches!(
            self,
            ChunkType::Struct
                | ChunkType::String
                | ChunkType::Frame
                | ChunkType::BinMeshPLG
                | ChunkType::MaterialEffectsPLG
        )
    }
}

fn parse_chunk_content<'a>(
    ty: &ChunkType,
    size: u32,
    version: u32,
    i: &'a [u8],
) -> IResult<&'a [u8], BsfChunkContent> {
    match ty {
        ChunkType::String => take(size)(i).map(|(i, data)| {
            (
                i,
                BsfChunkContent::String(std::str::from_utf8(data).unwrap().to_owned()),
            )
        }),
        ChunkType::Geometry => RpGeometry::parse(i, version)
            .map(|(i, geometry)| (i, BsfChunkContent::RpGeometry(geometry))),
        ChunkType::Material => RpMaterial::parse(i, version)
            .map(|(i, material)| (i, BsfChunkContent::RpMaterial(material))),
        _ => take(size)(i).map(|(i, data)| (i, BsfChunkContent::Data(data.to_vec()))),
    }
}

#[derive(Debug)]
pub struct BsfChunk {
    pub ty: ChunkType,
    pub size: u32,
    pub lib_id: u32,
    pub content: BsfChunkContent,
    pub children: Vec<BsfChunk>,
}

impl BsfChunk {
    pub fn get_version(&self) -> u32 {
        get_chunk_version(self.lib_id)
    }

    pub fn get_build(&self) -> u32 {
        get_chunk_build(self.lib_id)
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

pub fn parse_bsf_chunk(i: &[u8]) -> IResult<&[u8], BsfChunk> {
    let (i, ty) = le_u32(i)?;
    let ty = ChunkType::from_u32(ty).unwrap_or_else(|| unimplemented!("0x{:08X}", ty));
    let (i, size) = le_u32(i)?;
    let (i, lib_id) = le_u32(i)?;
    let (i, data) = take(size)(i)?;
    let mut children = Vec::new();
    let mut content = BsfChunkContent::None;
    if ty.has_children() {
        (_, children) = many0(parse_bsf_chunk)(data)?;
        if !children.is_empty() && children[0].ty == ChunkType::Struct {
            (_, content) = parse_chunk_content(
                &ty,
                children[0].size,
                get_chunk_version(lib_id),
                &data[3 * 4..],
            )?;
        }
    } else {
        (_, content) = parse_chunk_content(&ty, size, get_chunk_version(lib_id), data)?;
    }

    Ok((
        i,
        BsfChunk {
            ty,
            size,
            lib_id,
            content,
            children,
        },
    ))
}

#[derive(Debug)]
pub enum BsfChunkContent {
    None,
    Data(Vec<u8>),
    String(String),
    RpGeometry(RpGeometry),
    RpMaterial(RpMaterial),
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let file = fs::read("player.dff")?;
        let (_, dff) = parse_bsf_chunk(&file).unwrap();
        dbg!(dff);
        Ok(())
    }
}
