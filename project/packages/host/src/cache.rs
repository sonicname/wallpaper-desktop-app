use std::{
    fs, io,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU32, Ordering},
};

/// Files keyed by URL in one flat folder.
pub struct DiskCache {
    dir: PathBuf,
}

impl DiskCache {
    pub fn open(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    pub fn get(&self, url: &str) -> Option<Vec<u8>> {
        fs::read(self.path(url)).ok()
    }

    /// Writes to a temporary file first so that a crash never leaves half an image behind.
    pub fn put(&self, url: &str, bytes: &[u8]) -> io::Result<()> {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let name = format!(
            "{}-{}.part",
            process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let part = self.dir.join(name);
        fs::write(&part, bytes)?;
        fs::rename(&part, self.path(url)).inspect_err(|_| drop(fs::remove_file(&part)))
    }

    /// Deletes the oldest files until the folder is at most `max_bytes`.
    pub fn trim(&self, max_bytes: u64) -> io::Result<()> {
        let mut files = vec![];
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_file() {
                files.push((meta.modified()?, meta.len(), entry.path()));
            }
        }
        files.sort();
        let mut total: u64 = files.iter().map(|f| f.1).sum();
        for (_, bytes, path) in files {
            if total <= max_bytes {
                break;
            }
            fs::remove_file(path)?;
            total -= bytes;
        }
        Ok(())
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn path(&self, url: &str) -> PathBuf {
        // FNV-1a, 64 bit. Unlike the standard library hasher it is stable between releases.
        let hash = url.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
        });
        self.dir.join(format!("{hash:016x}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_reads_and_trims() {
        let dir = std::env::temp_dir().join(format!("wallpaper-cache-test-{}", process::id()));
        let cache = DiskCache::open(&dir).unwrap();
        assert_eq!(cache.get("https://a"), None);
        cache.put("https://a", &[1; 100]).unwrap();
        cache.put("https://b", &[2; 100]).unwrap();
        assert_eq!(cache.get("https://a").unwrap(), vec![1; 100]);
        cache.trim(150).unwrap();
        assert_eq!(fs::read_dir(cache.dir()).unwrap().count(), 1);
        fs::remove_dir_all(&dir).unwrap();
    }
}
