use anyhow::Result;
use std::{fs::OpenOptions, io::Write, path::PathBuf};

pub async fn save_with_cb(
    filename: &PathBuf,
    response: &mut reqwest::Response,
    cb: &dyn Fn(usize) -> (),
) -> Result<()> {
    let mut dest = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;

    while let Some(chunk) = response.chunk().await? {
        dest.write_all(&chunk)?;
        cb(chunk.len());
    }
    Ok(())
}

pub async fn save(filename: &PathBuf, response: &mut reqwest::Response) -> Result<()> {
    let mut dest = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)?;

    while let Some(chunk) = response.chunk().await? {
        dest.write_all(&chunk)?;
    }
    Ok(())
}
