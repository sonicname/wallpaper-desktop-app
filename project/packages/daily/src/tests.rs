use std::{collections::HashSet, io::Cursor as Bytes, time::Duration};

use wallpaper_core::{Capabilities, Cursor, Page, ProviderInfo, RateLimit};

use super::*;

const SCREEN: Size = Size::new(64, 36);
const TALL: Size = Size::new(36, 64);

static INFO: ProviderInfo = ProviderInfo {
    id: "fake",
    name: "Fake",
    capabilities: Capabilities {
        search: false,
        sorts: &[Sort::Top, Sort::Random, Sort::Newest],
        categories: &[Category::General],
        ratios: &[Ratio(16, 9), Ratio(9, 16)],
        colors: &[],
        min_size: true,
    },
    rate: RateLimit {
        requests: 1,
        per: Duration::from_secs(1),
    },
};

/// Numbered pictures, 24 a page.
struct Fake {
    total: u32,
    size: Size,
}

impl Provider for Fake {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn list(&self, request: &ListRequest, _: &dyn Http) -> Result<Page, ProviderError> {
        let page: u32 = request.cursor.as_ref().map_or(0, |c| c.0.parse().unwrap());
        let ids = page * 24..self.total.min((page + 1) * 24);
        let next = (ids.end < self.total).then(|| Cursor((page + 1).to_string()));
        let size = match request.ratios[..] {
            [Ratio(9, 16)] => Size::new(self.size.height, self.size.width),
            _ => self.size,
        };
        let items = ids.map(|id| item(id, size)).collect();
        Ok(Page {
            items,
            next,
            total: None,
        })
    }
}

struct Offline;

impl Http for Offline {
    fn get(&self, _: &str) -> Result<Vec<u8>, ProviderError> {
        unreachable!("the fake provider needs no network")
    }
}

fn item(id: u32, size: Size) -> Item {
    Item {
        provider: "fake",
        id: id.to_string(),
        size,
        image_url: String::new(),
        page_url: String::new(),
        file_bytes: 0,
        mime: String::new(),
        colors: vec![],
        thumbs: vec![],
    }
}

/// A picture that differs from every other one.
fn download(item: &Item) -> Result<Vec<u8>, ProviderError> {
    let seed: u32 = item.id.parse().unwrap();
    let picture = image::RgbImage::from_fn(item.size.width, item.size.height, |x, y| {
        image::Rgb([(x + seed) as u8, (y * 3) as u8, (seed * 7) as u8])
    });
    let mut bytes = Bytes::new(vec![]);
    picture
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    Ok(bytes.into_inner())
}

struct Folder(PathBuf);

impl Folder {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("wallpaper-daily-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        Self(dir)
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fake(total: u32) -> Fake {
    Fake {
        total,
        size: Size::new(128, 72),
    }
}

fn refill(daily: &Daily, provider: &Fake, size: Size) -> Result<usize, ProviderError> {
    let source = Source {
        provider,
        http: &Offline,
        download: &download,
    };
    daily.refill(size, &source, &mut fastrand::Rng::with_seed(7))
}

/// Uses the next picture, as a change of the wallpaper would.
fn next(daily: &Daily, day: i64) -> Option<String> {
    let ready = daily.ready(SCREEN)?;
    let shown_as = PathBuf::from(format!("/shown/{}", ready.key));
    daily.used(&[(ready.clone(), shown_as)], day).unwrap();
    Some(ready.key)
}

#[test]
fn keeps_a_few_pictures_ready_in_the_size_of_the_screen() {
    let folder = Folder::new("ready");
    let daily = Daily::open(&folder.0).unwrap();
    assert!(daily.ready(SCREEN).is_none());
    assert_eq!(refill(&daily, &fake(100), SCREEN), Ok(READY));
    assert_eq!(refill(&daily, &fake(100), SCREEN), Ok(0));
    let ready = daily.ready(SCREEN).unwrap();
    assert_eq!(image::image_dimensions(&ready.path).unwrap(), (64, 36));
    // A portrait screen needs pictures of its own.
    assert!(daily.ready(TALL).is_none());
    assert_eq!(refill(&daily, &fake(100), TALL), Ok(READY));
    let tall = daily.ready(TALL).unwrap();
    assert_eq!(image::image_dimensions(&tall.path).unwrap(), (36, 64));
}

#[test]
fn never_repeats_until_everything_was_shown() {
    let folder = Folder::new("repeat");
    let daily = Daily::open(&folder.0).unwrap();
    let provider = fake(40);
    let mut seen = HashSet::new();
    for day in 0..40 {
        refill(&daily, &provider, SCREEN).unwrap();
        let key = next(&daily, day).expect("a picture every day");
        assert!(seen.insert(key), "repeated on day {day}");
    }
    assert_eq!(seen.len(), 40);
    // Then it starts over instead of stopping for good.
    assert!(next(&daily, 40).is_none());
    assert_eq!(refill(&daily, &provider, SCREEN), Ok(0));
    assert_eq!(refill(&daily, &provider, SCREEN), Ok(READY));
}

#[test]
fn the_order_is_not_the_order_of_the_source() {
    let folder = Folder::new("order");
    let daily = Daily::open(&folder.0).unwrap();
    let mut keys = vec![];
    for day in 0..6 {
        refill(&daily, &fake(100), SCREEN).unwrap();
        keys.push(next(&daily, day).unwrap());
    }
    let sorted: Vec<String> = (0..6).map(|id| format!("fake:{id}")).collect();
    assert_ne!(keys, sorted);
}

#[test]
fn blocked_pictures_never_come_back() {
    let folder = Folder::new("block");
    let daily = Daily::open(&folder.0).unwrap();
    let provider = fake(4);
    refill(&daily, &provider, SCREEN).unwrap();
    let key = next(&daily, 0).unwrap();
    assert!(!daily.block(&[PathBuf::from("/some/other/wallpaper.png")]));
    assert!(daily.block(&[PathBuf::from(format!("/shown/{key}"))]));
    for day in 1..30 {
        let _ = refill(&daily, &provider, SCREEN);
        assert_ne!(next(&daily, day), Some(key.clone()));
    }
}

#[test]
fn everything_survives_a_restart() {
    let folder = Folder::new("restart");
    let daily = Daily::open(&folder.0).unwrap();
    daily.configure(|s| s.enabled = true).unwrap();
    refill(&daily, &fake(100), SCREEN).unwrap();
    next(&daily, 20_000).unwrap();
    drop(daily);
    let daily = Daily::open(&folder.0).unwrap();
    assert!(daily.settings().enabled);
    assert_eq!(daily.last_day(), Some(20_000));
    assert_eq!(daily.count(SCREEN), READY - 1);
}

#[test]
fn other_filters_drop_the_prepared_pictures() {
    let folder = Folder::new("filters");
    let daily = Daily::open(&folder.0).unwrap();
    refill(&daily, &fake(3), SCREEN).unwrap();
    let path = daily.ready(SCREEN).unwrap().path;
    daily.configure(|s| s.change_at = 60).unwrap();
    assert_eq!(
        daily.count(SCREEN),
        3,
        "the hour has nothing to do with the pictures"
    );
    daily.configure(|s| s.sort = Sort::Newest).unwrap();
    assert_eq!(daily.count(SCREEN), 0);
    assert!(!path.exists());
    // They were never shown, so they are still allowed.
    assert_eq!(refill(&daily, &fake(3), SCREEN), Ok(3));
}

#[test]
fn pictures_that_would_look_bad_are_left_out() {
    let folder = Folder::new("bad");
    let daily = Daily::open(&folder.0).unwrap();
    let small = Fake {
        total: 10,
        size: Size::new(32, 18),
    };
    assert_eq!(refill(&daily, &small, SCREEN), Ok(0));
    let panorama = Fake {
        total: 10,
        size: Size::new(400, 72),
    };
    assert_eq!(refill(&daily, &panorama, SCREEN), Ok(0));
    // A little more cropping is fine when there is nothing better.
    let wide = Fake {
        total: 10,
        size: Size::new(160, 72),
    };
    assert_eq!(refill(&daily, &wide, SCREEN), Ok(READY));
}

#[test]
fn a_broken_picture_is_replaced_by_another() {
    let folder = Folder::new("broken");
    let daily = Daily::open(&folder.0).unwrap();
    let provider = fake(10);
    let source = Source {
        provider: &provider,
        http: &Offline,
        download: &|item| match item.id.parse::<u32>().unwrap() {
            0..5 => Ok(b"not a picture".to_vec()),
            5 => Err(ProviderError::Status(404)),
            _ => download(item),
        },
    };
    let mut random = fastrand::Rng::with_seed(1);
    assert_eq!(daily.refill(SCREEN, &source, &mut random), Ok(READY));
}

#[test]
fn being_offline_is_reported_and_changes_nothing() {
    let folder = Folder::new("offline");
    let daily = Daily::open(&folder.0).unwrap();
    let provider = fake(10);
    let source = Source {
        provider: &provider,
        http: &Offline,
        download: &|_| Err(ProviderError::Network("down".into())),
    };
    let result = daily.refill(SCREEN, &source, &mut fastrand::Rng::with_seed(1));
    assert!(matches!(result, Err(ProviderError::Network(_))));
    assert_eq!(refill(&daily, &fake(10), SCREEN), Ok(READY));
}

#[test]
fn a_deleted_file_is_not_offered() {
    let folder = Folder::new("deleted");
    let daily = Daily::open(&folder.0).unwrap();
    refill(&daily, &fake(100), SCREEN).unwrap();
    fs::remove_file(daily.ready(SCREEN).unwrap().path).unwrap();
    assert_eq!(daily.count(SCREEN), READY - 1);
    assert!(daily.ready(SCREEN).unwrap().path.is_file());
    assert_eq!(refill(&daily, &fake(100), SCREEN), Ok(1));
}

#[test]
fn damaged_settings_fall_back_to_the_defaults() {
    let folder = Folder::new("damaged");
    fs::create_dir_all(&folder.0).unwrap();
    fs::write(folder.0.join("daily.json"), b"{oops").unwrap();
    let daily = Daily::open(&folder.0).unwrap();
    assert_eq!(daily.settings(), Settings::default());
    assert!(!daily.settings().enabled);
}

#[test]
fn a_clock_that_was_ahead_does_not_stop_the_changes() {
    let folder = Folder::new("clock");
    let daily = Daily::open(&folder.0).unwrap();
    refill(&daily, &fake(100), SCREEN).unwrap();
    next(&daily, 500).unwrap();
    let morning = |day| LocalTime {
        day,
        minute: 9 * 60,
    };
    assert!(!daily.is_due(morning(100)), "today had its change");
    assert!(daily.is_due(morning(101)));
}
