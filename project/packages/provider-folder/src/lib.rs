//! Pictures from a folder on this computer.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
    time::{Duration, SystemTime},
};

use wallpaper_core::{
    Capabilities, Cursor, Http, Item, ListRequest, Page, Provider, ProviderError, ProviderInfo,
    RateLimit, Ratio, Size, Sort, Thumb, ThumbCrop,
};

pub const ID: &str = "folder";

const PAGE: usize = 60;
/// Folders inside folders that are still searched.
const MAX_DEPTH: usize = 4;
const MAX_FILES: usize = 20_000;
/// What the app can decode.
const KINDS: [(&str, &str); 5] = [
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("png", "image/png"),
    ("webp", "image/webp"),
    ("bmp", "image/bmp"),
];

static INFO: ProviderInfo = ProviderInfo {
    id: ID,
    name: "Thư mục",
    capabilities: Capabilities {
        search: true,
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

struct Entry {
    path: PathBuf,
    modified: SystemTime,
    bytes: u64,
    mime: &'static str,
}

#[derive(Default)]
struct State {
    root: Option<PathBuf>,
    /// The files of the listing that is being paged through, in its order.
    files: Vec<Entry>,
}

pub struct Folder {
    /// Remembers the chosen folder.
    file: PathBuf,
    state: Mutex<State>,
}

impl Folder {
    pub fn open(file: impl Into<PathBuf>) -> Self {
        let file = file.into();
        let saved = fs::read_to_string(&file).unwrap_or_default();
        let root = Some(PathBuf::from(saved.trim())).filter(|path| path.is_absolute());
        Self {
            file,
            state: Mutex::new(State {
                root,
                files: vec![],
            }),
        }
    }

    /// The chosen folder, if any.
    pub fn path(&self) -> Option<PathBuf> {
        self.lock().root.clone()
    }

    pub fn set(&self, folder: PathBuf) -> io::Result<()> {
        fs::write(&self.file, folder.to_string_lossy().as_bytes())?;
        let mut state = self.lock();
        state.root = Some(folder);
        state.files.clear();
        Ok(())
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|broken| broken.into_inner())
    }
}

impl Provider for Folder {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn list(&self, request: &ListRequest, _: &dyn Http) -> Result<Page, ProviderError> {
        let mut state = self.lock();
        let Some(root) = state.root.clone() else {
            return Ok(Page::default());
        };
        let start = match &request.cursor {
            Some(Cursor(position)) => position.parse().unwrap_or(0),
            None => {
                // The first page looks at the folder again; the others keep its order.
                if !root.is_dir() {
                    return Err(ProviderError::NotFound);
                }
                state.files = scan(&root);
                match request.sort {
                    Sort::Random => fastrand::shuffle(&mut state.files),
                    _ => state
                        .files
                        .sort_by(|a, b| b.modified.cmp(&a.modified).then(a.path.cmp(&b.path))),
                }
                0
            }
        };
        let text = request.text.trim().to_lowercase();
        let named = |entry: &Entry| {
            let name = entry.path.file_name().unwrap_or_default();
            text.is_empty() || name.to_string_lossy().to_lowercase().contains(&text)
        };
        let mut items = vec![];
        let mut position = start;
        while position < state.files.len() && items.len() < PAGE {
            let entry = &state.files[position];
            position += 1;
            if !named(entry) {
                continue;
            }
            // Only the header is read. A file that is not a picture after all is left out.
            let Ok((width, height)) = image::image_dimensions(&entry.path) else {
                continue;
            };
            let size = Size::new(width, height);
            if request.accepts(size) {
                items.push(item(entry, size));
            }
        }
        // Without filters on the size, the number is known without opening every file.
        let unfiltered = request.min_size.is_none() && request.ratios.is_empty();
        let total = unfiltered.then(|| state.files.iter().filter(|e| named(e)).count() as u64);
        let next = (position < state.files.len()).then(|| Cursor(position.to_string()));
        Ok(Page { items, next, total })
    }
}

fn item(entry: &Entry, size: Size) -> Item {
    let location = entry.path.to_string_lossy().into_owned();
    Item {
        provider: ID.into(),
        id: location.clone(),
        size,
        image_url: location.clone(),
        page_url: String::new(),
        file_bytes: entry.bytes,
        mime: entry.mime.into(),
        colors: vec![],
        // The app makes a small copy when it shows the picture.
        thumbs: vec![Thumb {
            url: location,
            size,
            crop: ThumbCrop::Whole,
        }],
    }
}

fn scan(root: &Path) -> Vec<Entry> {
    let mut files = vec![];
    let mut folders = vec![(root.to_owned(), 0)];
    while let Some((folder, depth)) = folders.pop() {
        // A folder that cannot be read is skipped, like a file that cannot be read.
        for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
            let (path, Ok(meta)) = (entry.path(), entry.metadata()) else {
                continue;
            };
            if meta.is_dir() {
                if depth < MAX_DEPTH {
                    folders.push((path, depth + 1));
                }
                continue;
            }
            let extension = path.extension().unwrap_or_default().to_string_lossy();
            let kind = KINDS.iter().find(|k| k.0.eq_ignore_ascii_case(&extension));
            if let Some((_, mime)) = kind
                && files.len() < MAX_FILES
            {
                files.push(Entry {
                    modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    bytes: meta.len(),
                    mime,
                    path,
                });
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    struct Temp(PathBuf);

    impl Temp {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("wallpaper-folder-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(dir.join("pictures")).unwrap();
            Self(dir)
        }

        fn pictures(&self) -> PathBuf {
            self.0.join("pictures")
        }

        /// A picture that was changed `age` minutes ago.
        fn add(&self, name: &str, width: u32, height: u32, age: u64) -> PathBuf {
            let path = self.pictures().join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            image::RgbImage::new(width, height).save(&path).unwrap();
            let time = SystemTime::now() - Duration::from_secs(age * 60);
            File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(time)
                .unwrap();
            path
        }

        fn folder(&self) -> Folder {
            let folder = Folder::open(self.0.join("folder.txt"));
            folder.set(self.pictures()).unwrap();
            folder
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct NoNetwork;

    impl Http for NoNetwork {
        fn get(&self, _: &str) -> Result<Vec<u8>, ProviderError> {
            unreachable!("a folder never uses the network")
        }
    }

    /// The default request asks for a random order.
    fn newest() -> ListRequest {
        ListRequest {
            sort: Sort::Newest,
            ..Default::default()
        }
    }

    fn names(page: &Page) -> Vec<String> {
        let name = |item: &Item| {
            Path::new(&item.id)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        };
        page.items.iter().map(name).collect()
    }

    fn list(folder: &Folder, request: &ListRequest) -> Page {
        folder.list(request, &NoNetwork).unwrap()
    }

    #[test]
    fn lists_pictures_newest_first_with_their_real_size() {
        let temp = Temp::new("list");
        temp.add("old.png", 40, 30, 30);
        temp.add("new.jpg", 64, 36, 1);
        temp.add("deep/inside/middle.bmp", 20, 20, 10);
        fs::write(temp.pictures().join("notes.txt"), b"not a picture").unwrap();
        fs::write(temp.pictures().join("broken.png"), b"not a picture either").unwrap();

        let page = list(&temp.folder(), &newest());
        assert_eq!(names(&page), ["new.jpg", "middle.bmp", "old.png"]);
        let first = &page.items[0];
        assert_eq!(first.size, Size::new(64, 36));
        assert_eq!(first.mime, "image/jpeg");
        assert_eq!(
            first.local_file(),
            Some(temp.pictures().join("new.jpg").as_path())
        );
        assert_eq!(first.thumbs[0].url, first.image_url);
        assert_eq!(page.next, None);
        // The broken file is only found out when it is opened.
        assert_eq!(page.total, Some(4));
    }

    #[test]
    fn searches_names_and_filters_by_screen() {
        let temp = Temp::new("search");
        temp.add("Mountain Lake.png", 64, 36, 1);
        temp.add("mountain small.png", 32, 18, 2);
        temp.add("city.png", 64, 36, 3);
        temp.add("tall mountain.png", 36, 64, 4);
        let folder = temp.folder();
        let search = ListRequest {
            text: " MOUNTAIN ".into(),
            ..newest()
        };
        let page = list(&folder, &search);
        assert_eq!(
            names(&page),
            [
                "Mountain Lake.png",
                "mountain small.png",
                "tall mountain.png"
            ]
        );
        assert_eq!(page.total, Some(3));
        let fitting = ListRequest {
            min_size: Some(Size::new(64, 36)),
            ratios: vec![Ratio(16, 9)],
            ..search
        };
        let page = list(&folder, &fitting);
        assert_eq!(names(&page), ["Mountain Lake.png"]);
        assert_eq!(page.total, None);
    }

    #[test]
    fn pages_through_a_large_folder_without_gaps_or_repeats() {
        let temp = Temp::new("pages");
        for index in 0..150 {
            temp.add(&format!("{index:03}.png"), 4, 4, index);
        }
        let folder = temp.folder();
        for sort in [Sort::Newest, Sort::Random] {
            let mut request = ListRequest {
                sort,
                ..Default::default()
            };
            let mut seen = vec![];
            loop {
                let page = list(&folder, &request);
                assert!(page.items.len() <= PAGE);
                seen.extend(names(&page));
                match page.next {
                    Some(next) => request.cursor = Some(next),
                    None => break,
                }
            }
            let in_order: Vec<String> = (0..150).map(|i| format!("{i:03}.png")).collect();
            assert_eq!(seen.len(), 150);
            assert_eq!(seen == in_order, sort == Sort::Newest, "{sort:?}");
            seen.sort();
            assert_eq!(seen, in_order);
        }
    }

    #[test]
    fn remembers_the_folder_and_reports_a_missing_one() {
        let temp = Temp::new("remember");
        let folder = Folder::open(temp.0.join("folder.txt"));
        assert_eq!(folder.path(), None);
        assert_eq!(list(&folder, &newest()), Page::default());
        temp.add("a.png", 4, 4, 1);
        folder.set(temp.pictures()).unwrap();
        drop(folder);

        let folder = Folder::open(temp.0.join("folder.txt"));
        assert_eq!(folder.path(), Some(temp.pictures()));
        assert_eq!(list(&folder, &newest()).items.len(), 1);
        fs::remove_dir_all(temp.pictures()).unwrap();
        let result = folder.list(&newest(), &NoNetwork);
        assert_eq!(result, Err(ProviderError::NotFound));
    }

    #[test]
    fn a_new_first_page_sees_new_files() {
        let temp = Temp::new("fresh");
        temp.add("a.png", 4, 4, 5);
        let folder = temp.folder();
        assert_eq!(list(&folder, &newest()).items.len(), 1);
        temp.add("b.png", 4, 4, 1);
        assert_eq!(names(&list(&folder, &newest())), ["b.png", "a.png"]);
    }
}
