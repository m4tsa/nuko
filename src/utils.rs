use anyhow::Result;
use std::{fs, path::PathBuf};
use thiserror::Error;

const MANIFEST_FILE_NAME: &'static str = "Nuko.toml";
const MAX_UPWARDS_MANIFEST_PEEKS: usize = 8;

pub fn find_root_dir(dir: &str) -> Result<(PathBuf, PathBuf)> {
    // Check paths upwards if we are looking at the relative path
    let can_check_upwards = dir == ".";

    let path = fs::canonicalize(PathBuf::from(dir))?;

    let manifest_path = path.join(MANIFEST_FILE_NAME);

    if manifest_path.is_file() {
        Ok((path, manifest_path))
    } else {
        // Check parts upwards if we are on the default relative path
        if can_check_upwards {
            let mut up_path = path.clone();

            for _ in 0..MAX_UPWARDS_MANIFEST_PEEKS {
                if !up_path.pop() {
                    break;
                }

                let manifest_path = up_path.join(MANIFEST_FILE_NAME);

                if manifest_path.is_file() {
                    return Ok((up_path, manifest_path));
                }
            }
        }

        let abs_path = std::fs::canonicalize(path)?;

        return Err(UtilsError::CouldNotFindManifest(abs_path.to_string_lossy().into()).into());
    }
}

pub fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

#[derive(Error, Debug)]
pub enum UtilsError {
    #[error("could not find nuko manifest in {0}")]
    CouldNotFindManifest(String),
}
