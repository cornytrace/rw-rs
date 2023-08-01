use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;

use anyhow::bail;
use binrw::until_eof;
use binrw::BinRead;

use anyhow::Result;

pub trait ReadSeek: Read + Seek + Send + Sync {}
impl<T: Read + Seek + Send + Sync> ReadSeek for T {}

pub struct Img<'a> {
    entries: HashMap<String, DirEnt>,
    img_reader: Box<dyn ReadSeek + 'a>,
}
impl<'a> Img<'a> {
    pub fn new(path: &Path) -> Result<Img<'a>> {
        if !path.extension().map_or(false, |x| x == "img") {
            bail!("File does not end in .img")
        }
        let img_file = File::open(path.clone())?;
        let dir_path = path.with_extension("dir");
        if let Ok(mut dir_file) = File::open(dir_path) {
            return Img::from_v1(img_file, &mut dir_file);
        } else {
            return Img::from_v2(img_file);
        }
    }

    pub fn from_v1<R, S>(img_reader: R, mut dir_reader: S) -> Result<Img<'a>>
    where
        R: ReadSeek + 'a,
        S: ReadSeek,
    {
        let mut map = HashMap::new();
        {
            let list = DirList::read(&mut dir_reader)?;
            for entry in list.entries {
                map.insert(
                    entry
                        .name
                        .clone()
                        .into_string()
                        .unwrap()
                        .to_ascii_lowercase(),
                    entry,
                );
            }
        }
        Ok(Img {
            entries: map,
            img_reader: Box::new(img_reader),
        })
    }

    pub fn from_v2<R>(mut _img_reader: R) -> Result<Img<'a>>
    where
        R: ReadSeek,
    {
        unimplemented!("V2 .IMG files (San Andreas) not yet supported")
    }

    pub fn get_entry(&self, name: &str) -> Option<DirEnt> {
        return self.entries.get(name).cloned();
    }

    pub fn get_file(&mut self, name: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.get_entry(&name.to_ascii_lowercase()) {
            self.img_reader
                .seek(SeekFrom::Start(entry.offset as u64 * 2048))
                .unwrap();
            let mut res = vec![0; entry.size as usize * 2048];
            self.img_reader.read_exact(&mut res).unwrap();
            return Some(res);
        }
        None
    }
}

#[derive(BinRead)]
#[brw(little)]
pub struct DirList {
    #[br(parse_with = until_eof)]
    pub entries: Vec<DirEnt>,
}

#[derive(BinRead, Clone)]
#[brw(little)]
pub struct DirEnt {
    pub offset: u32,
    pub size: u32,
    #[brw(map = |x: [u8; 24]| CString::new(x.split(|x| *x == b'\0').next().unwrap()).unwrap())]
    pub name: CString,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let _list = Img::new(Path::new("/mnt/winstor/Games/GTAIII/models/gta3.img"))?;
        Ok(())
    }
}
