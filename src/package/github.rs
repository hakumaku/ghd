use std::fmt::Display;
use std::fs;
use std::io::Read;
use std::ops::Deref;
use std::path::Path;

use log::info;
use serde::Deserialize;
use ureq::{Agent, Response};

use super::ErrorKind;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Author {
    id: u32,
    login: String,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    #[serde(rename = "type")]
    t: String,
    site_admin: bool,
}

impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Author '{}'>", self.id)
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Asset {
    id: u32,
    url: String,
    browser_download_url: String,
    node_id: String,
    pub name: String,
    label: Option<String>,
    state: String,
    content_type: String,
    size: u64,
    download_count: u64,
    created_at: String,
    updated_at: String,
    uploader: Author,
}

impl Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Asset '{}'>", self.name)
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Release {
    url: String,
    html_url: String,
    assets_url: String,
    upload_url: String,
    tarball_url: Option<String>,
    zipball_url: Option<String>,
    id: u32,
    node_id: String,
    tag_name: String,
    target_commitish: String,
    name: Option<String>,
    body: Option<String>,
    draft: bool,
    prerelease: bool,
    created_at: String,
    published_at: String,
    author: Author,
    pub assets: Vec<Asset>,
}

impl IntoIterator for Release {
    type Item = Asset;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.assets.into_iter()
    }
}

impl Deref for Release {
    type Target = Vec<Asset>;

    fn deref(&self) -> &Self::Target {
        &self.assets
    }
}

impl Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Release '{}' ({})>", self.tag_name, self.published_at)
    }
}

#[derive(Debug)]
pub struct GithubAPIClient {
    base_url: String,
    bearer: String,
    client: Agent,
}

impl GithubAPIClient {
    pub fn new(access_token: &str) -> Self {
        Self {
            base_url: "https://api.github.com".to_owned(),
            bearer: format!("Bearer {}", access_token),
            client: Agent::new(),
        }
    }

    fn get(&self, path: &str) -> Result<Response, ErrorKind> {
        let url = format!("{}{}", self.base_url, path);
        Ok(self
            .client
            .get(&url)
            .set("Accept", "application/vnd.github+json")
            .set("Authorization", &self.bearer)
            .call()?)
    }

    fn download(&self, url: &str) -> Result<Response, ErrorKind> {
        Ok(self
            .client
            .get(url)
            .set("Accept", "application/octet-stream")
            .set("Authorization", &self.bearer)
            .call()?)
    }

    pub fn get_the_latest_release(&self, user: &str, repo: &str) -> Result<Release, ErrorKind> {
        info!("GET '{}/{}'", user, repo);

        let path = format!("/repos/{}/{}/releases/latest", user, repo);
        Ok(self.get(&path)?.into_json()?)
    }

    pub fn download_asset<P: AsRef<Path>>(&self, asset: &Asset, dest: P) -> Result<P, ErrorKind> {
        let asset_name = &asset.name;
        let updated_at = &asset.updated_at;
        info!("GET '{} ({})'", asset_name, updated_at);

        let response = self.download(&asset.browser_download_url)?;
        let length: usize = response.header("Content-Length").unwrap().parse().unwrap();
        let mut bytes: Vec<u8> = Vec::with_capacity(length);
        response
            .into_reader()
            .take(10_000_000)
            .read_to_end(&mut bytes)?;
        fs::write(&dest, bytes)?;

        Ok(dest)
    }
}
