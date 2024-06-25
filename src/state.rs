use std::path::Path;

use fxhash::FxHashMap;
use tokio::{
    fs::File,
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
};

use crate::neocities::{ListError, Neocities};

pub async fn read_state_file(
    path: impl AsRef<Path>,
) -> Result<FxHashMap<String, String>, io::Error> {
    let mut state = FxHashMap::default();
    let mut lines = BufReader::new(File::open(path).await?).lines();
    while let Some(line) = lines.next_line().await? {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        state.insert(key.to_string(), value.to_string());
    }
    Ok(state)
}

pub async fn write_state_file(
    state: &FxHashMap<String, String>,
    path: impl AsRef<Path>,
) -> io::Result<()> {
    let mut file = File::create(path).await?;
    for (key, value) in state {
        file.write_all(key.as_bytes()).await?;
        file.write_all(b":").await?;
        file.write_all(value.as_bytes()).await?;
        file.write_all(b"\n").await?;
    }
    Ok(())
}

pub async fn fetch_state(
    neocities: &Neocities,
) -> Result<FxHashMap<String, String>, ListError> {
    let mut state = FxHashMap::default();
    let response = neocities.list().await?;
    for file in response {
        if let Some(sha1_hash) = file.sha1_hash {
            state.insert(file.path, sha1_hash);
        }
    }
    Ok(state)
}
