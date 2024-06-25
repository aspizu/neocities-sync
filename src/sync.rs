use std::{
    cell::RefCell,
    io::Write,
    path::{Path, PathBuf},
};

use futures::{future::try_join_all, TryFutureExt};
use fxhash::FxHashMap;
use glob::glob;
use sha1::{Digest, Sha1};
use tokio::{fs, try_join};

use crate::{
    neocities::{DeleteError, ListError, Neocities, UploadError},
    state::{fetch_state, read_state_file, write_state_file},
};

#[rustfmt::skip]
const ALLOWED_FILE_TYPES: &[&str] = &[
    "apng", "asc", "atom", "avif", "bin", "cjs", "css", "csv", "dae", "eot", "epub",
    "geojson", "gif", "glb", "gltf", "gpg", "htm", "html", "ico", "jpeg", "jpg", "js",
    "json", "key", "kml", "knowl", "less", "manifest", "map", "markdown", "md", "mf",
    "mid", "midi", "mjs", "mtl", "obj", "opml", "osdx", "otf", "pdf", "pgp", "pls",
    "png", "py", "rdf", "resolveHandle", "rss", "sass", "scss", "svg", "text", "toml",
    "ts", "tsv", "ttf", "txt", "webapp", "webmanifest", "webp", "woff", "woff2", "xcf",
    "xml", "yaml", "yml"
];

#[derive(Debug)]
pub enum SyncError {
    InvalidAuth,
    InvalidFileType,
    MissingFiles,
    ReqwestError(reqwest::Error),
    IOError(std::io::Error),
}

impl From<ListError> for SyncError {
    fn from(error: ListError) -> Self {
        match error {
            ListError::InvalidAuth => Self::InvalidAuth,
            ListError::ReqwestError(error) => Self::ReqwestError(error),
        }
    }
}

impl From<UploadError> for SyncError {
    fn from(error: UploadError) -> Self {
        match error {
            UploadError::InvalidFileType => Self::InvalidFileType,
            UploadError::InvalidAuth => Self::InvalidAuth,
            UploadError::ReqwestError(error) => Self::ReqwestError(error),
        }
    }
}

impl From<DeleteError> for SyncError {
    fn from(error: DeleteError) -> Self {
        match error {
            DeleteError::MissingFiles => Self::MissingFiles,
            DeleteError::InvalidAuth => Self::InvalidAuth,
            DeleteError::ReqwestError(error) => Self::ReqwestError(error),
        }
    }
}

impl From<reqwest::Error> for SyncError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error)
    }
}

impl From<std::io::Error> for SyncError {
    fn from(error: std::io::Error) -> Self {
        Self::IOError(error)
    }
}

async fn process(
    path: impl AsRef<Path>,
    current_state: &FxHashMap<String, String>,
    new_state: &RefCell<FxHashMap<String, String>>,
    state_path_relative_to_path: &Option<PathBuf>,
    to_be_uploaded: &RefCell<Vec<(String, Vec<u8>)>>,
    subpath: PathBuf,
) -> Result<(), SyncError> {
    if state_path_relative_to_path.as_deref().is_some_and(|it| it == subpath) {
        return Ok(());
    }
    if subpath.is_dir() {
        return Ok(());
    }
    let file = fs::read(&subpath).await?;
    let subpath = pathdiff::diff_paths(&subpath, path.as_ref()).unwrap();
    let mut hasher = Sha1::new();
    hasher.write_all(&file)?;
    let new_hash = format!("{:x}", hasher.finalize());
    let old_hash = current_state.get(subpath.to_str().unwrap());
    let is_modified = Some(&new_hash) != old_hash;
    let new_state = &mut *new_state.borrow_mut();
    new_state.insert(subpath.to_str().unwrap().to_string(), new_hash);
    if is_modified {
        let to_be_uploaded = &mut *to_be_uploaded.borrow_mut();
        to_be_uploaded.push((subpath.to_str().unwrap().to_string(), file));
    }
    Ok(())
}

pub struct SyncStats {
    pub uploaded: usize,
    pub deleted: usize,
}

pub async fn sync(
    neocities: &Neocities,
    path: impl AsRef<Path>,
    state_path: impl AsRef<Path>,
    ignore_disllowed_file_types: bool,
) -> Result<SyncStats, SyncError> {
    let current_state = if let Ok(state) = read_state_file(&state_path).await {
        state
    } else {
        fetch_state(neocities).await?
    };
    let new_state: RefCell<FxHashMap<String, String>> = Default::default();
    let state_path_relative_to_path = pathdiff::diff_paths(&state_path, &path);
    let to_be_uploaded: RefCell<Vec<(String, Vec<u8>)>> = Default::default();
    let paths = glob(path.as_ref().join("**/*").to_str().unwrap()).unwrap();
    let mut futs = vec![];
    for subpath in paths {
        let subpath = subpath.map_err(|err| err.into_error())?;
        if ignore_disllowed_file_types
            && !subpath
                .extension()
                .is_some_and(|it| ALLOWED_FILE_TYPES.contains(&it.to_str().unwrap()))
        {
            continue;
        }
        futs.push(process(
            &path,
            &current_state,
            &new_state,
            &state_path_relative_to_path,
            &to_be_uploaded,
            subpath,
        ));
    }
    try_join_all(futs).await?;
    let new_state = new_state.into_inner();
    let to_be_uploaded = to_be_uploaded.into_inner();
    let to_be_deleted = current_state
        .keys()
        .filter(|&subpath| !new_state.contains_key(subpath))
        .cloned()
        .collect::<Vec<_>>();
    let stats =
        SyncStats { uploaded: to_be_uploaded.len(), deleted: to_be_deleted.len() };
    let upload_fut = neocities.upload(to_be_uploaded).err_into::<SyncError>();
    let delete_fut = neocities.delete(to_be_deleted).err_into::<SyncError>();
    try_join!(upload_fut, delete_fut)?;
    write_state_file(&new_state, state_path).await?;
    Ok(stats)
}
