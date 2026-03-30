use std::path::PathBuf;
use thiserror::Error;
use crate::APP_NAME;

const SETTINGS_FILE_NAME: &str = "settings.yml";

/// Represents the paths to various files the application cares about.
#[derive(Debug)]
pub struct Paths {
    pub settings_file: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self, PathsError> {
        let home_dir = match std::env::home_dir() {
            Some(home_dir) => home_dir,
            None => return Err(PathsError::HomeDirectoryNotFound),
        };
        let mut settings_file = home_dir;
        settings_file.push(".local");
        settings_file.push("state");
        settings_file.push(APP_NAME);
        settings_file.push(SETTINGS_FILE_NAME);
        Ok(Self {
            settings_file
        })
    }
}


#[derive(Error, Debug, Copy, Clone)]
pub enum PathsError {
    #[error("Home directory not found")]
    HomeDirectoryNotFound,    
}
