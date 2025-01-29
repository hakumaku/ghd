use clap::{Parser, Subcommand};
use log::error;

use ghd::{load_config, Downloader};

#[derive(Subcommand, Debug)]
enum Commands {
    Sync {},
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    match args.command {
        Commands::Sync {} => {
            let config = load_config();
            let downloader =
                Downloader::new(&config.github_pat, &config.download_path, &config.bin_path);

            for package in &config.packages {
                let result = downloader.sync(package);
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }
    }
}
