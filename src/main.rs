#[macro_use]
extern crate lazy_static;

use anyhow::Result;
use std::{net::SocketAddr, str::FromStr};

#[macro_use]
mod utils;

mod cli;
mod cmd;
mod console;

fn main() {
    match run_cli() {
        Ok(()) => (),
        Err(err) => panic!("{:?}", err),
    }
}

fn run_cli() -> Result<()> {
    let matches = cli::create_cli().get_matches();

    if let Some(color_mode_str) = matches.value_of("color") {
        console::set_color_mode(console::ColorMode::from_str(color_mode_str).unwrap());
    }

    match matches.subcommand() {
        ("build", Some(sub_matches)) => {
            let cli_config = cli::create_cli_config(&matches)?;
            let out_path = cli_config
                .root_path()
                .join(sub_matches.value_of("out_dir").unwrap());

            cmd::cmd_build(cli_config, out_path)?;
        }
        ("init", Some(sub_matches)) => {
            cmd::cmd_init(sub_matches.value_of("path").unwrap());
        }
        ("serve", Some(sub_matches)) => {
            let cli_config = cli::create_cli_config(&matches)?;
            let socket_addr = SocketAddr::new(
                sub_matches.value_of("host").unwrap().parse()?,
                sub_matches.value_of("port").unwrap().parse()?,
            );
            let out_path = cli_config
                .root_path()
                .join(sub_matches.value_of("out_dir").unwrap());

            cmd::cmd_serve(cli_config, socket_addr, out_path)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
