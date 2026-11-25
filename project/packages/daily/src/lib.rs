//! One new wallpaper a day: when it is due, which pictures come next, and which never again.

mod schedule;

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use serde::{Deserialize, Serialize};
use wallpaper_core::{
    Category, Http, Item, ListRequest, LocalTime, Provider, ProviderError, Ratio, Rgb, Size, Sort,
    fit,
};
use wallpaper_host::{cover, write_picture, write_whole};

pub use schedule::current_day;

/// Pictures kept ready for every screen size, so that a change needs no network.
pub const READY: usize = 3;
/// Enough candidates to make the choice feel random, without asking for too many pages.
const CANDIDATES: usize = 48;
const MAX_PAGES: usize = 8;
const MAX_SEEN: usize = 3000;
const MAX_SHOWN: usize = 60;
/// Decoding needs three bytes per pixel.
const MAX_PIXELS: u64 = 60_000_000;
/// How much of a picture may be cut away, first choice and last resort.
const CROP: [f32; 2] = [0.12, 0.30];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub enabled: bool,
    pub paused: bool,
    /// Minutes after midnight.
    pub change_at: u32,
    pub sort: Sort,
    /// Empty means every category.
    pub categories: Vec<Category>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            paused: false,
            change_at: 8 * 60,
            sort: Sort::Top,
            categories: vec![],
        }
    }
}

/// A picture that is cropped, saved and waiting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ready {
    pub key: String,
    pub size: Size,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Shown {
    path: PathBuf,
    key: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Data {
    settings: Settings,
    last_day: Option<i64>,
    /// Oldest first.
    seen: Vec<String>,
    blocked: Vec<String>,
    queue: Vec<Ready>,
    /// Which picture is behind a wallpaper file, newest last.
    shown: Vec<Shown>,
}

pub struct Daily {
    dir: PathBuf,
    data: Mutex<Data>,
}

/// Where new pictures come from.
pub struct Source<'a> {
    pub provider: &'a dyn Provider,
    pub http: &'a dyn Http,
    pub download: &'a dyn Fn(&Item) -> Result<Vec<u8>, ProviderError>,
}

impl Daily {
    pub fn open(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(dir.join("queue"))?;
        let bytes = fs::read(dir.join("daily.json")).unwrap_or_default();
        let data = serde_json::from_slice(&bytes).unwrap_or_default();
        Ok(Self {
            dir,
            data: Mutex::new(data),
        })
    }

    pub fn settings(&self) -> Settings {
        self.lock().settings.clone()
    }

    /// Changes settings. Pictures chosen with other filters are dropped.
    pub fn configure(&self, change: impl FnOnce(&mut Settings)) -> io::Result<()> {
        self.edit(|data| {
            let before = data.settings.clone();
            change(&mut data.settings);
            let after = &data.settings;
            if (before.sort, &before.categories) != (after.sort, &after.categories) {
                let dropped: Vec<Ready> = std::mem::take(&mut data.queue);
                // They were never shown, so they may come back.
                data.seen
                    .retain(|key| dropped.iter().all(|ready| ready.key != *key));
                for ready in dropped {
                    let _ = fs::remove_file(ready.path);
                }
            }
        })
    }

    pub fn is_due(&self, now: LocalTime) -> bool {
        let mut data = self.lock();
        let change_at = data.settings.change_at;
        data.last_day = schedule::correct(data.last_day, now, change_at);
        schedule::is_due(data.last_day, now, change_at)
    }

    /// The day of the last change, as counted by [`current_day`].
    pub fn last_day(&self) -> Option<i64> {
        self.lock().last_day
    }

    /// The next picture for screens of `size`.
    pub fn ready(&self, size: Size) -> Option<Ready> {
        let data = self.lock();
        let mut queue = data.queue.iter();
        queue.find(|r| r.size == size && r.path.is_file()).cloned()
    }

    pub fn count(&self, size: Size) -> usize {
        let data = self.lock();
        data.queue
            .iter()
            .filter(|r| r.size == size && r.path.is_file())
            .count()
    }

    /// Records that pictures became the wallpaper, under the file names in the second place.
    pub fn used(&self, pictures: &[(Ready, PathBuf)], day: i64) -> io::Result<()> {
        self.edit(|data| {
            for (ready, shown_as) in pictures {
                data.queue.retain(|r| r != ready);
                let _ = fs::remove_file(&ready.path);
                data.shown.retain(|s| s.path != *shown_as);
                data.shown.push(Shown {
                    path: shown_as.clone(),
                    key: ready.key.clone(),
                });
            }
            trim(&mut data.shown, MAX_SHOWN);
            data.last_day = Some(day);
        })
    }

    /// Never shows the pictures behind these wallpaper files again. Returns whether any of them
    /// came from here.
    pub fn block(&self, shown: &[PathBuf]) -> bool {
        self.edit(|data| {
            let keys: Vec<String> = data
                .shown
                .iter()
                .filter(|s| shown.contains(&s.path))
                .map(|s| s.key.clone())
                .collect();
            let found = !keys.is_empty();
            for key in keys {
                if !data.blocked.contains(&key) {
                    data.blocked.push(key);
                }
            }
            found
        })
        .unwrap_or(false)
    }

    /// Prepares pictures until [`READY`] are waiting for screens of `size`. Returns how many
    /// were added.
    pub fn refill(
        &self,
        size: Size,
        source: &Source,
        random: &mut fastrand::Rng,
    ) -> Result<usize, ProviderError> {
        let needed = READY.saturating_sub(self.count(size));
        if needed == 0 {
            return Ok(0);
        }
        let mut candidates = self.candidates(size, source, self.settings().sort)?;
        if candidates.is_empty() {
            candidates = self.candidates(size, source, Sort::Random)?;
        }
        if candidates.is_empty() {
            // Everything was shown. Let the older half come back, once the last one is used up.
            if self.count(size) == 0 {
                let _ = self.edit(|data| drop(data.seen.drain(..data.seen.len().div_ceil(2))));
            }
            return Ok(0);
        }
        random.shuffle(&mut candidates);
        let mut added = 0;
        for item in candidates {
            if added == needed {
                break;
            }
            match self.prepare(&item, size, source) {
                Ok(true) => added += 1,
                // This picture is broken; another one will do.
                Ok(false) => {}
                Err(ProviderError::BadResponse(_) | ProviderError::Status(_)) => {}
                // Without a connection the others would fail as well.
                Err(_) if added > 0 => break,
                Err(error) => return Err(error),
            }
        }
        Ok(added)
    }

    fn candidates(
        &self,
        size: Size,
        source: &Source,
        sort: Sort,
    ) -> Result<Vec<Item>, ProviderError> {
        let capabilities = source.provider.info().capabilities;
        let sort = match capabilities.sorts.first() {
            Some(first) if !capabilities.sorts.contains(&sort) => *first,
            _ => sort,
        };
        let mut request = ListRequest {
            sort,
            categories: self.settings().categories,
            min_size: capabilities.min_size.then_some(size),
            ratios: Ratio::nearest(size, capabilities.ratios)
                .into_iter()
                .collect(),
            ..Default::default()
        };
        let mut found: Vec<Item> = vec![];
        for _ in 0..MAX_PAGES {
            let page = source.provider.list(&request, source.http)?;
            for item in page.items {
                let pixels = item.size.width as u64 * item.size.height as u64;
                let sharp = fit(item.size, size).scale <= 1.0;
                let known = found.iter().any(|f| f.id == item.id) || self.knows(&item);
                if sharp && pixels <= MAX_PIXELS && !known {
                    found.push(item);
                }
            }
            match page.next {
                Some(next) if found.len() < CANDIDATES => request.cursor = Some(next),
                _ => break,
            }
        }
        let loss = |item: &Item| fit(item.size, size).crop_loss;
        for limit in CROP {
            if found.iter().any(|item| loss(item) <= limit) {
                found.retain(|item| loss(item) <= limit);
                return Ok(found);
            }
        }
        Ok(vec![])
    }

    fn prepare(&self, item: &Item, size: Size, source: &Source) -> Result<bool, ProviderError> {
        let bytes = (source.download)(item)?;
        let Ok(pixels) = cover(&bytes, size, 0.0, Rgb(0, 0, 0)) else {
            return Ok(false);
        };
        let Ok(path) = write_picture(&self.dir.join("queue"), &pixels) else {
            return Ok(false);
        };
        let saved = self.edit(|data| {
            data.seen.push(key(item));
            trim(&mut data.seen, MAX_SEEN);
            data.queue.retain(|r| r.path.is_file());
            data.queue.push(Ready {
                key: key(item),
                size,
                path,
            });
        });
        Ok(saved.is_ok())
    }

    fn knows(&self, item: &Item) -> bool {
        let (key, data) = (key(item), self.lock());
        data.seen.contains(&key) || data.blocked.contains(&key)
    }

    fn lock(&self) -> MutexGuard<'_, Data> {
        self.data
            .lock()
            .unwrap_or_else(|broken| broken.into_inner())
    }

    fn edit<T>(&self, change: impl FnOnce(&mut Data) -> T) -> io::Result<T> {
        let mut data = self.lock();
        let result = change(&mut data);
        self.store(&data)?;
        Ok(result)
    }

    fn store(&self, data: &Data) -> io::Result<()> {
        write_whole(&self.dir.join("daily.json"), &serde_json::to_vec(data)?)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

fn key(item: &Item) -> String {
    format!("{}:{}", item.provider, item.id)
}

fn trim<T>(list: &mut Vec<T>, max: usize) {
    if list.len() > max {
        list.drain(..list.len() - max);
    }
}

#[cfg(test)]
mod tests;
