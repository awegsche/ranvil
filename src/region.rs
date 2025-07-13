use byteorder::{BigEndian, ReadBytesExt};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
    path::Path,
};

pub const SECTOR_SIZE: usize = 4 * 1024;

// -------------------------------------------------------------------------------------------------
// ---- Region -------------------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

/// Region struct.
///
/// Contains raw byte data. If you want the chunks, expand this into `ChunkRegion`.
#[derive(Debug)]
pub struct Region {
    x: i32,
    y: i32,

    data: Vec<u8>,
}

// -------------------------------------------------------------------------------------------------
// ---- impls --------------------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------
impl Region {
    pub fn new<P: AsRef<Path>>(x: i32, y: i32, path: P) -> Self {
        let mut data = vec![];
        let mut reader = BufReader::new(File::open(path).unwrap());
        reader.read_to_end(&mut data).unwrap();
        Self { x, y, data }
    }


    /// returns the location of the chunk at index `index`
    fn get_location(&self, index: usize) -> std::io::Result<Option<(u32, u32)>> {
        let mut cursor = Cursor::new(&self.data[index * 4..(index + 1) * 4]);

        let position = cursor.read_u24::<BigEndian>()? as u32;
        let size = cursor.read_u8()? as u32;

        if position != 0 && size != 0 {
            Ok(Some((position, size)))
        } else {
            Ok(None)
        }
    }

    /// gets the chunk data for the chunk at index `index`.
    ///
    /// Note: this is the raw encrypted byte data, it must be decrypted and expanded into an
    /// NBT value in order to be used.
    pub fn get_chunk_data(&self, index: usize) -> std::io::Result<Option<&[u8]>> {
        if let Some((position, size)) = self.get_location(index)? {
            let begin = position as usize * SECTOR_SIZE;
            let end = (begin + size as usize * SECTOR_SIZE).min(self.data.len());
            Ok(Some(&self.data[begin..end]))
        } else {
            Ok(None)
        }
    }

    /// gets the chunk data for the chunk at index `index`.
    ///
    /// Note: this gets the raw, decrypted chunk data.
    /// It must be expanded into an NBT value in order to be used.
    pub fn get_chunk_nbt_data(
        &self,
        index: usize,
    ) -> Result<Option<Vec<u8>>, crate::error::MCLoadError> {
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
                Err(crate::error::MCLoadError::IncompatibleCompressionType(
                    compression,
                ))
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_x_coord(&self) -> i32 {
        self.x
    }
    pub fn get_z_coord(&self) -> i32 {
        self.y
    }
}

// -------------------------------------------------------------------------------------------------
// ---- helper functions ---------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

pub fn get_region_coords(filename: &str) -> (i32, i32) {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"r\.(-?\d+)\.(-?\d+)\.mca").unwrap());
    let caps = RE.captures(filename).unwrap();
    let x = caps.get(1).unwrap().as_str().parse().unwrap();
    let y = caps.get(2).unwrap().as_str().parse().unwrap();
    (x, y)
}
