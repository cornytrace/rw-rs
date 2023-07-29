use nom::bytes::complete::take;
use nom::multi::{count, many0};
use nom::number::complete::{le_f32, le_u32};
use nom::IResult;
use nom_derive::*;
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
}

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

#[derive(Clone, Copy, Debug, Nom)]
pub struct RpTriangle {
    pub vertex2: u16,
    pub vertex1: u16,
    pub material_id: u16,
    pub vertex3: u16,
}

impl RpTriangle {
    pub fn as_arr(self) -> [u16; 3] {
        [self.vertex1, self.vertex2, self.vertex3]
    }
}

#[derive(Clone, Debug, Nom)]
pub struct RwV3d {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl RwV3d {
    pub fn as_arr(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

#[derive(Clone, Debug, Nom)]
pub struct RwSphere {
    pub pos: RwV3d,
    pub radius: f32,
}

#[derive(Clone, Debug)]
pub struct RpGeometry {
    format: u32,
    pub num_triangles: u32,
    pub num_vertices: u32,
    pub num_morphs: u32,
    pub prelit: Vec<RwRGBA>,
    pub tex_coords: Vec<Vec<RwTexCoords>>,
    pub triangles: Vec<RpTriangle>,
    pub vertices: Vec<RwV3d>,
    pub normals: Vec<RwV3d>,
}

const RP_GEOMETRYTRISTRIP: u32 = 0x00000001;
const RP_GEOMETRYTEXTURED: u32 = 0x00000004;
const RP_GEOMETRYPRELIT: u32 = 0x00000008;
const RP_GEOMETRYTEXTURED2: u32 = 0x00000080;
const RP_GEOMETRYNATIVE: u32 = 0x01000000;

impl RpGeometry {
    fn parse(i: &[u8], version: u32) -> IResult<&[u8], Self> {
        let (i, format) = le_u32(i)?;
        let (i, num_triangles) = le_u32(i)?;
        let (i, num_vertices) = le_u32(i)?;
        let (mut i, num_morphs) = le_u32(i)?;

        let mut num_tex_sets = (format & 0x00FF0000) << 16;
        if num_tex_sets == 0 {
            if format & RP_GEOMETRYTEXTURED != 0 {
                num_tex_sets = 1;
            }
            if format & RP_GEOMETRYTEXTURED2 != 0 {
                num_tex_sets = 2;
            }
        }

        if version < 0x34000 {
            let _ambient;
            let _specular;
            let _diffuse;
            (i, _ambient) = le_f32(i)?;
            (i, _specular) = le_f32(i)?;
            (i, _diffuse) = le_f32(i)?;
        }

        let mut prelit = Vec::new();
        let mut tex_coords = Vec::new();
        let mut triangles = Vec::new();

        if format & RP_GEOMETRYNATIVE == 0 {
            if format & RP_GEOMETRYPRELIT != 0 {
                (i, prelit) = count(RwRGBA::parse_le, num_vertices as usize)(i)?;
            }
            (i, tex_coords) = count(
                count(RwTexCoords::parse_le, num_vertices as usize),
                num_tex_sets as usize,
            )(i)?;
            (i, triangles) = count(RpTriangle::parse_le, num_triangles as usize)(i)?;
        }

        // TODO: Multiple Morph sets

        let (i, _) = RwSphere::parse_le(i)?;
        let (i, has_vertices) = le_u32(i)?;
        let (mut i, has_normals) = le_u32(i)?;

        let mut vertices = Vec::new();
        if has_vertices > 0 {
            (i, vertices) = count(RwV3d::parse_le, num_vertices as usize)(i)?;
        }

        let mut normals = Vec::new();
        if has_normals > 0 {
            (i, normals) = count(RwV3d::parse_le, num_vertices as usize)(i)?;
        }

        Ok((
            i,
            Self {
                format,
                num_triangles,
                num_vertices,
                num_morphs,
                prelit,
                tex_coords,
                triangles,
                vertices,
                normals,
            },
        ))
    }

    pub fn is_tristrip(&self) -> bool {
        self.format & RP_GEOMETRYTRISTRIP > 0
    }
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
