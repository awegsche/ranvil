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
//
use mc_anvil::{Region, Save};


fn main() {
    println!("Hello, world!");

    let minecraft_saves_dir = dirs::config_dir().unwrap().join(".minecraft/saves");
    println!(
        "Minecraft saves directory: {}",
        minecraft_saves_dir.display()
    );

    let mut saves = minecraft_saves_dir
        .read_dir()
        .unwrap()
        .filter_map(Result::ok)
        .map(|p| Save::from_path(p.path()))
        .filter_map(Result::ok)
        .collect::<Vec<Save>>();

    for save in saves.iter() {
        println!("{:?}", save);
    }

    let save1 = &mut saves[0];

    save1.load_region(0, 0);

    if let Some(region) = save1.get_region(0, 0) {
        let chunk_data = region.get_chunk_nbt_data(0).unwrap().unwrap();

        if let Ok(chunk) = from_bytes(&chunk_data) {
            if let Some(NbtField {
                value: NbtValue::List(NbtList::Compound(sections)),
                ..
            }) = chunk.get("sections")
            {
                println!("sections found: {} entries", sections.len());

                for section in sections.iter()
                //    .filter_map(|s| match s.value {
                //    NbtValue::Compound(c) => Some(c),
                //    _ => None,
                //})
                {
                    if let Some(y) = section.get("Y") {
                        println!("section Y: {}", y);
                    }
                    if let Some(palette) = section.get_path(&["block_states", "palette"]) {
                        println!("-- palette: {}", palette);
                    }
                }
            }
        }
    }
}
