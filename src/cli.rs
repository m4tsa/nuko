use crate::{console::ColorMode, utils::leak_str};
use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg, SubCommand};

pub fn create_cli() -> App<'static, 'static> {
    // Generate lowercase options for all the color modes
    let color_modes = ColorMode::variants()
        .iter()
        .map(|s| leak_str(s.to_lowercase()))
        .collect::<Vec<&'static str>>();

    //
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
        .subcommands(vec![
            SubCommand::with_name("init")
                .about("Create a new nuko site")
                .arg(
                    Arg::with_name("path")
                        .default_value(".")
                        .help("Relative path to the directory to use."),
                ),
            SubCommand::with_name("build")
                .about("Builds the nuko site into the project dir")
                .arg(
                    Arg::with_name("output_dir")
                        .long("output-dir")
                        .short("o")
                        .default_value("out")
                        .takes_value(true)
                        .help("Path to the output directory for the build command"),
                ),
        ])
}
