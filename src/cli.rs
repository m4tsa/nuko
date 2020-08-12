use crate::{
    console::ColorMode,
    utils::{find_root_dir, leak_str},
};
use anyhow::Result;
use clap::{
    crate_authors, crate_description, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand,
};
use config::project::ProjectConfig;
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
    root_path: PathBuf,
    project_config: ProjectConfig,
}

impl CliConfig {
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn project_config(&self) -> &ProjectConfig {
        &self.project_config
    }
}

pub fn create_cli_config(matches: &ArgMatches) -> Result<CliConfig> {
    let (root_path, manifest_path) = find_root_dir(matches.value_of("root_dir").unwrap())?;

    let project_config = ProjectConfig::read_file(&manifest_path)?;

    Ok(CliConfig {
        root_path,
        project_config,
    })
}
