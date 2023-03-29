use std::io::Cursor;
use std::mem::size_of;

use binrw::binread;
use binrw::until_eof;
use binrw::BinRead;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(FromPrimitive)]
#[repr(u32)]
pub enum ChunkType {
    Struct = 0x00000001,
    Clump = 0x00000010,
}

#[derive(BinRead)]
#[brw(little)]
pub struct Bsf {
    #[br(parse_with = until_eof)]
    chunks: Vec<BsfChunk>,
}

#[derive(BinRead)]
#[brw(little)]
pub struct BsfChunk {
    pub ty: u32,
    pub size: u32,
    pub version: u32,
    #[br(count = size)]
    data: Vec<u8>,
}
impl BsfChunk {
    pub fn get_children(&self) -> Vec<BsfChunk> {
        let mut res = Vec::new();
        if has_children(self.ty) {
            let mut cursor = Cursor::new(&self.data);
            while cursor.position() < self.size.into() {
                res.push(BsfChunk::read(&mut cursor).unwrap())
            }
        }
        res
    }
}

fn has_children(ty: u32) -> bool {
    match ChunkType::from_u32(ty) {
        Some(ChunkType::Struct) => false,
        Some(ChunkType::Clump) => true,
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use anyhow::Result;

    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let mut file = File::open("player.dff")?;
        let _dff = Bsf::read(&mut file)?;
        Ok(())
    }
}
