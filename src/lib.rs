use std::path::{Path, PathBuf};

pub mod chunkregion;
pub mod error;
pub mod region;

// -------------------------------------------------------------------------------------------------
// ---- public functions ---------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------
/// Returns a Vec<_> of all the minecraft savegames
pub fn get_saves() -> Option<Vec<SaveMeta>> {
    let minecraft_saves_dir = dirs::config_dir()?.join(".minecraft/saves");

    Some(
        minecraft_saves_dir
            .read_dir()
            .unwrap()
            .filter_map(Result::ok)
            .map(|p| SaveMeta::from_path(p.path()))
            .filter_map(Result::ok)
            .collect::<Vec<_>>(),
    )
}

/// Returns a Vec<_> of all the minecraft savegames
/// for the given game instance (e.g. CurseForge installation)
pub fn get_saves_from_instance<P: AsRef<Path>>(instance: P) -> Option<Vec<SaveMeta>> {
    Some(
        instance
            .as_ref()
            .read_dir()
            .unwrap()
            .filter_map(Result::ok)
            .map(|p| SaveMeta::from_path(p.path()))
            .filter_map(Result::ok)
            .collect::<Vec<_>>(),
    )
}

pub fn get_save<S: Into<String>>(name: S) -> Option<Save> {
    SaveMeta::from_path(
        dirs::config_dir()?
            .join(".minecraft/saves")
            .join(name.into()),
    )
    .ok()
    .map(|meta| meta.into())
}

// -------------------------------------------------------------------------------------------------
// ---- structs ------------------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct SaveMeta {
    pub name: String,
    pub path: PathBuf,
    pub regions: Vec<(i32, i32)>,
}

#[derive(Debug, Clone)]
pub struct Save {
    pub meta: SaveMeta,
    pub regions: Vec<region::Region>,
}

#[derive(Debug, Clone)]
pub struct GridView<'a> {
    save: &'a SaveMeta,
}

// -------------------------------------------------------------------------------------------------
// ---- impls --------------------------------------------------------------------------------------
// -------------------------------------------------------------------------------------------------

impl std::fmt::Display for SaveMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{} regions]", self.name, self.regions.len())
    }
}

impl From<SaveMeta> for Save {
    fn from(meta: SaveMeta) -> Self {
        let root_path = meta.path.join("region");
        let regions = meta
            .regions
            .iter()
            .map(|(x, y)| {
                region::Region::new(*x, *y, root_path.join(&format!("r.{}.{}.mca", x, y)))
            })
            .collect();
        Self { meta, regions }
    }
}

impl std::fmt::Display for Save {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{} regions]", self.meta.name, self.regions.len())
    }
}

impl SaveMeta {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, error::MCLoadError> {
        if !path.as_ref().exists() {
            return Err(error::MCLoadError::PathNotFoundError);
        }

        let regions = path
            .as_ref()
            .join("region")
            .read_dir()?
            .filter_map(|res| match res {
                Ok(dir) => Some(region::get_region_coords(dir.file_name().to_str().unwrap())),
                Err(_) => None,
            })
            .collect::<Vec<_>>();

        Ok(Self {
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

    pub fn get_region_path(&self, x: i32, y: i32) -> PathBuf {
        self.path.join(&format!("region/r.{}.{}.mca", x, y))
    }

    fn has_region(&self, x: i32, y: i32) -> bool {
        self.regions.iter().any(|(_x, _y)| *_x == x && *_y == y)
    }

    pub fn get_grid_view<'a>(&'a self) -> GridView<'a> {
        GridView { save: self }
    }
}

impl<'a> std::fmt::Display for GridView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} [{} regions]", self.save.name, self.save.regions.len())?;

        let min_x = self.save.regions.iter().map(|(x, _)| *x).min().unwrap();
        let max_x = self.save.regions.iter().map(|(x, _)| *x).max().unwrap();
        let min_z = self.save.regions.iter().map(|(_, z)| *z).min().unwrap();
        let max_z = self.save.regions.iter().map(|(_, z)| *z).max().unwrap();

        write!(f, "    +")?;
        for _ in min_x..=max_x {
            write!(f, "-")?;
        }
        writeln!(f, "+")?;

        for z in min_z..=max_z {
            if z % 5 == 0 {
                write!(f, "{:3} |", z)?;
            } else {
                write!(f, "    |")?;
            }
            for x in min_x..=max_x {
                if self.save.has_region(x, z) {
                    write!(f, "X")?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f, "|")?;
        }
        write!(f, "    +")?;
        for _ in min_x..=max_x {
            write!(f, "-")?;
        }
        writeln!(f, "+")?;

        write!(f, "   ")?;
        let mut x = min_x;
        loop  {
            if x % 5 == 0 { break; }
            write!(f, " ")?;
            x += 1;
        }
        loop {
            if x > max_x { break; }
        write!(f, "{:3}  ", x)?;
        x += 5;
        }

        Ok(())
    }
}
