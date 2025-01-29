use std::env;
use std::path::PathBuf;

use config::Config;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Package {
    pub user: String,
    pub repo: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub github_pat: String,
    pub download_path: PathBuf,
    pub bin_path: PathBuf,
    pub packages: Vec<Package>,
}

pub fn load_config() -> AppConfig {
    let home = env::var("HOME").unwrap();
    let download_path = PathBuf::from(&home).join("Downloads").display().to_string();
    let bin_path = PathBuf::from(&home)
        .join(".local")
        .join("bin")
        .display()
        .to_string();

    let path = format!("{}/.config/ghd/config.toml", home);
    let config_file = config::File::with_name(&path);
    Config::builder()
        .set_default("download_path", download_path)
        .unwrap()
        .set_default("bin_path", bin_path)
        .unwrap()
        .add_source(config_file)
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap()
}
