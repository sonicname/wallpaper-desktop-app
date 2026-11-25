//! Daily changes, driven by a fake clock, desktop and source. One test per process, because
//! Slint allows one platform.

#[path = "../examples/support/stage.rs"]
mod stage;

use std::{
    fs,
    io::Cursor as Bytes,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use slint::ComponentHandle;
use stage::Stage;
use wallpaper_core::{
    Autostart, Backdrop, BackdropMode, Capabilities, Category, Desktop, DesktopError, Http, Item,
    ListRequest, LocalTime, Page, Picture, Provider, ProviderError, ProviderInfo, RateLimit, Ratio,
    Screen, Size, Sort, System,
};
use wallpaper_desktop::{App, AppWindow, Config};
use wallpaper_host::DiskCache;

const WAIT: Duration = Duration::from_secs(10);

static INFO: ProviderInfo = ProviderInfo {
    id: "fake",
    name: "Fake",
    capabilities: Capabilities {
        search: false,
        sorts: &[Sort::Top, Sort::Newest],
        categories: &[Category::General, Category::Anime],
        ratios: &[Ratio(16, 9)],
        colors: &[],
        min_size: true,
    },
    rate: RateLimit {
        requests: 1000,
        per: Duration::from_secs(1),
    },
};

struct Fake {
    offline: AtomicBool,
    asked: AtomicUsize,
}

fn url(id: u32) -> String {
    format!("https://picture.invalid/{id}")
}

impl Provider for Fake {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn list(&self, _: &ListRequest, _: &dyn Http) -> Result<Page, ProviderError> {
        self.asked.fetch_add(1, Ordering::Relaxed);
        if self.offline.load(Ordering::Relaxed) {
            return Err(ProviderError::Network("down".into()));
        }
        let items = (0..30).map(|id| Item {
            provider: "fake",
            id: id.to_string(),
            size: Size::new(320, 180),
            image_url: url(id),
            page_url: String::new(),
            file_bytes: 0,
            mime: String::new(),
            colors: vec![],
            thumbs: vec![],
        });
        Ok(Page {
            items: items.collect(),
            next: None,
            total: Some(30),
        })
    }
}

struct FakeDesktop {
    shown: Mutex<Backdrop>,
    changes: AtomicUsize,
    fails: AtomicBool,
}

impl FakeDesktop {
    fn path(&self) -> PathBuf {
        self.shown.lock().unwrap().pictures[0].path.clone()
    }

    fn changes(&self) -> usize {
        self.changes.load(Ordering::Relaxed)
    }

    fn go_to(&self, workspace: Option<&str>) {
        self.shown.lock().unwrap().workspace = workspace.map(Into::into);
    }
}

impl Desktop for FakeDesktop {
    fn screens(&self) -> Vec<Screen> {
        vec![Screen {
            id: "only".into(),
            size: Size::new(160, 90),
            primary: true,
        }]
    }

    fn workspace(&self) -> Option<String> {
        self.shown.lock().unwrap().workspace.clone()
    }

    fn read(&self) -> Result<Backdrop, DesktopError> {
        Ok(self.shown.lock().unwrap().clone())
    }

    fn apply(
        &self,
        pictures: &[(String, PathBuf)],
        workspace: Option<&str>,
    ) -> Result<(), DesktopError> {
        assert!(workspace.is_some(), "a change must know its workspace");
        assert_eq!(self.workspace().as_deref(), workspace);
        if self.fails.load(Ordering::Relaxed) {
            return Err(DesktopError::NotApplied);
        }
        assert_eq!(image::image_dimensions(&pictures[0].1).unwrap(), (160, 90));
        self.shown.lock().unwrap().pictures[0].path = pictures[0].1.clone();
        self.changes.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn restore(&self, backdrop: &Backdrop) -> Result<(), DesktopError> {
        assert_eq!(self.workspace(), backdrop.workspace);
        let path = backdrop.pictures[0].existing().unwrap().to_owned();
        self.shown.lock().unwrap().pictures[0].path = path;
        Ok(())
    }
}

struct FakeSystem {
    now: Mutex<LocalTime>,
    busy: AtomicBool,
    autostart: Mutex<Autostart>,
}

impl FakeSystem {
    fn set(&self, day: i64, hour: u32) {
        *self.now.lock().unwrap() = LocalTime {
            day,
            minute: hour * 60,
        };
    }
}

impl System for FakeSystem {
    fn now(&self) -> LocalTime {
        *self.now.lock().unwrap()
    }

    fn do_not_disturb(&self) -> bool {
        self.busy.load(Ordering::Relaxed)
    }

    fn autostart(&self) -> Autostart {
        *self.autostart.lock().unwrap()
    }

    fn set_autostart(&self, on: bool) -> Result<Autostart, String> {
        let state = if on { Autostart::On } else { Autostart::Off };
        *self.autostart.lock().unwrap() = state;
        Ok(state)
    }
}

fn picture(id: u32) -> Vec<u8> {
    let picture = image::RgbImage::from_fn(320, 180, |x, y| {
        image::Rgb([(x + id * 9) as u8, y as u8, (id * 8) as u8])
    });
    let mut bytes = Bytes::new(vec![]);
    picture
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

fn files(dir: &Path) -> usize {
    fs::read_dir(dir).unwrap().count()
}

#[test]
fn a_new_wallpaper_every_day() {
    let mut stage = Stage::new(1280, 820, 1.0);
    let data = std::env::temp_dir().join(format!("wallpaper-daily-flow-{}", std::process::id()));
    let _ = fs::remove_dir_all(&data);
    let originals = DiskCache::open(data.join("originals")).unwrap();
    for id in 0..30 {
        originals.put(&url(id), &picture(id)).unwrap();
    }
    let mine = data.join("mine.jpg");
    fs::write(&mine, b"the wallpaper from before").unwrap();
    let provider = Arc::new(Fake {
        offline: AtomicBool::new(false),
        asked: AtomicUsize::new(0),
    });
    let desktop = Arc::new(FakeDesktop {
        shown: Mutex::new(Backdrop {
            mode: BackdropMode::Picture,
            placement: 4,
            workspace: Some("first".into()),
            pictures: vec![Picture {
                screen: "only".into(),
                path: mine.clone(),
                kept: None,
            }],
        }),
        changes: AtomicUsize::new(0),
        fails: AtomicBool::new(false),
    });
    let system = Arc::new(FakeSystem {
        now: Mutex::new(LocalTime { day: 0, minute: 0 }),
        busy: AtomicBool::new(false),
        autostart: Mutex::new(Autostart::Off),
    });
    system.set(100, 9);
    let app = App::new(Config {
        provider: provider.clone(),
        desktop: desktop.clone(),
        system: system.clone(),
        data_dir: Some(data.clone()),
    })
    .unwrap();
    let window: AppWindow = app.window().clone_strong();

    // Off by default: nothing is fetched and nothing changes.
    app.tick();
    stage.run_for(Duration::from_millis(200));
    assert_eq!(
        (desktop.changes(), provider.asked.load(Ordering::Relaxed)),
        (0, 0)
    );
    assert!(!window.get_daily().enabled);

    // Switching it on gives today's picture, and prepares the next ones.
    window.invoke_toggle_daily();
    let count = |n| {
        let desktop = desktop.clone();
        move || desktop.changes() == n
    };
    assert!(
        stage.run_until(WAIT, count(1)),
        "{}",
        window.get_daily().status
    );
    let queue = data.join("daily").join("queue");
    let full = || {
        !window.get_changing()
            && files(&queue) == 3
            && window
                .get_daily()
                .status
                .contains("Đã đổi hôm nay. Có sẵn 3 ảnh")
    };
    assert!(stage.run_until(WAIT, full), "{}", window.get_daily().status);

    // Only once a day, and not before the hour.
    for (day, hour) in [(100, 10), (100, 23), (101, 0), (101, 7)] {
        system.set(day, hour);
        app.tick();
        stage.run_for(Duration::from_millis(60));
    }
    assert_eq!(desktop.changes(), 1);
    system.set(101, 8);
    app.tick();
    assert!(stage.run_until(WAIT, count(2)));
    assert!(stage.run_until(WAIT, full));

    // A week without the computer is one change, not seven. No network is needed for it.
    provider.offline.store(true, Ordering::Relaxed);
    system.set(108, 12);
    app.tick();
    assert!(stage.run_until(WAIT, count(3)));
    app.tick();
    stage.run_for(Duration::from_millis(300));
    assert_eq!(desktop.changes(), 3);
    assert!(
        window.get_daily().status.contains("Có sẵn 2 ảnh"),
        "{}",
        window.get_daily().status
    );

    // Offline, the source is not asked again every minute.
    let asked = provider.asked.load(Ordering::Relaxed);
    for _ in 0..5 {
        app.tick();
        stage.run_for(Duration::from_millis(30));
    }
    assert_eq!(provider.asked.load(Ordering::Relaxed), asked);

    // Not during a game, and not on a workspace that cannot be told. Later, yes.
    system.set(109, 9);
    system.busy.store(true, Ordering::Relaxed);
    app.tick();
    stage.run_for(Duration::from_millis(150));
    assert_eq!(desktop.changes(), 3);
    system.busy.store(false, Ordering::Relaxed);
    desktop.go_to(None);
    app.tick();
    stage.run_for(Duration::from_millis(150));
    assert_eq!(desktop.changes(), 3);
    desktop.go_to(Some("first"));
    app.tick();
    assert!(stage.run_until(WAIT, count(4)));

    // Paused, the day passes. "Next" still works.
    window.invoke_toggle_paused();
    system.set(110, 9);
    app.tick();
    stage.run_for(Duration::from_millis(150));
    assert_eq!(desktop.changes(), 4);
    window.invoke_next_daily();
    assert!(stage.run_until(WAIT, count(5)));
    window.invoke_toggle_paused();
    stage.run_for(Duration::from_millis(150));
    assert_eq!(
        desktop.changes(),
        5,
        "asking for the next one counts for the day"
    );

    // The queue is empty now: "next" waits for a picture instead of failing.
    stage.run_until(WAIT, || !window.get_changing());
    assert_eq!(files(&queue), 0);
    window.invoke_next_daily();
    stage.run_for(Duration::from_millis(200));
    assert_eq!(desktop.changes(), 5);
    assert!(window.get_daily().problem, "{}", window.get_daily().status);
    provider.offline.store(false, Ordering::Relaxed);
    window.invoke_next_daily();
    assert!(
        stage.run_until(WAIT, count(6)),
        "{}",
        window.get_daily().status
    );
    assert!(stage.run_until(WAIT, full));

    // "Never again" moves on, and only works for pictures from here.
    let before = desktop.path();
    window.invoke_block_daily();
    assert!(stage.run_until(WAIT, count(7)));
    assert_ne!(desktop.path(), before);
    stage.run_until(WAIT, || !window.get_changing());
    let ours = desktop.path();
    desktop.shown.lock().unwrap().pictures[0].path = mine.clone();
    window.invoke_block_daily();
    stage.run_for(Duration::from_millis(150));
    assert_eq!(desktop.changes(), 7);
    assert!(
        window.get_toast().contains("không phải"),
        "{}",
        window.get_toast()
    );
    desktop.shown.lock().unwrap().pictures[0].path = ours;

    // Undo goes back, and does not make the day due again.
    assert!(window.get_can_undo());
    window.invoke_undo();
    stage.run_until(WAIT, || !window.get_changing());
    app.tick();
    stage.run_for(Duration::from_millis(150));
    assert_eq!(desktop.changes(), 7);

    // A locked wallpaper is not retried every minute, but the next day is not lost.
    assert!(stage.run_until(WAIT, full));
    desktop.fails.store(true, Ordering::Relaxed);
    system.set(111, 9);
    app.tick();
    stage.run_until(WAIT, || window.get_toast().contains("không nhận"));
    desktop.fails.store(false, Ordering::Relaxed);
    app.tick();
    stage.run_for(Duration::from_millis(150));
    assert_eq!(desktop.changes(), 7);
    assert!(
        window.get_daily().status.contains("Sẽ đổi"),
        "{}",
        window.get_daily().status
    );
    assert_eq!(
        files(&queue),
        3,
        "a failed change must not use up a picture"
    );
    // Changing a setting tries again at once.
    window.invoke_shift_hour(-1);
    assert!(stage.run_until(WAIT, count(8)));
    assert_eq!(window.get_daily().hour, "07:00");

    // The hour wraps around, filters drop what was prepared, autostart is a switch.
    for _ in 0..8 {
        window.invoke_shift_hour(-1);
    }
    assert_eq!(window.get_daily().hour, "23:00");
    stage.run_until(WAIT, || !window.get_changing());
    let asked = provider.asked.load(Ordering::Relaxed);
    window.invoke_pick_daily_sort(1);
    assert!(stage.run_until(WAIT, || provider.asked.load(Ordering::Relaxed) > asked));
    assert!(!window.get_daily().autostart);
    window.invoke_toggle_autostart();
    assert!(window.get_daily().autostart);
    window.invoke_toggle_autostart();
    assert!(!window.get_daily().autostart);

    // Closing keeps running, and the window comes back with its pictures.
    app.open().unwrap();
    let rows = || slint::Model::row_count(&window.get_rows());
    assert!(stage.run_until(WAIT, || rows() > 0));
    app.close();
    assert_eq!(rows(), 0);
    app.open().unwrap();
    assert!(stage.run_until(WAIT, || rows() > 0));
    assert!(window.get_daily().enabled);

    drop(app);
    let _ = fs::remove_dir_all(&data);
}
