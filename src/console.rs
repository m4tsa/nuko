use clap::arg_enum;
use std::{
    env,
    sync::atomic::{AtomicBool, Ordering},
};

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum ColorMode {
        Auto,
        Always,
        Never
    }
}

static COLORS_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_color_mode(mode: ColorMode) {
    let enabled = match mode {
        ColorMode::Auto => {
            let cli_colors = env::var("CLICOLOR").unwrap_or_else(|_| "1".into()) != "0".to_string();
            let cli_colors_force =
                env::var("CLICOLOR_FORCE").unwrap_or_else(|_| "0".into()) != "0".to_string();
            let no_color = env::var("NO_COLOR").is_ok();

            cli_colors_force || (cli_colors && !no_color && atty::is(atty::Stream::Stdout))
        }
        ColorMode::Always => true,
        ColorMode::Never => false,
    };

    COLORS_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn has_color() -> bool {
    COLORS_ENABLED.load(Ordering::Relaxed)
}
