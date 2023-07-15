use nom::bytes::complete::take;
use nom::multi::many0;
use nom::number::complete::le_u32;
use nom::IResult;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

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
        match self {
            ChunkType::Struct => false,
            ChunkType::String => false,
            ChunkType::Frame => false,
            ChunkType::BinMeshPLG => false,
            ChunkType::MaterialEffectsPLG => false,
            _ => true,
        }
    }
}

fn parse_chunk_content<'a>(
    ty: &ChunkType,
    size: u32,
    i: &'a [u8],
) -> IResult<&'a [u8], BsfChunkContent> {
    match ty {
        _ => take(size)(i).map(|(i, data)| (i, BsfChunkContent::Data(data.to_vec()))),
    }
}

#[derive(Debug)]
pub struct BsfChunk {
    pub ty: ChunkType,
    pub size: u32,
    pub version: u32,
    pub content: BsfChunkContent,
    pub children: Vec<BsfChunk>,
}

pub fn parse_bsf_chunk(i: &[u8]) -> IResult<&[u8], BsfChunk> {
    let (i, ty) = le_u32(i)?;
    let ty = ChunkType::from_u32(ty).unwrap_or_else(|| unimplemented!("0x{:08X}", ty));
    let (i, size) = le_u32(i)?;
    let (i, version) = le_u32(i)?;
    let (i, data) = take(size)(i)?;
    let mut children = Vec::new();
    let mut content = BsfChunkContent::None;
    if ty.has_children() {
        (_, children) = many0(parse_bsf_chunk)(data)?;
    } else {
        (_, content) = parse_chunk_content(&ty, size, data)?;
    }

    Ok((
        i,
        BsfChunk {
            ty,
            size,
            version,
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
}

struct RwRGBA {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

struct RwTexCoords {
    u: f32,
    v: f32,
}

struct RpTriangle {
    vertex2: u16,
    vertex1: u16,
    material_id: u16,
    vertex3: u16,
}

struct RpGeometry {
    format: u32,
    num_triangles: u32,
    num_vertices: u32,
    num_morph: u32,
    prelit: Vec<RwRGBA>,
    tex_coords: Vec<RwTexCoords>,
    triangles: Vec<RpTriangle>,
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
