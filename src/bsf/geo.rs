use nom::multi::count;
use nom::number::complete::le_u32;
use nom::IResult;
use nom_derive::{Nom, Parse};

use super::tex::{RpSurfProp, RwRGBA};
use crate::bsf::tex::RwTexCoords;

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
    pub surface_prop: Option<RpSurfProp>,
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
    pub fn parse(i: &[u8], version: u32) -> IResult<&[u8], Self> {
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

        let mut surface_prop = None;
        if version < 0x34000 {
            let s;
            (i, s) = RpSurfProp::parse_le(i)?;
            surface_prop = Some(s);
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
                surface_prop,
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
