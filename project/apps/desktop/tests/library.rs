//! Favorites, the folder on this computer and switching between sources, against fakes.
//! One test per process, because Slint allows one platform.

#[path = "../examples/support/stage.rs"]
mod stage;

use std::{
    fs,
    io::Cursor as Bytes,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use slint::{ComponentHandle, Model};
use stage::Stage;
use wallpaper_core::{
    Autostart, Backdrop, BackdropMode, Capabilities, Desktop, DesktopError, Http, Item,
    ListRequest, LocalTime, Page, Picture, Provider, ProviderError, ProviderInfo, RateLimit, Ratio,
    Screen, Size, Sort, System, Thumb, ThumbCrop,
};
use wallpaper_desktop::{App, AppWindow, Config};
use wallpaper_host::DiskCache;

const WAIT: Duration = Duration::from_secs(10);
const SCREEN: Size = Size::new(320, 180);

static INFO: ProviderInfo = ProviderInfo {
    id: "web",
    name: "Web",
    capabilities: Capabilities {
        search: true,
        sorts: &[Sort::Top],
        categories: &[],
        ratios: &[Ratio(16, 9)],
        colors: &[],
        min_size: true,
    },
    rate: RateLimit {
        requests: 1000,
        per: Duration::from_secs(1),
    },
};

struct Web;

fn url(kind: &str, id: u32) -> String {
    format!("https://web.invalid/{kind}/{id}")
}

impl Provider for Web {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn list(&self, _: &ListRequest, _: &dyn Http) -> Result<Page, ProviderError> {
        let items = (0..6).map(|id| Item {
            provider: "web".into(),
            id: id.to_string(),
            size: Size::new(640, 360),
            image_url: url("full", id),
            page_url: url("page", id),
            file_bytes: 0,
            mime: "image/png".into(),
            colors: vec![],
            thumbs: vec![Thumb {
                url: url("small", id),
                size: Size::new(160, 90),
                crop: ThumbCrop::Whole,
            }],
        });
        Ok(Page {
            items: items.collect(),
            next: None,
            total: Some(6),
        })
    }
}

struct OneScreen(Mutex<Backdrop>);

impl OneScreen {
    fn path(&self) -> PathBuf {
        self.0.lock().unwrap().pictures[0].path.clone()
    }
}

impl Desktop for OneScreen {
    fn screens(&self) -> Vec<Screen> {
        vec![Screen {
            id: "only".into(),
            size: SCREEN,
            primary: true,
        }]
    }

    fn workspace(&self) -> Option<String> {
        self.0.lock().unwrap().workspace.clone()
    }

    fn read(&self) -> Result<Backdrop, DesktopError> {
        Ok(self.0.lock().unwrap().clone())
    }

    fn apply(&self, pictures: &[(String, PathBuf)], _: Option<&str>) -> Result<(), DesktopError> {
        self.0.lock().unwrap().pictures[0].path = pictures[0].1.clone();
        Ok(())
    }

    fn restore(&self, backdrop: &Backdrop) -> Result<(), DesktopError> {
        *self.0.lock().unwrap() = backdrop.clone();
        Ok(())
    }
}

/// The folder dialog answers with whatever the test put here.
struct Machine {
    picked: Mutex<Option<PathBuf>>,
}

impl System for Machine {
    fn now(&self) -> LocalTime {
        LocalTime::new(2026, 11, 26, 9, 0)
    }

    fn do_not_disturb(&self) -> bool {
        false
    }

    fn autostart(&self) -> Autostart {
        Autostart::Off
    }

    fn set_autostart(&self, _: bool) -> Result<Autostart, String> {
        Ok(Autostart::Off)
    }

    fn pick_folder(&self) -> Option<PathBuf> {
        self.picked.lock().unwrap().clone()
    }
}

fn png(width: u32, height: u32, shade: u8) -> Vec<u8> {
    let picture = image::RgbImage::from_fn(width, height, |x, y| {
        image::Rgb([shade, (x % 256) as u8, (y % 256) as u8])
    });
    let mut bytes = Bytes::new(vec![]);
    picture
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

/// A picture file that was changed `age` minutes ago.
fn file(folder: &Path, name: &str, width: u32, height: u32, age: u64) {
    let path = folder.join(name);
    fs::write(&path, png(width, height, age as u8)).unwrap();
    let time = SystemTime::now() - Duration::from_secs(age * 60);
    let opened = fs::File::options().write(true).open(&path).unwrap();
    opened.set_modified(time).unwrap();
}

fn cells(window: &AppWindow) -> Vec<wallpaper_desktop::Cell> {
    let rows = window.get_rows();
    rows.iter()
        .flat_map(|row| row.cells.iter().collect::<Vec<_>>())
        .collect()
}

fn on(options: slint::ModelRc<wallpaper_desktop::Option>) -> Vec<bool> {
    options.iter().map(|o| o.on).collect()
}

#[test]
fn favorites_and_a_folder_of_my_own() {
    let mut stage = Stage::new(1280, 820, 1.0);
    let data = std::env::temp_dir().join(format!("wallpaper-library-{}", std::process::id()));
    let _ = fs::remove_dir_all(&data);
    let app_data = data.join("app");
    let thumbs = DiskCache::open(app_data.join("thumbs")).unwrap();
    let originals = DiskCache::open(app_data.join("originals")).unwrap();
    for id in 0..6 {
        thumbs
            .put(&url("small", id), &png(160, 90, id as u8))
            .unwrap();
        originals
            .put(&url("full", id), &png(640, 360, id as u8))
            .unwrap();
    }
    let pictures = data.join("my pictures");
    fs::create_dir_all(&pictures).unwrap();
    file(&pictures, "lake.png", 640, 360, 1);
    file(&pictures, "hill.png", 640, 360, 2);
    file(&pictures, "tall.png", 360, 640, 3);
    file(&pictures, "tiny.png", 32, 18, 4);

    let desktop = Arc::new(OneScreen(Mutex::new(Backdrop {
        mode: BackdropMode::Picture,
        placement: 4,
        workspace: Some("one".into()),
        pictures: vec![Picture {
            screen: "only".into(),
            path: PathBuf::new(),
            kept: None,
        }],
    })));
    let machine = Arc::new(Machine {
        picked: Mutex::new(None),
    });
    let app = App::new(Config {
        providers: vec![Arc::new(Web)],
        desktop: desktop.clone(),
        system: machine.clone(),
        data_dir: Some(app_data.clone()),
    })
    .unwrap();
    let window = app.window().clone_strong();
    app.open().unwrap();
    let count = |n: usize| {
        let window = window.clone_strong();
        move || cells(&window).len() == n && !window.get_busy()
    };
    let settle = |stage: &mut Stage| assert!(stage.run_until(WAIT, || !window.get_changing()));
    let applied = |stage: &mut Stage| {
        window.invoke_apply();
        settle(stage);
        assert_eq!(window.get_toast(), "Đã đặt làm hình nền.");
        image::image_dimensions(desktop.path()).unwrap()
    };

    // The sources from the web come first, then the user's own.
    assert!(stage.run_until(WAIT, count(6)));
    let names: Vec<_> = window.get_sources().iter().map(|s| s.label).collect();
    assert_eq!(names, ["Web", "Yêu thích", "Thư mục"]);
    assert_eq!(on(window.get_sources()), [true, false, false]);

    // A heart keeps a picture, and shows in the grid.
    window.invoke_open(2);
    assert!(!window.get_detail().favorite);
    assert_eq!(window.get_detail().source, "Web · 2");
    assert_eq!(window.get_detail().page_label, "Mở trang gốc");
    window.invoke_toggle_favorite();
    assert!(stage.run_until(WAIT, || window.get_detail().favorite));
    let hearts: Vec<i32> = cells(&window)
        .iter()
        .filter(|c| c.favorite)
        .map(|c| c.index)
        .collect();
    assert_eq!(hearts, [2]);

    // Favorites are a source of their own, with a thumbnail that needs no cache.
    window.invoke_pick_source(1);
    assert!(!window.get_detail_open());
    assert!(stage.run_until(WAIT, count(1)));
    assert_eq!(on(window.get_sources()), [false, true, false]);
    assert!(!window.get_can_search());
    assert_eq!(window.get_summary(), "1 ảnh");
    fs::remove_dir_all(app_data.join("thumbs")).unwrap();
    app.close();
    app.open().unwrap();
    assert!(stage.run_until(WAIT, || cells(&window).iter().any(|c| c.ready)));
    window.invoke_open(0);
    assert!(window.get_detail().favorite);
    assert_eq!(
        window.get_detail().source,
        "Web · 2",
        "it still names where it came from"
    );
    assert_eq!(applied(&mut stage), (320, 180));

    // Taken out again, it is gone the next time the list is shown.
    window.invoke_toggle_favorite();
    assert!(!window.get_detail().favorite);
    window.invoke_pick_source(0);
    assert!(stage.run_until(WAIT, count(6)));
    assert!(cells(&window).iter().all(|c| !c.favorite));
    window.invoke_pick_source(1);
    assert!(stage.run_until(WAIT, || {
        window.get_message().contains("Chưa có ảnh yêu thích")
    }));
    assert!(cells(&window).is_empty());

    // The folder source asks for a folder first. Cancelling the dialog changes nothing.
    window.invoke_pick_source(2);
    assert!(stage.run_until(WAIT, || window.get_needs_folder() && !window.get_busy()));
    assert_eq!(window.get_message(), "");
    window.invoke_pick_folder();
    stage.run_for(Duration::from_millis(200));
    assert!(window.get_needs_folder());
    *machine.picked.lock().unwrap() = Some(pictures.clone());
    window.invoke_pick_folder();
    assert!(stage.run_until(WAIT, count(4)));
    assert!(!window.get_needs_folder());
    assert_eq!(window.get_folder_label(), "Thư mục: my pictures");
    assert_eq!(window.get_summary(), "4 ảnh");
    assert!(stage.run_until(WAIT, || cells(&window).iter().all(|c| c.ready)));

    // Nothing is hidden until the user asks for pictures that fit the screen.
    assert!(!window.get_screen_fit().on);
    window.invoke_toggle_screen_fit();
    assert!(stage.run_until(WAIT, count(2)));
    window.invoke_toggle_screen_fit();
    assert!(stage.run_until(WAIT, count(4)));
    window.invoke_search("TALL".into());
    assert!(stage.run_until(WAIT, count(1)));
    window.invoke_search("".into());
    assert!(stage.run_until(WAIT, count(4)));

    // A file is previewed, set and kept like any other picture. Newest first.
    window.invoke_open(0);
    assert_eq!(window.get_detail().source, "Thư mục · lake.png");
    assert_eq!(window.get_detail().page_label, "Mở thư mục chứa ảnh");
    assert_eq!(window.get_detail().resolution, "640 × 360");
    assert!(stage.run_until(WAIT, || window.get_picture_sharp()
        && window.get_has_picture()));
    assert_eq!(applied(&mut stage), (320, 180));
    window.invoke_toggle_favorite();
    assert!(stage.run_until(WAIT, || window.get_detail().favorite));
    window.set_detail_open(false);

    // Every source keeps its own filters.
    window.set_search_text("hill".into());
    window.invoke_search("hill".into());
    assert!(stage.run_until(WAIT, count(1)));
    window.invoke_pick_source(1);
    assert_eq!(window.get_search_text(), "");
    assert!(stage.run_until(WAIT, count(1)));
    assert_eq!(window.get_detail().source, "Thư mục · lake.png");
    window.invoke_pick_source(2);
    assert_eq!(window.get_search_text(), "hill");
    assert!(stage.run_until(WAIT, count(1)));
    window.set_search_text("".into());
    window.invoke_search("".into());
    assert!(stage.run_until(WAIT, count(4)));

    // The daily change can take its pictures from the folder: only those that fit, in turns.
    window.invoke_pick_daily_source(2);
    assert_eq!(on(window.get_daily().sources), [false, false, true]);
    // The first one may be the picture that was just set by hand, so the path says nothing.
    window.invoke_toggle_daily();
    let changed = || window.get_daily().status.contains("Đã đổi hôm nay") && !window.get_changing();
    assert!(
        stage.run_until(WAIT, changed),
        "{}",
        window.get_daily().status
    );
    assert_eq!(image::image_dimensions(desktop.path()).unwrap(), (320, 180));
    let mut shown = vec![desktop.path()];
    for _ in 0..3 {
        assert!(stage.run_until(WAIT, || window.get_daily().status.contains("Có sẵn")));
        let last = desktop.path();
        window.invoke_next_daily();
        assert!(stage.run_until(WAIT, || desktop.path() != last && !window.get_changing()));
        assert_eq!(image::image_dimensions(desktop.path()).unwrap(), (320, 180));
        shown.push(desktop.path());
    }
    shown.sort();
    shown.dedup();
    assert_eq!(
        shown.len(),
        2,
        "lake and hill fit the screen; tall and tiny do not"
    );

    // Favorites work as a daily source as well.
    window.invoke_pick_daily_source(1);
    assert_eq!(on(window.get_daily().sources), [false, true, false]);
    assert!(stage.run_until(WAIT, || window.get_daily().status.contains("Có sẵn 1 ảnh")));

    // A picture that is no favorite any more is not shown tomorrow either.
    window.invoke_open(0);
    assert!(window.get_detail().favorite);
    window.invoke_toggle_favorite();
    assert!(
        stage.run_until(WAIT, || window.get_daily().problem),
        "{}",
        window.get_daily().status
    );
    window.set_detail_open(false);

    // A folder that disappeared is explained, not shown as empty.
    fs::remove_dir_all(&pictures).unwrap();
    window.invoke_pick_source(0);
    window.invoke_pick_source(2);
    assert!(stage.run_until(WAIT, || window.get_message().contains("Không tìm thấy")));
    assert!(window.get_can_retry());

    drop(app);
    let _ = fs::remove_dir_all(&data);
}
