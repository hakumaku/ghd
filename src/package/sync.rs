use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{fs, io};

use super::{Asset, ErrorKind, GithubAPIClient, Package};
use flate2::read::GzDecoder;
use log::info;
use tar::Archive;
use zip::ZipArchive;

/// Move all files in `src` to `dst`, and remove `src` directory.
fn rename_dir_all<P: AsRef<Path>>(src: P, dst: P) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(&src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            rename_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::rename(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }

    fs::remove_dir_all(&src)
}

/// Extract a downloaded file.
fn extract<P: AsRef<Path>>(path: P) -> Result<PathBuf, ErrorKind> {
    let file = File::open(&path)?;
    let file_stem = path.as_ref().file_stem().unwrap();
    let dest = path
        .as_ref()
        .parent()
        .unwrap()
        .to_path_buf()
        .join(file_stem);

    let extension = path.as_ref().extension().unwrap();
    match extension.to_str() {
        Some("zip") => {
            info!("extracting .zip to '{}'", dest.display());
            let mut archive = ZipArchive::new(file)?;
            archive.extract(&dest)?;
        }
        Some("gz") => {
            info!("extracting .tar.gz to '{}'", dest.display());
            let tar = GzDecoder::new(file);
            let mut archive = Archive::new(tar);
            archive.unpack(&dest)?;
        }
        _ => panic!("unsupported extension"),
    };

    let num_files = fs::read_dir(&dest)?.count();
    if num_files == 1 {
        let dir = fs::read_dir(&dest)?.next().unwrap()?.path();
        rename_dir_all(&dir, &dest)?;
    }

    Ok(dest)
}

#[derive(Debug)]
pub struct Downloader {
    api: GithubAPIClient,
    download_path: PathBuf,
    bin_path: PathBuf,
}

impl Downloader {
    pub fn new(github_pat: &String, download_path: &Path, bin_path: &Path) -> Self {
        Self {
            api: GithubAPIClient::new(&github_pat),
            download_path: download_path.to_path_buf(),
            bin_path: bin_path.to_path_buf(),
        }
    }

    /// Iterate files in the directory,
    /// and copy all executable files to the destination.
    fn copy_exec_all<P: AsRef<Path>>(&self, src: P) -> Result<(), ErrorKind> {
        fs::read_dir(&src)?
            .map(|entry| entry.unwrap())
            .filter(|entry| {
                let metadata = entry.metadata().unwrap();
                metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
            })
            .for_each(|entry| {
                let exec = entry.path();
                let filename = exec.file_name().unwrap();
                info!("copying '{}'", filename.to_str().unwrap());
                fs::copy(&exec, &self.bin_path.join(filename)).unwrap();
            });

        Ok(())
    }

    fn download_asset(&self, asset: &Asset) -> Result<PathBuf, ErrorKind> {
        let dest = self.download_path.join(&asset.name);

        self.api.download_asset(asset, dest)
    }

    fn check_release(&self, package: &Package) -> Result<Asset, ErrorKind> {
        self.api
            .get_the_latest_release(&package.user, &package.repo)?
            .into_iter()
            .filter(|asset| asset.name.ends_with(&package.name))
            .next()
            .ok_or(ErrorKind::NoMatchingPattern(package.name.clone()))
    }

    pub fn sync(&self, package: &Package) -> Result<(), ErrorKind> {
        self.check_release(package)
            .and_then(|asset| self.download_asset(&asset))
            .and_then(extract)
            .and_then(|path| self.copy_exec_all(&path))
    }
}
