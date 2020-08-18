use crate::cli::CliConfig;
use anyhow::Result;
use nuko_core::{config::SiteConfig, site::Site};
use std::path::PathBuf;

pub fn cmd_build(cli_config: CliConfig, out_path: PathBuf) -> Result<()> {
    let site_config = SiteConfig::read_file(cli_config.manifest_path())?;

    let mut site = Site::new(cli_config.root_path(), site_config, out_path)?;

    site.load_content()?;

    Ok(())
}
