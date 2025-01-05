use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read, Seek},
    path::Path,
};

#[derive(Copy, Clone, Debug)]
pub struct PakEntry {
    #[allow(unused)]
    hash: u32,
    nameofs_flags: u32,
    offset: u32,
    #[allow(unused)]
    datalen: u32,
    filesize: u32,
}

impl PakEntry {
    pub fn name_offset(&self) -> u32 {
        self.nameofs_flags & 0xFFFFFF
    }
    pub fn flags(&self) -> u8 {
        (self.nameofs_flags >> 24) as u8
    }
}

pub struct Pak {
    file: File,
    // entries: Box<[Entry]>,
    // names: Box<[u8]>,
    index: BTreeMap<Box<[u8]>, PakEntry>,
}

impl Pak {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut header = [0u8; 16];
        file.read_exact(&mut header)?;
        let header_size = u16::from_be_bytes([header[4], header[5]]);
        let entry_size = u16::from_be_bytes([header[6], header[7]]);
        let entry_count = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);
        let name_size = u32::from_be_bytes([header[12], header[13], header[14], header[15]]);
        if &header[..4] != b"PKG\n" || header_size != 16 || entry_size != 20 {
            return Err(io::Error::new(io::ErrorKind::Other, "invalid magic"));
        }
        let entries: Box<[PakEntry]> = (0..entry_count)
            .map(|_| {
                let mut entry = [0u8; 20];
                file.read_exact(&mut entry)?;
                let hash = u32::from_be_bytes([entry[0], entry[1], entry[2], entry[3]]);
                let nameofs_flags = u32::from_be_bytes([entry[4], entry[5], entry[6], entry[7]]);
                let offset = u32::from_be_bytes([entry[8], entry[9], entry[10], entry[11]]);
                let datalen = u32::from_be_bytes([entry[12], entry[13], entry[14], entry[15]]);
                let filesize = u32::from_be_bytes([entry[16], entry[17], entry[18], entry[19]]);
                Ok::<_, io::Error>(PakEntry {
                    hash,
                    nameofs_flags,
                    offset,
                    datalen,
                    filesize,
                })
            })
            .collect::<Result<_, io::Error>>()?;
        let mut names = vec![0u8; usize::try_from(name_size).unwrap()].into_boxed_slice();
        file.read_exact(&mut names)?;
        let mut index = BTreeMap::new();
        for entry in entries.iter() {
            let nameofs = usize::try_from(entry.name_offset()).unwrap();
            if nameofs > names.len() {
                continue;
            }
            let Some(name) = names.split_at(nameofs).1.split(|x| *x == b'\0').next() else {
                unreachable!();
            };
            index.insert(name.to_vec().into_boxed_slice(), *entry);
        }
        Ok(Self {
            file,
            // entries,
            // names,
            index,
        })
    }
    pub fn read(&mut self, file: &str) -> io::Result<Vec<u8>> {
        let entry = self
            .index
            .get(file.as_bytes())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "file not found in pak index"))?;
        if entry.flags() != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "pak compression not implemented",
            ));
        }
        self.file.seek(io::SeekFrom::Start(entry.offset.into()))?;
        let mut ret = vec![0u8; usize::try_from(entry.filesize).unwrap()];
        self.file.read_exact(&mut ret)?;
        Ok(ret)
    }
    pub fn file_list(&self) -> Vec<String> {
        self.index
            .keys()
            .flat_map(|x| std::str::from_utf8(x).ok().map(|x| x.to_owned()))
            .collect()
    }
}

pub struct WindowsDatEntry {
    offset: usize,
    filesize: usize,
}

pub struct WindowsDat {
    file: File,
    index: BTreeMap<Box<[u8]>, WindowsDatEntry>,
}

impl WindowsDat {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut header = [0u8; 4];
        file.read_exact(&mut header)?;
        let index_size = usize::try_from(u32::from_le_bytes(header)).unwrap();
        let mut indices = vec![0u8; 4 * index_size];
        file.read_exact(&mut indices)?;
        let index = indices
            .chunks_exact(4)
            .map(|offset| {
                let offset = usize::try_from(u32::from_le_bytes([
                    offset[0], offset[1], offset[2], offset[3],
                ]))
                .unwrap();
                file.seek(io::SeekFrom::Start(offset as u64))?;
                let mut size_namelen = [0u8; 8];
                file.read_exact(&mut size_namelen)?;
                let size = usize::try_from(u32::from_le_bytes([
                    size_namelen[0],
                    size_namelen[1],
                    size_namelen[2],
                    size_namelen[3],
                ]))
                .unwrap();
                let namelen = usize::try_from(u32::from_le_bytes([
                    size_namelen[4],
                    size_namelen[5],
                    size_namelen[6],
                    size_namelen[7],
                ]))
                .unwrap();
                let mut name = vec![0u8; namelen];
                file.read_exact(&mut name)?;

                Ok((
                    name.into_boxed_slice(),
                    WindowsDatEntry {
                        offset: offset + size_namelen.len() + namelen,
                        filesize: size,
                    },
                ))
            })
            .collect::<io::Result<_>>()?;
        Ok(Self { file, index })
    }
    pub fn read(&mut self, file: &str) -> io::Result<Vec<u8>> {
        let entry = self
            .index
            .get(file.as_bytes())
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "file not found in dat index"))?;
        self.file.seek(io::SeekFrom::Start(entry.offset as u64))?;
        let mut ret = vec![0u8; entry.filesize];
        self.file.read_exact(&mut ret)?;
        Ok(ret)
    }
    pub fn file_list(&self) -> Vec<String> {
        self.index
            .keys()
            .flat_map(|x| std::str::from_utf8(x).ok().map(|x| x.to_owned()))
            .collect()
    }
}

#[cfg(target_os = "linux")]
pub type Data = Pak;
#[cfg(target_os = "windows")]
pub type Data = WindowsDat;

#[cfg(test)]
mod test {
    use super::Pak;

    #[test]
    fn test() {
        let path =
            "/data/data/Games/SteamLibrary/steamapps/common/FTL Faster Than Light/data/ftl.dat";
        let mut file = Pak::open(path).unwrap();
        assert!(file
            .read("data/text_misc.xml")
            .unwrap()
            .starts_with(b"<?xml"));
        for k in file.index.keys().cloned().collect::<Vec<_>>() {
            let s = std::str::from_utf8(&k).unwrap();
            if !s.ends_with(".xml") {
                continue;
            }
            let v = file.read(s).unwrap();
            let v = std::str::from_utf8(&v)
                .unwrap()
                .replace("</event>-", "</event>")
                .replace("/>.\r", "/>\r")
                .replace(">1.f<", ">1.0<");
            if v.contains("<event ") && !v.contains("<sectorDescription ") {
                if let Err(err) = quick_xml::de::from_str::<crate::xml::XmlEvents>(&v) {
                    eprintln!("{s}");
                    panic!("{err}");
                }
            } else if s.contains("luep") && !s.contains("text_") {
                if let Err(err) = quick_xml::de::from_str::<crate::xml::XmlBlueprints>(&v) {
                    eprintln!("{s}");
                    panic!("{err}");
                }
            } else if s.contains("text_") {
                if let Err(err) = quick_xml::de::from_str::<crate::xml::XmlText>(&v) {
                    eprintln!("{s}");
                    panic!("{err}");
                }
            }
        }
    }
}
