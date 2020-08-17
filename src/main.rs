use anyhow::Result;
use std::str::FromStr;

#[macro_use]
mod utils;

mod cli;
mod cmd;
mod console;

fn main() {
    match run_cli() {
        Ok(()) => (),
        Err(err) => panic!("{}", err),
    }
}

fn run_cli() -> Result<()> {
    let matches = cli::create_cli().get_matches();

    if let Some(color_mode_str) = matches.value_of("color") {
        console::set_color_mode(console::ColorMode::from_str(color_mode_str).unwrap());
    }

    match matches.subcommand() {
        ("build", Some(_sub_matches)) => {
            let cli_config = cli::create_cli_config(&matches)?;

            cmd::cmd_build(cli_config)?;
        }
        ("init", Some(sub_matches)) => {
            cmd::cmd_init(sub_matches.value_of("path").unwrap());
        }
        _ => unreachable!(),
    }

    Ok(())
}
