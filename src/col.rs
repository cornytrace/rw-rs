// GTA Collision files, version 1

use nom::{bytes::complete::*, multi::count, number::complete::*, IResult};
use nom_derive::{Nom, Parse};

const FOURCC_V1: &[u8] = b"COLL";

type TVector = crate::bsf::geo::RwV3d;

#[derive(Clone, Copy, Debug, Nom)]
pub struct TBounds {
    pub radius: f32,
    pub center: TVector,
    pub min: TVector,
    pub max: TVector,
}

#[derive(Clone, Copy, Debug, Nom)]
pub struct TSurface {
    pub material: u8,
    pub flag: u8,
    pub brightness: u8,
    pub light: u8,
}

#[derive(Clone, Copy, Debug, Nom)]
pub struct TSphere {
    pub radius: f32,
    pub center: TVector,
    pub surface: TSurface,
}

#[derive(Clone, Copy, Debug, Nom)]
pub struct TBox {
    pub min: TVector,
    pub max: TVector,
    pub surface: TSurface,
}

#[derive(Clone, Copy, Debug, Nom)]
pub struct TVertex(pub [f32; 3]);

#[derive(Clone, Copy, Debug, Nom)]
pub struct TFace {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub surface: TSurface,
}

#[derive(Clone, Debug)]
pub struct CollV1 {
    pub model_name: [u8; 22],
    pub model_id: u16,
    pub bounds: TBounds,
    pub spheres: Vec<TSphere>,
    pub boxes: Vec<TBox>,
    pub vertices: Vec<TVertex>,
    pub faces: Vec<TFace>,
}

impl CollV1 {
    pub fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, _) = tag(FOURCC_V1)(i)?;
        let (i, _file_size) = le_u32(i)?;
        let (i, model_name) = take(22usize)(i)?;
        let model_name = model_name.try_into().unwrap();
        let (i, model_id) = le_u16(i)?;
        let (i, bounds) = TBounds::parse_le(i)?;

        let (i, num_spheres) = le_u32(i)?;
        let (i, spheres) = count(TSphere::parse_le, num_spheres as usize)(i)?;

        let (i, num_unk) = le_u32(i)?;
        assert!(num_unk == 0);

        let (i, num_boxes) = le_u32(i)?;
        let (i, boxes) = count(TBox::parse_le, num_boxes as usize)(i)?;

        let (i, num_vertices) = le_u32(i)?;
        let (i, vertices) = count(TVertex::parse_le, num_vertices as usize)(i)?;

        let (i, num_faces) = le_u32(i)?;
        let (i, faces) = count(TFace::parse_le, num_faces as usize)(i)?;

        Ok((
            i,
            Self {
                model_name,
                model_id,
                bounds,
                spheres,
                boxes,
                vertices,
                faces,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn it_works() -> Result<()> {
        let i = std::fs::read("comNbtm.col")?;
        let (_, coll) = CollV1::parse(&i).map_err(|err| err.to_owned())?;
        println!("{:?}", coll);

        Ok(())
    }
}
