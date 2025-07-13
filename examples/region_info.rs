use ranvil::SaveMeta;

fn main() {
    let minecraft_saves_dir = dirs::config_dir().unwrap().join(".minecraft/saves");
    println!(
        "Minecraft saves directory: {}",
        minecraft_saves_dir.display()
    );

    let saves = minecraft_saves_dir
        .read_dir()
        .unwrap()
        .filter_map(Result::ok)
        .map(|p| SaveMeta::from_path(p.path()))
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    for save in saves.iter() {
        println!("{}", save.get_grid_view());
    }
}
