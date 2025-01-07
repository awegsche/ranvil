use byteorder::{BigEndian, ReadBytesExt};
use once_cell::sync::Lazy;
use regex::Regex;
use rnbt::{NbtField, NbtList, NbtValue, from_bytes, read_nbt};
use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
    path::{Path, PathBuf},
};
//use serde_nbt::{from_bytes};

const SECTOR_SIZE: usize = 4 * 1024;

#[derive(Debug)]
pub struct Save {
    pub name: String,
    pub path: PathBuf,
    pub regions: Vec<Region>,
}

impl std::fmt::Display for Save {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{} regions]", self.name, self.regions.len())
    }
}

#[derive(Debug)]
pub struct Region {
    x: i32,
    y: i32,

    data: Vec<u8>,
}

/// Returns a Vec<_> of all the minecraft savegames
pub fn get_saves() -> Option<Vec<Save>> {
    let minecraft_saves_dir = dirs::config_dir()?.join(".minecraft/saves");

    Some(
        minecraft_saves_dir
            .read_dir()
            .unwrap()
            .filter_map(Result::ok)
            .map(|p| Save::from_path(p.path()))
            .filter_map(Result::ok)
            .collect::<Vec<Save>>(),
    )
}

pub fn get_save<S: Into<String>>(name: S) -> Option<Save> {
    Save::from_path(dirs::config_dir()?.join(".minecraft/saves").join(name.into())).ok()
}

impl Region {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y, data: vec![] }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        if self.data.len() > 0 {
            return;
        }
        let mut reader = BufReader::new(File::open(path).unwrap());
        reader.read_to_end(&mut self.data).unwrap();
    }

    pub fn get_location(&self, index: usize) -> std::io::Result<Option<(u32, u32)>> {
        let mut cursor = Cursor::new(&self.data[index * 4..(index + 1) * 4]);

        let position = cursor.read_u24::<BigEndian>()? as u32;
        let size = cursor.read_u8()? as u32;

        if position != 0 && size != 0 {
            Ok(Some((position, size)))
        } else {
            Ok(None)
        }
    }

    pub fn get_chunk_data(&self, index: usize) -> std::io::Result<Option<&[u8]>> {
        if let Some((position, size)) = self.get_location(index)? {
            let begin = position as usize * SECTOR_SIZE;
            let end = (begin + size as usize * SECTOR_SIZE).min(self.data.len());
            Ok(Some(&self.data[begin..end]))
        } else {
            Ok(None)
        }
    }

    pub fn get_chunk_nbt_data(&self, index: usize) -> Result<Option<Vec<u8>>, MCLoadError> {
        if let Some((position, size)) = self.get_location(index)? {
            let begin = position as usize * SECTOR_SIZE;
            let end = (begin + size as usize * SECTOR_SIZE).min(self.data.len());
            let data = &self.data[begin..end];

            let mut cursor = Cursor::new(&data);
            let real_size = cursor.read_u32::<BigEndian>()? as usize;
            let compression = cursor.read_u8()?;

            if compression == 2 {
                let real_cursor = Cursor::new(&data[5..4 + real_size]);
                let mut decompressed = vec![];
                let mut decoder = flate2::read::ZlibDecoder::new(real_cursor);
                decoder.read_to_end(&mut decompressed).unwrap();
                Ok(Some(decompressed))
            } else {
                Err(MCLoadError::IncompatibleCompressionType(compression))
            }
        } else {
            Ok(None)
        }
    }
}

fn get_region_coords(filename: &str) -> (i32, i32) {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"r\.(-?\d+)\.(-?\d+)\.mca").unwrap());
    let caps = RE.captures(filename).unwrap();
    let x = caps.get(1).unwrap().as_str().parse().unwrap();
    let y = caps.get(2).unwrap().as_str().parse().unwrap();
    (x, y)
}

#[derive(Debug)]
pub enum MCLoadError {
    IoError(std::io::Error),
    PathNotFoundError,
    IncompatibleCompressionType(u8),
}

impl From<std::io::Error> for MCLoadError {
    fn from(e: std::io::Error) -> Self {
        MCLoadError::IoError(e)
    }
}

impl std::fmt::Display for MCLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MCLoadError::IoError(e) => write!(f, "IO error: {}", e),
            MCLoadError::PathNotFoundError => write!(f, "Path not found"),
            MCLoadError::IncompatibleCompressionType(t) => {
                write!(f, "Incompatible compression type {}", t)
            }
        }
    }
}

impl std::error::Error for MCLoadError {}

impl Save {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, MCLoadError> {
        if !path.as_ref().exists() {
            return Err(MCLoadError::PathNotFoundError);
        }

        let regions = path
            .as_ref()
            .join("region")
            .read_dir()?
            .filter_map(|res| match res {
                Ok(dir) => Some(get_region_coords(dir.file_name().to_str().unwrap())),
                Err(_) => None,
            })
            .map(|(x, y)| Region::new(x, y))
            .collect::<Vec<Region>>();

        Ok(Save {
            path: path.as_ref().to_owned(),
            name: path
                .as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            regions,
        })
    }

    pub fn get_region(&self, x: i32, y: i32) -> Option<&Region> {
        self.regions.iter().find(|r| r.x == x && r.y == y)
    }

    pub fn get_region_mut(&mut self, x: i32, y: i32) -> Option<&mut Region> {
        self.regions.iter_mut().find(|r| r.x == x && r.y == y)
    }

    pub fn get_region_path(&self, x: i32, y: i32) -> PathBuf {
        self.path.join(&format!("region/r.{}.{}.mca", x, y))
    }

    pub fn load_region(&mut self, x: i32, y: i32) {
        let path = self.get_region_path(x, y);
        if let Some(region) = self.get_region_mut(x, y) {
            region.load(path);
        }
    }
}
