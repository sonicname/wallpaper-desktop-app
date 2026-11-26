//! Pictures the user wants to keep, from any source. Also a source of its own.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use wallpaper_core::{
    Capabilities, Http, Item, ListRequest, Page, Provider, ProviderError, ProviderInfo, RateLimit,
    Ratio, Size, Sort, Thumb, ThumbCrop, local_path,
};
use wallpaper_host::write_whole;

pub const ID: &str = "favorites";

static INFO: ProviderInfo = ProviderInfo {
    id: ID,
    name: "Yêu thích",
    capabilities: Capabilities {
        search: false,
        sorts: &[Sort::Newest, Sort::Random],
        categories: &[],
        ratios: Ratio::COMMON,
        colors: &[],
        min_size: true,
    },
    // Nothing here uses the network.
    rate: RateLimit {
        requests: 1000,
        per: Duration::from_secs(1),
    },
};

pub struct Favorites {
    dir: PathBuf,
    /// Oldest first.
    items: Mutex<Vec<Item>>,
}

impl Favorites {
    pub fn open(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(dir.join("thumbs"))?;
        let bytes = fs::read(dir.join("favorites.json")).unwrap_or_default();
        let items: Vec<Item> = serde_json::from_slice(&bytes).unwrap_or_default();
        let favorites = Self {
            dir,
            items: Mutex::new(items),
        };
        favorites.remove_unused_thumbs();
        Ok(favorites)
    }

    pub fn len(&self) -> usize {
        self.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, item: &Item) -> bool {
        self.contains_key(&item.key())
    }

    /// Whether the picture with this [`Item::key`] is kept.
    pub fn contains_key(&self, key: &str) -> bool {
        self.lock().iter().any(|i| i.key() == key)
    }

    /// Keeps `item`. With a copy of its thumbnail, it shows without the network, and after the
    /// caches were emptied.
    pub fn add(&self, mut item: Item, thumb: Option<(Vec<u8>, Size, ThumbCrop)>) -> io::Result<()> {
        if self.contains(&item) {
            return Ok(());
        }
        if let Some((bytes, size, crop)) = thumb {
            let path = self.thumb_file(&item);
            write_whole(&path, &bytes)?;
            item.thumbs = vec![Thumb {
                url: path.to_string_lossy().into_owned(),
                size,
                crop,
            }];
        }
        let mut items = self.lock();
        items.push(item);
        self.store(&items)
    }

    /// The copy of the thumbnail stays until the next start, in case the user changes their
    /// mind while the picture is still open.
    pub fn remove(&self, item: &Item) -> io::Result<()> {
        let key = item.key();
        let mut items = self.lock();
        items.retain(|i| i.key() != key);
        self.store(&items)
    }

    fn thumb_file(&self, item: &Item) -> PathBuf {
        // FNV-1a, 64 bit: stable between releases.
        let hash = item.key().bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
        });
        self.dir.join("thumbs").join(format!("{hash:016x}.img"))
    }

    fn remove_unused_thumbs(&self) {
        let items = self.lock();
        let used: Vec<&Path> = items
            .iter()
            .flat_map(|item| &item.thumbs)
            .filter_map(|thumb| local_path(&thumb.url))
            .collect();
        let Ok(files) = fs::read_dir(self.dir.join("thumbs")) else {
            return;
        };
        for file in files.flatten() {
            if !used.contains(&file.path().as_path()) {
                // Only costs a little space if it fails.
                let _ = fs::remove_file(file.path());
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Item>> {
        self.items
            .lock()
            .unwrap_or_else(|broken| broken.into_inner())
    }

    fn store(&self, items: &[Item]) -> io::Result<()> {
        write_whole(
            &self.dir.join("favorites.json"),
            &serde_json::to_vec(items)?,
        )
    }
}

impl Provider for Favorites {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn list(&self, request: &ListRequest, _: &dyn Http) -> Result<Page, ProviderError> {
        // A file from this computer may have been deleted since. It comes back with the file.
        let exists = |item: &&Item| item.local_file().is_none_or(Path::is_file);
        let mut items: Vec<Item> = self
            .lock()
            .iter()
            .rev()
            .filter(|item| request.accepts(item.size))
            .filter(exists)
            .cloned()
            .collect();
        if request.sort == Sort::Random {
            fastrand::shuffle(&mut items);
        }
        Ok(Page {
            total: Some(items.len() as u64),
            items,
            next: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Folder(PathBuf);

    impl Folder {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("wallpaper-fav-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct NoNetwork;

    impl Http for NoNetwork {
        fn get(&self, _: &str) -> Result<Vec<u8>, ProviderError> {
            unreachable!("favorites never use the network")
        }
    }

    fn item(id: &str, width: u32, height: u32) -> Item {
        Item {
            provider: "web".into(),
            id: id.into(),
            size: Size::new(width, height),
            image_url: format!("https://pictures.invalid/{id}.jpg"),
            page_url: String::new(),
            file_bytes: 1,
            mime: "image/jpeg".into(),
            colors: vec![],
            thumbs: vec![Thumb {
                url: format!("https://pictures.invalid/{id}-small.jpg"),
                size: Size::new(300, 200),
                crop: ThumbCrop::Whole,
            }],
        }
    }

    /// The default request asks for a random order.
    fn newest() -> ListRequest {
        ListRequest {
            sort: Sort::Newest,
            ..Default::default()
        }
    }

    fn ids(favorites: &Favorites, request: &ListRequest) -> Vec<String> {
        let page = favorites.list(request, &NoNetwork).unwrap();
        page.items.into_iter().map(|i| i.id).collect()
    }

    #[test]
    fn keeps_pictures_across_restarts_newest_first() {
        let folder = Folder::new("keep");
        let favorites = Favorites::open(&folder.0).unwrap();
        assert!(favorites.is_empty());
        for id in ["a", "b", "c"] {
            favorites.add(item(id, 1920, 1080), None).unwrap();
        }
        favorites.add(item("b", 1920, 1080), None).unwrap();
        favorites.remove(&item("a", 1920, 1080)).unwrap();
        drop(favorites);
        let favorites = Favorites::open(&folder.0).unwrap();
        assert_eq!(favorites.len(), 2);
        assert!(favorites.contains(&item("c", 0, 0)));
        assert!(!favorites.contains(&item("a", 0, 0)));
        assert_eq!(ids(&favorites, &newest()), ["c", "b"]);
    }

    #[test]
    fn the_same_id_from_another_source_is_another_picture() {
        let folder = Folder::new("sources");
        let favorites = Favorites::open(&folder.0).unwrap();
        favorites.add(item("a", 10, 10), None).unwrap();
        let other = Item {
            provider: "elsewhere".into(),
            ..item("a", 10, 10)
        };
        assert!(!favorites.contains(&other));
    }

    #[test]
    fn the_thumbnail_is_copied_and_cleaned_up_later() {
        let folder = Folder::new("thumb");
        let favorites = Favorites::open(&folder.0).unwrap();
        let copy = (
            b"small picture".to_vec(),
            Size::new(150, 100),
            ThumbCrop::Cropped,
        );
        favorites.add(item("a", 1920, 1080), Some(copy)).unwrap();
        let page = favorites.list(&ListRequest::default(), &NoNetwork).unwrap();
        let thumb = &page.items[0].thumbs[0];
        assert_eq!(
            (thumb.size, thumb.crop),
            (Size::new(150, 100), ThumbCrop::Cropped)
        );
        let file = local_path(&thumb.url)
            .expect("a file on this computer")
            .to_owned();
        assert_eq!(fs::read(&file).unwrap(), b"small picture");

        // Removed, the copy stays while the app runs, and goes with the next start.
        favorites.remove(&page.items[0]).unwrap();
        assert!(file.is_file());
        favorites.add(page.items[0].clone(), None).unwrap();
        drop(favorites);
        let favorites = Favorites::open(&folder.0).unwrap();
        assert!(file.is_file(), "still used");
        favorites.remove(&item("a", 0, 0)).unwrap();
        drop(favorites);
        Favorites::open(&folder.0).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn filters_by_screen_and_hides_deleted_files() {
        let folder = Folder::new("filter");
        let favorites = Favorites::open(folder.0.join("store")).unwrap();
        favorites.add(item("wide", 3840, 2160), None).unwrap();
        favorites.add(item("small", 1280, 720), None).unwrap();
        favorites.add(item("tall", 2160, 3840), None).unwrap();
        let file = folder.0.join("mine.png");
        fs::write(&file, b"x").unwrap();
        let local = Item {
            image_url: file.to_string_lossy().into_owned(),
            ..item("local", 3840, 2160)
        };
        favorites.add(local, None).unwrap();

        let screen = ListRequest {
            min_size: Some(Size::new(2560, 1440)),
            ratios: vec![Ratio(16, 9)],
            ..newest()
        };
        assert_eq!(ids(&favorites, &screen), ["local", "wide"]);
        fs::remove_file(&file).unwrap();
        assert_eq!(ids(&favorites, &screen), ["wide"]);
        assert_eq!(
            favorites.len(),
            4,
            "the record stays, in case the file comes back"
        );

        let random = ListRequest {
            sort: Sort::Random,
            ..Default::default()
        };
        let mut all = ids(&favorites, &random);
        all.sort();
        assert_eq!(all, ["small", "tall", "wide"]);
    }

    #[test]
    fn a_damaged_file_is_an_empty_list() {
        let folder = Folder::new("damaged");
        fs::write(folder.0.join("favorites.json"), b"[{oops").unwrap();
        let favorites = Favorites::open(&folder.0).unwrap();
        assert!(favorites.is_empty());
        favorites.add(item("a", 1, 1), None).unwrap();
        assert_eq!(favorites.len(), 1);
    }
}
