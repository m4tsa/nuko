use std::str::FromStr;

mod cli;
mod cmd;
mod console;
mod utils;

fn main() {
    let matches = cli::create_cli().get_matches();

    if let Some(color_mode_str) = matches.value_of("color") {
        console::set_color_mode(console::ColorMode::from_str(color_mode_str).unwrap());
    }

    match matches.subcommand() {
        ("build", Some(_sub_matches)) => {
            cmd::cmd_build();
        }
        ("init", Some(sub_matches)) => {
            cmd::cmd_init(sub_matches.value_of("path").unwrap());
        }
        _ => unreachable!(),
    }
}
