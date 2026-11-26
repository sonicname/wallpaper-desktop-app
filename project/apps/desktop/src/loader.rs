use std::{
    fs, io,
    path::Path,
    sync::{Arc, Condvar, Mutex, Weak},
    thread,
    time::UNIX_EPOCH,
};

use wallpaper_core::{Item, ProviderError, Rgb, Size, local_path};
use wallpaper_host::{DiskCache, HostHttp, Pixels, cover, dimensions, thumbnail};

const WORKERS: usize = 6;
const MAX_FILE: u64 = 100 << 20;
/// The longer side of the small copy of a file on this computer, enough for a cell on a screen
/// scaled to 150%. Pictures that are not larger are shown as they are.
const THUMB_SIDE: u32 = 640;

pub struct Job {
    /// The job is skipped once the cell it was meant for is gone.
    pub wanted: Weak<()>,
    pub url: String,
    pub target: Size,
    pub radius: f32,
    pub background: Rgb,
    pub done: Box<dyn FnOnce(Pixels) + Send>,
}

/// Loads thumbnails on a few threads, newest request first: what just scrolled into view
/// matters more than what already left it.
pub struct Loader {
    shared: Arc<Shared>,
}

struct Shared {
    queue: Mutex<Queue>,
    ready: Condvar,
    http: Arc<HostHttp>,
    thumbs: Option<DiskCache>,
    originals: Option<DiskCache>,
    /// Decoding a large file takes a lot of memory, so the workers take turns.
    large: Mutex<()>,
}

#[derive(Default)]
struct Queue {
    jobs: Vec<Job>,
    closed: bool,
}

impl Loader {
    pub fn new(
        http: Arc<HostHttp>,
        thumbs: Option<DiskCache>,
        originals: Option<DiskCache>,
    ) -> Self {
        let shared = Arc::new(Shared {
            queue: Mutex::default(),
            ready: Condvar::new(),
            http,
            thumbs,
            originals,
            large: Mutex::default(),
        });
        for _ in 0..WORKERS {
            let shared = shared.clone();
            thread::spawn(move || shared.work());
        }
        Self { shared }
    }

    pub fn push(&self, job: Job) {
        self.shared.queue.lock().unwrap().jobs.push(job);
        self.shared.ready.notify_one();
    }

    /// The bytes of a thumbnail, from disk when possible.
    pub fn thumb(&self, url: &str) -> Option<Vec<u8>> {
        self.shared.bytes(url, true)
    }

    /// The bytes of the full picture, from disk when possible.
    pub fn original(&self, item: &Item) -> Result<Vec<u8>, ProviderError> {
        let bytes = self.fresh_original(item)?;
        if let Some(cache) = &self.shared.originals
            && item.local_file().is_none()
        {
            let _ = cache.put(&item.image_url, &bytes);
        }
        Ok(bytes)
    }

    /// Like [`Self::original`], but leaves nothing behind on disk.
    pub fn fresh_original(&self, item: &Item) -> Result<Vec<u8>, ProviderError> {
        if let Some(file) = item.local_file() {
            return read_file(file);
        }
        let cache = self.shared.originals.as_ref();
        if let Some(bytes) = cache.and_then(|c| c.get(&item.image_url)) {
            return Ok(bytes);
        }
        let bytes = self.shared.http.image(&item.image_url)?;
        // A cut off JPEG still decodes, with a grey lower part.
        if (bytes.len() as u64) < item.file_bytes {
            return Err(ProviderError::BadResponse("incomplete picture".into()));
        }
        Ok(bytes)
    }

    pub fn share(&self) -> Self {
        Self {
            shared: self.shared.clone(),
        }
    }
}

impl Drop for Loader {
    fn drop(&mut self) {
        // Workers hold one reference each; the last outside handle stops them.
        if Arc::strong_count(&self.shared) <= WORKERS + 1 {
            self.shared.queue.lock().unwrap().closed = true;
            self.shared.ready.notify_all();
        }
    }
}

impl Shared {
    fn work(&self) {
        while let Some(job) = self.next() {
            if job.wanted.strong_count() == 0 {
                continue;
            }
            if let Some(pixels) = self.load(&job) {
                (job.done)(pixels);
            }
        }
    }

    fn next(&self) -> Option<Job> {
        let mut queue = self.queue.lock().unwrap();
        loop {
            if queue.closed {
                return None;
            }
            if let Some(job) = queue.jobs.pop() {
                return Some(job);
            }
            queue = self.ready.wait(queue).unwrap();
        }
    }

    fn load(&self, job: &Job) -> Option<Pixels> {
        let decode = |bytes: Vec<u8>| cover(&bytes, job.target, job.radius, job.background).ok();
        // A damaged cache file must not hide the picture forever, so fetch it again.
        self.bytes(&job.url, true)
            .and_then(decode)
            .or_else(|| self.bytes(&job.url, false).and_then(decode))
    }

    fn bytes(&self, url: &str, use_cache: bool) -> Option<Vec<u8>> {
        if let Some(file) = local_path(url) {
            return self.small_copy(file, use_cache);
        }
        if use_cache && let Some(bytes) = self.thumbs.as_ref().and_then(|c| c.get(url)) {
            return Some(bytes);
        }
        let bytes = self.http.image(url).ok()?;
        if let Some(cache) = &self.thumbs {
            // A full disk only costs speed.
            let _ = cache.put(url, &bytes);
        }
        Some(bytes)
    }

    /// A file on this computer, shrunk if the picture is large. The copy is kept until the file
    /// changes, because decoding a large picture for every cell would be slow.
    fn small_copy(&self, file: &Path, use_cache: bool) -> Option<Vec<u8>> {
        let meta = fs::metadata(file).ok()?;
        let changed = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
        let key = format!(
            "file:{}|{}|{}",
            file.display(),
            meta.len(),
            changed.as_secs()
        );
        let cache = self.thumbs.as_ref();
        if use_cache && let Some(bytes) = cache.and_then(|c| c.get(&key)) {
            return Some(bytes);
        }
        let bytes = read_file(file).ok()?;
        let size = dimensions(&bytes)?;
        if size.width.max(size.height) <= THUMB_SIDE {
            return Some(bytes);
        }
        let small = {
            let _turn = self
                .large
                .lock()
                .unwrap_or_else(|broken| broken.into_inner());
            thumbnail(&bytes, THUMB_SIDE).ok()?
        };
        if let Some(cache) = cache {
            // A full disk only costs speed.
            let _ = cache.put(&key, &small);
        }
        Some(small)
    }
}

fn read_file(file: &Path) -> Result<Vec<u8>, ProviderError> {
    let missing = |error: io::Error| match error.kind() {
        io::ErrorKind::NotFound => ProviderError::NotFound,
        _ => ProviderError::BadResponse(error.to_string()),
    };
    if fs::metadata(file).map_err(missing)?.len() > MAX_FILE {
        return Err(ProviderError::BadResponse("the file is too large".into()));
    }
    fs::read(file).map_err(missing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use wallpaper_core::RateLimit;
    use wallpaper_host::dimensions;

    fn item(file: &Path) -> Item {
        let location = file.to_string_lossy().into_owned();
        Item {
            provider: "folder".into(),
            id: location.clone(),
            size: Size::new(1, 1),
            image_url: location,
            page_url: String::new(),
            file_bytes: u64::MAX,
            mime: String::new(),
            colors: vec![],
            thumbs: vec![],
        }
    }

    /// Noise does not compress, so the file is large.
    fn noise(path: &Path, width: u32, height: u32) {
        let mut seed = 1_u32;
        let picture = image::RgbImage::from_fn(width, height, |_, _| {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            image::Rgb([(seed >> 16) as u8, (seed >> 8) as u8, (seed >> 24) as u8])
        });
        picture.save(path).unwrap();
    }

    #[test]
    fn files_on_this_computer_need_no_network() {
        let dir: PathBuf =
            std::env::temp_dir().join(format!("wallpaper-loader-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("pictures")).unwrap();
        let (large, small) = (
            dir.join("pictures/large.png"),
            dir.join("pictures/small.png"),
        );
        noise(&large, 1600, 900);
        noise(&small, 64, 36);
        // Few bytes, but many pixels: what counts is the size of the picture.
        let flat = dir.join("pictures/flat.png");
        image::RgbImage::new(2000, 1000).save(&flat).unwrap();
        assert!(fs::metadata(&flat).unwrap().len() < 100_000);

        let limit = RateLimit {
            requests: 1,
            per: std::time::Duration::from_secs(1),
        };
        let cache = DiskCache::open(dir.join("thumbs")).unwrap();
        let loader = Loader::new(Arc::new(HostHttp::new(limit)), Some(cache), None);
        let cached = || fs::read_dir(dir.join("thumbs")).unwrap().count();

        // A small file is used as it is.
        let bytes = loader.thumb(small.to_str().unwrap()).unwrap();
        assert_eq!(bytes, fs::read(&small).unwrap());
        assert_eq!(cached(), 0);

        // A large one gets a small copy, made once.
        let copy = loader.thumb(large.to_str().unwrap()).unwrap();
        assert_eq!(dimensions(&copy), Some(Size::new(640, 360)));
        // Noise is the worst case for JPEG, and still far smaller than the file.
        assert!(copy.len() < 400_000, "{}", copy.len());
        assert_eq!(cached(), 1);
        assert_eq!(loader.thumb(large.to_str().unwrap()).unwrap(), copy);
        assert_eq!(cached(), 1);

        let copy = loader.thumb(flat.to_str().unwrap()).unwrap();
        assert_eq!(dimensions(&copy), Some(Size::new(640, 320)));
        assert_eq!(cached(), 2);

        // A changed file gets a new copy.
        noise(&large, 900, 1600);
        let copy = loader.thumb(large.to_str().unwrap()).unwrap();
        assert_eq!(dimensions(&copy), Some(Size::new(360, 640)));

        // The full picture is read from the file, whatever size the item claims.
        assert_eq!(
            loader.original(&item(&small)).unwrap(),
            fs::read(&small).unwrap()
        );
        fs::remove_file(&small).unwrap();
        assert_eq!(
            loader.fresh_original(&item(&small)),
            Err(ProviderError::NotFound)
        );
        assert_eq!(loader.thumb(small.to_str().unwrap()), None);
        let _ = fs::remove_dir_all(&dir);
    }
}
