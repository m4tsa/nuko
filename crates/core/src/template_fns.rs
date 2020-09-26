use crate::config::SiteConfig;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, path::PathBuf};

pub struct GetUrl {
    config: SiteConfig,
    out_path: PathBuf,
}
impl GetUrl {
    pub fn new(config: SiteConfig, out_path: PathBuf) -> GetUrl {
        Self { config, out_path }
    }
}

impl tera::Function for GetUrl {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let hash = args
            .get("hash")
            .map_or(false, |c| c == &tera::Value::Bool(true));

        let path = PathBuf::from(
            args.get("path")
                .expect("path argument is requried for get_url")
                .as_str()
                .unwrap(),
        );

        let path = path.strip_prefix("/").unwrap_or(&path);

        let url = if hash {
            let asset_path = self.out_path.join(path);

            if !asset_path.is_file() {
                panic!(
                    "no asset file found at {:?}, required by get_url function",
                    asset_path
                );
            }

            let mut hasher = Sha256::new();
            hasher.update(fs::read(asset_path).expect("asset file"));
            let hash = hasher.finalize();

            format!(
                "{}/{}?h={:x}",
                self.config.site.base_url,
                path.to_string_lossy(),
                hash
            )
        } else {
            if path.extension().is_some() {
                format!("{}/{}", self.config.site.base_url, path.to_string_lossy())
            } else {
                format!("{}/{}/", self.config.site.base_url, path.to_string_lossy())
            }
        };

        Ok(tera::Value::String(url))
    }
}
