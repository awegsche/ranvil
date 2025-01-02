#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Chunk {
    DataVersion: i32,
    xPos: i32,
    yPos: i32,
    zPos: i32,
    Status: String,
    LastUpdate: i64,
    sections: Vec<Section>,
    block_entities: Vec<BlockEntity>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Section {
    Y: u8,
    block_states: BlockStates,
    biomes: Biomes,
    BlockLight: Vec<u8>,
    SkyLight: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BlockStates {
    data: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Biomes {
    data: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum BlockEntity {
   Banners(Banners), 
   Barrel(Barrel),
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Banners {
    CustomName: String,
    Patterns: Vec<BannerPattern>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BannerPattern {
    Color: i32,
    Pattern: String,
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Barrel {
    CustomName: String,
}
