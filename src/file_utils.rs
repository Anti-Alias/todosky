use std::path::Path;
use std::io;

pub fn create_parent_path_of(path: impl AsRef<Path>) -> Result<(), io::Error> {
    let path = path.as_ref();
    println!("Path: {}", path.display());
    if let Some(parent_path) = path.parent() {
        std::fs::create_dir_all(parent_path)?;
    }
    Ok(())
}
