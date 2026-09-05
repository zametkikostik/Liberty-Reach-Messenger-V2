//! Secure file wiping (best-effort anti-forensics)

use std::fs::{self, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use std::path::Path;
use rand::RngCore;

/// Overwrite file with random data multiple times, then delete.
/// On modern SSDs absolute guarantees need full-disk encryption.
pub fn secure_wipe_file(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::metadata(path)?;
    let len = metadata.len() as usize;
    if len == 0 {
        fs::remove_file(path)?;
        return Ok(());
    }
    let mut file = OpenOptions::new().write(true).open(path)?;
    let mut buffer = vec![0u8; len.min(1024 * 1024)];
    for _ in 0..3 {
        file.seek(SeekFrom::Start(0))?;
        let mut remaining = len;
        while remaining > 0 {
            let chunk = remaining.min(buffer.len());
            rand::thread_rng().fill_bytes(&mut buffer[..chunk]);
            file.write_all(&buffer[..chunk])?;
            remaining -= chunk;
        }
        file.sync_all()?;
    }
    file.seek(SeekFrom::Start(0))?;
    let zeros = vec![0u8; buffer.len()];
    let mut remaining = len;
    while remaining > 0 {
        let chunk = remaining.min(zeros.len());
        file.write_all(&zeros[..chunk])?;
        remaining -= chunk;
    }
    file.sync_all()?;
    drop(file);
    fs::remove_file(path)?;
    Ok(())
}
