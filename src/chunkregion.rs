use std::path::PathBuf;
use rnbt::{NbtField, NbtValue};
use crate::region::Region;

#[derive(Debug)]
pub struct ChunkRegion {
    pub x: i32,
    pub y: i32,
    pub path: PathBuf,
    pub chunks: Vec<Option<NbtField>>,
}

impl ChunkRegion {
    pub fn new(x: i32, y: i32, path: PathBuf) -> Self {
        Self { 
            x, 
            y, 
            path,
            chunks: Vec::new(),
        }
    }
}

impl From<Region> for ChunkRegion {
    fn from(region: Region) -> Self {
        let mut chunks = Vec::new();
        
        // A region contains 32x32 chunks (1024 total)
        for index in 0..1024 {
            if let Ok(Some(chunk_data)) = region.get_chunk_nbt_data(index) {
                // Parse the NBT data using rnbt::from_bytes
                if let Ok(nbt_field) = rnbt::from_bytes(&chunk_data) {
                    chunks.push(Some(nbt_field));
                } else {
                    chunks.push(None);
                }
            } else {
                chunks.push(None);
            }
        }
        
        Self {
            x: region.get_x_coord(),
            y: region.get_z_coord(),
            path: PathBuf::new(), // We don't have path info in Region, so use empty
            chunks,
        }
    }
} 