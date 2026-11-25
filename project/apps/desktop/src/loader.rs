use std::{
    sync::{Arc, Condvar, Mutex, Weak},
    thread,
};

use wallpaper_core::{Item, ProviderError, Rgb, Size};
use wallpaper_host::{DiskCache, HostHttp, Pixels, cover};

const WORKERS: usize = 6;

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
        if let Some(cache) = &self.shared.originals {
            let _ = cache.put(&item.image_url, &bytes);
        }
        Ok(bytes)
    }

    /// Like [`Self::original`], but leaves nothing behind on disk.
    pub fn fresh_original(&self, item: &Item) -> Result<Vec<u8>, ProviderError> {
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
}
