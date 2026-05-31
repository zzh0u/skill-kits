use crate::{cli, core::paths::AppPaths, gui};
use clap::Parser;

pub fn run() -> anyhow::Result<()> {
    let cli = cli::args::Cli::parse();
    if cli.gui {
        return gui::run_native(AppPaths::default_user_paths()?);
    }

    cli::handlers::run_parsed_cli(cli)
}
