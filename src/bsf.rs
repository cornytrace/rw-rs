use nom::bytes::complete::take;
use nom::multi::many0;
use nom::number::complete::le_u32;
use nom::IResult;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(FromPrimitive, Debug)]
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
    Clump = 0x00000010,
    Frame = 0x0253F2FE,
}
impl ChunkType {
    fn has_children(&self) -> bool {
        match self {
            ChunkType::Struct => false,
            ChunkType::String => false,
            ChunkType::Frame => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct BsfChunk {
    pub ty: ChunkType,
    pub size: u32,
    pub version: u32,
    pub content: BsfChunkContent,
}

pub fn parse_bsf_chunk(i: &[u8]) -> IResult<&[u8], BsfChunk> {
    let (i, ty) = le_u32(i)?;
    let ty = ChunkType::from_u32(ty).unwrap_or_else(|| unimplemented!("0x{:08X}", ty));
    let (i, size) = le_u32(i)?;
    let (i, version) = le_u32(i)?;
    let (i, data) = take(size)(i)?;
    let content;
    let mut i = i;
    if ty.has_children() {
        let (i2, res) = many0(parse_bsf_chunk)(data)?;
        i = i2;
        content = BsfChunkContent::Children(res);
    } else {
        content = BsfChunkContent::Data(data.to_vec());
    }

    Ok((
        i,
        BsfChunk {
            ty,
            size,
            version,
            content,
        },
    ))
}

#[derive(Debug)]
pub enum BsfChunkContent {
    Data(Vec<u8>),
    Children(Vec<BsfChunk>),
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
