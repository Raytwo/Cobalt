use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use rawzip::{CompressionMethod, ZipArchiveEntryWayfinder};
use semver::Version;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdaterError {
    #[error("github request failed: {0}")]
    Http(#[from] minreq::Error),
    #[error("github returned status {0}")]
    Status(i32),
    #[error("malformed response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("could not parse version {tag:?}: {source}")]
    BadVersion { tag: String, source: semver::Error },
    #[error("release has no asset named {0:?}")]
    AssetNotFound(&'static str),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("zip: {0}")]
    Zip(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionDifference {
    Regular,
    ChangeToBeta,
    ChangeToStable,
}

pub struct AvailableUpdate {
    pub version: String,
    pub title: String,
    pub published: String,
    pub changelog: String,
    pub kind: VersionDifference,
    asset_url: String,
}

pub fn check(current_version: &str) -> Result<Option<AvailableUpdate>, UpdaterError> {
    let releases = fetch_releases()?;
    let Some(release) = pick_candidate(releases) else {
        return Ok(None);
    };

    let current = parse_version(current_version)?;
    let target = parse_version(&release.tag_name)?;

    let Some(kind) = classify(&current, &target) else {
        return Ok(None);
    };

    let asset = release
        .assets
        .into_iter()
        .find(|a| a.name == "release.zip")
        .ok_or(UpdaterError::AssetNotFound("release.zip"))?;

    Ok(Some(AvailableUpdate {
        version: target.to_string(),
        title: release.name,
        published: format_date(&release.published_at),
        changelog: release.body,
        kind,
        asset_url: asset.browser_download_url,
    }))
}

pub fn apply(update: &AvailableUpdate) -> Result<(), UpdaterError> {
    let bytes = download(&update.asset_url)?;
    extract_archive(&bytes, Path::new("sd:/"))?;
    unsafe { skyline::nn::oe::RequestToRelaunchApplication() }
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    published_at: String,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

fn fetch_releases() -> Result<Vec<GhRelease>, UpdaterError> {
    let resp = minreq::get("https://api.github.com/repos/Raytwo/Cobalt/releases")
        .with_header("User-Agent", concat!("Cobalt-Updater/", env!("CARGO_PKG_VERSION")))
        .with_header("Accept", "application/vnd.github+json")
        .send()?;

    if !(200..300).contains(&resp.status_code) {
        return Err(UpdaterError::Status(resp.status_code));
    }

    Ok(serde_json::from_slice(resp.as_bytes())?)
}

fn download(url: &str) -> Result<Vec<u8>, UpdaterError> {
    let resp = minreq::get(url)
        .with_header("User-Agent", concat!("Cobalt-Updater/", env!("CARGO_PKG_VERSION")))
        .send()?;

    if !(200..300).contains(&resp.status_code) {
        return Err(UpdaterError::Status(resp.status_code));
    }

    Ok(resp.into_bytes())
}

fn pick_candidate(releases: Vec<GhRelease>) -> Option<GhRelease> {
    let mut latest_stable: Option<GhRelease> = None;
    let mut latest_pre: Option<GhRelease> = None;

    for r in releases {
        if r.prerelease {
            if latest_pre.is_none() {
                latest_pre = Some(r);
            }
        } else if latest_stable.is_none() {
            latest_stable = Some(r);
        }
        if latest_stable.is_some() && latest_pre.is_some() {
            break;
        }
    }

    match (latest_stable, latest_pre) {
        (Some(s), Some(p)) => {
            if parse_version(&p.tag_name).ok() > parse_version(&s.tag_name).ok() {
                Some(p)
            } else {
                Some(s)
            }
        },
        (s, p) => s.or(p),
    }
}

fn parse_version(raw: &str) -> Result<Version, UpdaterError> {
    let tag = raw.trim_start_matches('v');
    Version::parse(tag).map_err(|source| UpdaterError::BadVersion {
        tag: tag.to_string(),
        source,
    })
}

fn classify(current: &Version, target: &Version) -> Option<VersionDifference> {
    if target <= current {
        return None;
    }

    let current_is_pre = !current.pre.is_empty();
    let target_is_pre = !target.pre.is_empty();

    Some(match (current_is_pre, target_is_pre) {
        (false, true) => VersionDifference::ChangeToBeta,
        (true, false) => VersionDifference::ChangeToStable,
        _ => VersionDifference::Regular,
    })
}

fn format_date(published_at: &str) -> String {
    let mut parts = published_at.split('-');
    let (Some(year), Some(month), Some(day_rest)) = (parts.next(), parts.next(), parts.next()) else {
        return String::new();
    };
    let day = day_rest.get(..2).unwrap_or("");
    format!("{month}/{day}/{year}")
}

fn extract_archive(data: &[u8], dest_root: &Path) -> Result<(), UpdaterError> {
    let archive = rawzip::ZipArchive::from_slice(data).map_err(|e| UpdaterError::Zip(format!("invalid zip data: {e:?}")))?;

    struct Plan {
        dst: PathBuf,
        is_dir: bool,
        wayfinder: ZipArchiveEntryWayfinder,
        compression: CompressionMethod,
    }

    let mut plan: Vec<Plan> = Vec::new();
    let mut iter = archive.entries();
    while let Some(entry) = iter
        .next_entry()
        .map_err(|e| UpdaterError::Zip(format!("failed to iterate entries: {e:?}")))?
    {
        let normalized = entry
            .file_path()
            .try_normalize()
            .map_err(|e| UpdaterError::Zip(format!("malformed entry path: {e:?}")))?;

        plan.push(Plan {
            dst: dest_root.join(normalized.as_ref()),
            is_dir: entry.is_dir(),
            wayfinder: entry.wayfinder(),
            compression: entry.compression_method(),
        });
    }

    for item in plan {
        if item.is_dir {
            std::fs::create_dir_all(&item.dst)?;
            continue;
        }

        if let Some(parent) = item.dst.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let local = archive
            .get_entry(item.wayfinder)
            .map_err(|e| UpdaterError::Zip(format!("get_entry {}: {e:?}", item.dst.display())))?;

        let mut file = File::create(&item.dst)?;

        match item.compression {
            CompressionMethod::Store => {
                let mut reader = local.verifying_reader(local.data());
                io::copy(&mut reader, &mut file)?;
            },
            CompressionMethod::Deflate => {
                let decoder = flate2::read::DeflateDecoder::new(local.data());
                let mut reader = local.verifying_reader(decoder);
                io::copy(&mut reader, &mut file)?;
            },
            other => {
                return Err(UpdaterError::Zip(format!(
                    "{}: unsupported compression method {other:?}",
                    item.dst.display(),
                )));
            },
        }

        file.flush()?;
    }

    Ok(())
}
