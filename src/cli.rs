use crate::{
    console::ColorMode,
    utils::{find_root_dir, leak_str},
};
use anyhow::Result;
use clap::{
    crate_authors, crate_description, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand,
};
use std::path::{Path, PathBuf};

pub fn create_cli() -> App<'static, 'static> {
    // Generate lowercase options for all the color modes
    let color_modes = ColorMode::variants()
        .iter()
        .map(|s| leak_str(s.to_lowercase()))
        .collect::<Vec<&'static str>>();

    // Create the clap cli application
    App::new("nuko")
        .about(crate_description!())
        .author(crate_authors!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("color")
                .long("color")
                .possible_values(&color_modes)
                .case_insensitive(true)
                .default_value("auto")
                .help("Console coloring mode"),
        )
        .arg(
            Arg::with_name("root_dir")
                .long("root-dir")
                .takes_value(true)
                .default_value(".")
                .help("Directory to use as project root dir for the command"),
        )
        .arg(
            Arg::with_name("base_url")
                .long("base-url")
                .takes_value(true)
                .help("Overrides the base url for the output"),
        )
        .subcommands(vec![
            SubCommand::with_name("init")
                .about("Create a new nuko site project")
                .arg(
                    Arg::with_name("path")
                        .default_value(".")
                        .help("Relative path to the directory to use."),
                ),
            SubCommand::with_name("build")
                .about("Builds the nuko site into the project dir")
                .arg(
                    Arg::with_name("out_dir")
                        .long("out-dir")
                        .short("o")
                        .default_value("out")
                        .takes_value(true)
                        .help("Path to the output directory for the build command"),
                ),
        ])
}

// Global config
pub struct CliConfig {
    base_url: Option<String>,
    root_path: PathBuf,
    manifest_path: PathBuf,
}

impl CliConfig {
    pub fn base_url(&self) -> &Option<String> {
        &self.base_url
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }
}

pub fn create_cli_config(matches: &ArgMatches) -> Result<CliConfig> {
    let base_url = matches.value_of("base_url").map(|s| s.into());

    let (root_path, manifest_path) = find_root_dir(matches.value_of("root_dir").unwrap())?;

    Ok(CliConfig {
        base_url,
        root_path,
        manifest_path,
    })
}
