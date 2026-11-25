//! Drives the real window against a scripted source. Slint allows one platform per process,
//! so the steps share a single test.

#[path = "../examples/support/stage.rs"]
mod stage;

use std::{
    fs,
    io::Cursor as Bytes,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use slint::{
    ComponentHandle, Model,
    platform::{Key, WindowEvent},
};
use stage::Stage;
use wallpaper_core::{
    Backdrop, BackdropMode, Capabilities, Category, Cursor, Desktop, DesktopError, Http, Item,
    ListRequest, Page, Picture, Provider, ProviderError, ProviderInfo, RateLimit, Ratio, Rgb,
    Screen, Size, Sort,
};
use wallpaper_desktop::{App, AppWindow, Config};
use wallpaper_host::DiskCache;

static INFO: ProviderInfo = ProviderInfo {
    id: "fake",
    name: "Fake",
    capabilities: Capabilities {
        search: true,
        sorts: &[Sort::Random, Sort::Newest],
        categories: &[Category::General, Category::Anime],
        ratios: &[Ratio(16, 9), Ratio(9, 16)],
        colors: &[Rgb(255, 0, 0)],
        min_size: true,
    },
    rate: RateLimit {
        requests: 1000,
        per: Duration::from_secs(1),
    },
};

type Answer = dyn Fn(&ListRequest) -> Result<Page, ProviderError> + Send + Sync;

struct Fake {
    answer: Mutex<Arc<Answer>>,
    asked: Mutex<Vec<ListRequest>>,
}

impl Fake {
    fn answers(
        &self,
        f: impl Fn(&ListRequest) -> Result<Page, ProviderError> + Send + Sync + 'static,
    ) {
        *self.answer.lock().unwrap() = Arc::new(f);
    }

    fn count(&self) -> usize {
        self.asked.lock().unwrap().len()
    }

    fn last(&self) -> ListRequest {
        self.asked.lock().unwrap().last().unwrap().clone()
    }
}

impl Provider for Fake {
    fn info(&self) -> &ProviderInfo {
        &INFO
    }

    fn list(&self, request: &ListRequest, _: &dyn Http) -> Result<Page, ProviderError> {
        self.asked.lock().unwrap().push(request.clone());
        let answer = self.answer.lock().unwrap().clone();
        answer(request)
    }
}

/// A landscape and a portrait screen, and two workspaces with pictures of their own.
struct FakeDesktop {
    front: Mutex<usize>,
    shown: Mutex<[Backdrop; 2]>,
    /// Switches to the other workspace in the middle of the next change.
    switches: AtomicBool,
    fails: AtomicBool,
}

impl FakeDesktop {
    fn new(picture: &Path) -> Self {
        let pictures = ["left", "right"].map(|screen| Picture {
            screen: screen.into(),
            path: picture.into(),
            kept: None,
        });
        let backdrop = |workspace: &str| Backdrop {
            mode: BackdropMode::Picture,
            placement: 2,
            workspace: Some(workspace.into()),
            pictures: pictures.clone().into(),
        };
        Self {
            front: Mutex::new(0),
            shown: Mutex::new([backdrop("first"), backdrop("second")]),
            switches: AtomicBool::new(false),
            fails: AtomicBool::new(false),
        }
    }

    fn switch_to(&self, workspace: usize) {
        *self.front.lock().unwrap() = workspace;
    }

    fn path(&self, screen: usize) -> PathBuf {
        self.path_on(*self.front.lock().unwrap(), screen)
    }

    fn path_on(&self, workspace: usize, screen: usize) -> PathBuf {
        self.shown.lock().unwrap()[workspace].pictures[screen]
            .path
            .clone()
    }

    fn edit(&self, change: impl FnOnce(&mut Backdrop)) {
        change(&mut self.shown.lock().unwrap()[*self.front.lock().unwrap()]);
    }

    fn set(&self, screen: &str, path: &Path) {
        self.edit(|shown| {
            let mut pictures = shown.pictures.iter_mut();
            pictures.find(|p| p.screen == screen).unwrap().path = path.into();
        });
    }

    fn require(&self, workspace: Option<&str>) -> Result<(), DesktopError> {
        if self.workspace().as_deref() == workspace {
            return Ok(());
        }
        Err(DesktopError::OtherWorkspace)
    }
}

impl Desktop for FakeDesktop {
    fn screens(&self) -> Vec<Screen> {
        let screen = |id: &str, width, height, primary| Screen {
            id: id.into(),
            size: Size::new(width, height),
            primary,
        };
        vec![
            screen("left", 2560, 1440, true),
            screen("right", 1440, 2560, false),
        ]
    }

    fn workspace(&self) -> Option<String> {
        self.read().unwrap().workspace
    }

    fn read(&self) -> Result<Backdrop, DesktopError> {
        let front = *self.front.lock().unwrap();
        Ok(self.shown.lock().unwrap()[front].clone())
    }

    fn apply(
        &self,
        pictures: &[(String, PathBuf)],
        workspace: Option<&str>,
    ) -> Result<(), DesktopError> {
        self.require(workspace)?;
        for (screen, path) in pictures {
            assert!(path.is_file());
            self.set(screen, path);
            if self.fails.load(Ordering::Relaxed) {
                return Err(DesktopError::NotApplied);
            }
        }
        self.edit(|shown| shown.placement = 4);
        if self.switches.swap(false, Ordering::Relaxed) {
            self.switch_to(1);
            return Err(DesktopError::WorkspaceSwitched);
        }
        Ok(())
    }

    fn restore(&self, backdrop: &Backdrop) -> Result<(), DesktopError> {
        self.require(backdrop.workspace.as_deref())?;
        for picture in &backdrop.pictures {
            let path = picture
                .existing()
                .ok_or(DesktopError::Missing(picture.path.clone()))?;
            self.set(&picture.screen, path);
        }
        self.edit(|shown| shown.placement = backdrop.placement);
        Ok(())
    }
}

fn original_url(id: &str) -> String {
    format!("https://picture.invalid/{id}")
}

/// A picture with a red left half and a blue right half.
fn two_halves() -> Vec<u8> {
    let picture = image::RgbImage::from_fn(640, 360, |x, _| {
        image::Rgb(if x < 320 { [255, 0, 0] } else { [0, 0, 255] })
    });
    let mut bytes = Bytes::new(vec![]);
    picture
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

fn size_of(path: &Path) -> (u32, u32) {
    image::image_dimensions(path).unwrap()
}

fn items(ids: std::ops::Range<u32>) -> Vec<Item> {
    ids.map(|id| Item {
        provider: "fake",
        id: id.to_string(),
        size: Size::new(2560, 1440),
        image_url: original_url(&id.to_string()),
        page_url: String::new(),
        file_bytes: 1,
        mime: "image/jpeg".into(),
        colors: vec![],
        thumbs: vec![],
    })
    .collect()
}

fn cells(window: &AppWindow) -> usize {
    window
        .get_rows()
        .iter()
        .map(|row| row.cells.row_count())
        .sum()
}

fn on(options: slint::ModelRc<wallpaper_desktop::Option>) -> Vec<bool> {
    options.iter().map(|o| o.on).collect()
}

#[test]
fn browsing() {
    const WAIT: Duration = Duration::from_secs(5);
    let mut stage = Stage::new(1280, 820, 1.0);
    let fake = Arc::new(Fake {
        answer: Mutex::new(Arc::new(|_| Err(ProviderError::Network("down".into())))),
        asked: Mutex::default(),
    });
    let data = std::env::temp_dir().join(format!("wallpaper-flow-{}", std::process::id()));
    let _ = fs::remove_dir_all(&data);
    let originals = DiskCache::open(data.join("originals")).unwrap();
    let mine = data.join("mine.jpg");
    fs::write(&mine, b"the wallpaper from before").unwrap();
    let desktop = Arc::new(FakeDesktop::new(&mine));
    let app = App::new(Config {
        provider: fake.clone(),
        desktop: desktop.clone(),
        data_dir: Some(data.clone()),
    })
    .unwrap();
    let window = app.window().clone_strong();
    window.show().unwrap();

    // The first request already asks for pictures that fit the screen.
    assert!(
        stage.run_until(WAIT, || window.get_can_retry()),
        "the error must be shown"
    );
    assert!(window.get_message().contains("Không kết nối được"));
    assert!(!window.get_busy());
    assert_eq!(fake.last().min_size, Some(Size::new(2560, 1440)));
    assert_eq!(fake.last().ratios, vec![Ratio(16, 9)]);
    let after_error = fake.count();
    stage.run_for(Duration::from_millis(200));
    assert_eq!(
        fake.count(),
        after_error,
        "an error must not be retried on its own"
    );

    // Retry works, and a random listing that only repeats itself ends.
    fake.answers(|_| {
        Ok(Page {
            items: items(0..7),
            next: Some(Cursor("1".into())),
            total: Some(7),
        })
    });
    window.invoke_retry();
    assert!(
        stage.run_until(WAIT, || cells(&window) == 7),
        "{} cells",
        cells(&window)
    );
    stage.run_for(Duration::from_millis(300));
    assert_eq!(cells(&window), 7, "repeated pictures must not be added");
    assert!(!window.get_busy());
    assert_eq!(window.get_message(), "");
    let settled = fake.count();
    stage.run_for(Duration::from_millis(300));
    assert_eq!(
        fake.count(),
        settled,
        "a listing without news must stop loading"
    );

    // Paging follows the cursor of the source and stops at its end.
    fake.answers(|request| {
        let page: u32 = request.cursor.as_ref().map_or(1, |c| c.0.parse().unwrap());
        let next = (page < 3).then(|| Cursor((page + 1).to_string()));
        Ok(Page {
            items: items(page * 100..page * 100 + 4),
            next,
            total: Some(12),
        })
    });
    window.invoke_pick_sort(1);
    assert!(
        stage.run_until(WAIT, || cells(&window) == 12),
        "{} cells",
        cells(&window)
    );
    assert_eq!(fake.last().sort, Sort::Newest);
    assert_eq!(on(window.get_sorts()), [false, true]);

    // Filters change the request; the last category cannot be turned off.
    window.invoke_toggle_screen_fit();
    stage.run_until(WAIT, || cells(&window) == 12);
    assert_eq!((fake.last().min_size, fake.last().ratios.len()), (None, 0));
    window.invoke_pick_category(0);
    stage.run_until(WAIT, || cells(&window) == 12);
    assert_eq!(fake.last().categories, vec![Category::Anime]);
    let before = fake.count();
    window.invoke_pick_category(1);
    stage.run_for(Duration::from_millis(200));
    assert_eq!(fake.count(), before);
    assert_eq!(on(window.get_categories()), [false, true]);
    window.invoke_pick_category(0);
    stage.run_until(WAIT, || cells(&window) == 12);
    assert!(
        fake.last().categories.is_empty(),
        "every category on means no filter"
    );

    // A slow answer to an old search must not replace the new one.
    fake.answers(|request| {
        if request.text == "slow" {
            std::thread::sleep(Duration::from_millis(400));
            return Ok(Page {
                items: items(0..20),
                next: None,
                total: Some(20),
            });
        }
        Ok(Page {
            items: items(50..53),
            next: None,
            total: Some(3),
        })
    });
    window.invoke_search("slow".into());
    stage.run_for(Duration::from_millis(50));
    window.invoke_search("fast".into());
    stage.run_for(Duration::from_millis(800));
    assert_eq!(cells(&window), 3);
    assert_eq!(window.get_summary(), "3 ảnh");

    // Stepping through the preview stays inside the listing.
    window.invoke_open(2);
    assert!(window.get_detail_open());
    assert_eq!(window.get_detail().resolution, "2560 × 1440");
    assert!(window.get_detail().fit_good);
    window.invoke_step(1);
    assert!(window.get_detail().source.ends_with("52"));
    window.invoke_step(-1);
    assert!(window.get_detail().source.ends_with("51"));

    // Arrow keys and Escape work without a click first.
    let press = |stage: &mut Stage, key: Key| {
        let text = slint::SharedString::from(key);
        stage
            .window
            .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
        stage
            .window
            .dispatch_event(WindowEvent::KeyReleased { text });
        stage.run_for(Duration::from_millis(50));
    };
    stage.run_for(Duration::from_millis(50));
    press(&mut stage, Key::RightArrow);
    assert!(window.get_detail().source.ends_with("52"));
    press(&mut stage, Key::LeftArrow);
    assert!(window.get_detail().source.ends_with("51"));
    press(&mut stage, Key::Escape);
    assert!(!window.get_detail_open());

    // A picture that cannot be downloaded changes nothing.
    window.invoke_open(1);
    assert!(!window.get_can_undo());
    let toast = |window: &AppWindow| window.get_toast().to_string();
    window.invoke_apply();
    assert!(window.get_changing());
    assert!(stage.run_until(Duration::from_secs(30), || !window.get_changing()));
    assert!(
        toast(&window).contains("Không kết nối được"),
        "{}",
        toast(&window)
    );
    assert_eq!(desktop.path(0), mine);
    assert!(!window.get_can_undo());

    // Every screen gets its own crop, in its own size.
    originals.put(&original_url("51"), &two_halves()).unwrap();
    window.invoke_apply();
    assert!(stage.run_until(WAIT, || !window.get_changing()));
    assert_eq!(toast(&window), "Đã đặt làm hình nền.");
    assert_eq!(size_of(&desktop.path(0)), (2560, 1440));
    assert_eq!(size_of(&desktop.path(1)), (1440, 2560));
    assert!(window.get_can_undo());
    let first = desktop.path(0);

    // A single screen leaves the other one alone, and the preview follows it.
    assert_eq!(window.get_targets().row_count(), 3);
    window.invoke_pick_target(2);
    assert_eq!(on(window.get_targets()), [false, false, true]);
    assert_eq!(window.get_screen_name(), "1440 × 2560");
    assert!(
        !window.get_detail().fit_good,
        "a landscape picture does not fit a portrait screen"
    );
    originals
        .put(&original_url("52"), &two_halves()[..])
        .unwrap();
    window.invoke_step(1);
    window.invoke_apply();
    assert!(stage.run_until(WAIT, || !window.get_changing()));
    assert_eq!(desktop.path(0), first);

    // A change that fails half way is rolled back and leaves no undo step.
    window.invoke_pick_target(0);
    let portrait = desktop.path(1);
    desktop.fails.store(true, Ordering::Relaxed);
    window.invoke_apply();
    assert!(stage.run_until(WAIT, || !window.get_changing()));
    assert!(
        toast(&window).contains("Windows không nhận"),
        "{}",
        toast(&window)
    );
    assert_eq!(
        (desktop.path(0), desktop.path(1)),
        (first.clone(), portrait)
    );
    desktop.fails.store(false, Ordering::Relaxed);

    // Undo walks back to the very first wallpaper, then is gone.
    for expected in [&first, &mine] {
        assert!(window.get_can_undo());
        window.invoke_undo();
        assert!(stage.run_until(WAIT, || !window.get_changing()));
        assert_eq!(&desktop.path(0), expected);
    }
    assert!(!window.get_can_undo());
    assert_eq!(desktop.path(1), mine);
    assert_eq!(desktop.read().unwrap().placement, 2);
    let left: Vec<_> = fs::read_dir(data.join("pictures"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(left, ["history.json"], "unused pictures must be deleted");

    // Undo belongs to a workspace: on another one there is nothing to undo.
    let change = |stage: &mut Stage, action: fn(&AppWindow)| {
        action(&window);
        assert!(stage.run_until(WAIT, || !window.get_changing()));
    };
    change(&mut stage, AppWindow::invoke_apply);
    let on_first = desktop.path(0);
    assert_ne!(on_first, mine);
    desktop.switch_to(1);
    change(&mut stage, AppWindow::invoke_undo);
    assert!(
        toast(&window).contains("không còn gì"),
        "{}",
        toast(&window)
    );
    assert_eq!(desktop.path_on(0, 0), on_first);
    assert_eq!(desktop.path_on(1, 0), mine);

    // A change here keeps the picture the other workspace shows, and has its own way back.
    window.invoke_step(-1);
    change(&mut stage, AppWindow::invoke_apply);
    assert_ne!(desktop.path_on(1, 0), mine);
    assert!(on_first.is_file(), "the first workspace still shows it");
    change(&mut stage, AppWindow::invoke_undo);
    assert_eq!(desktop.path_on(1, 0), mine);
    assert_eq!(desktop.path_on(0, 0), on_first);
    desktop.switch_to(0);
    change(&mut stage, AppWindow::invoke_undo);
    assert_eq!(desktop.path_on(0, 0), mine);

    // Switching while the picture is prepared changes neither workspace.
    window.invoke_apply();
    desktop.switch_to(1);
    assert!(stage.run_until(WAIT, || !window.get_changing()));
    assert!(toast(&window).contains("chưa đổi"), "{}", toast(&window));
    assert_eq!(
        (desktop.path_on(0, 0), desktop.path_on(1, 0)),
        (mine.clone(), mine.clone())
    );

    // Switching in the middle of the change never touches the new one, and can be undone later.
    desktop.switch_to(0);
    desktop.switches.store(true, Ordering::Relaxed);
    change(&mut stage, AppWindow::invoke_apply);
    assert!(toast(&window).contains("quay lại"), "{}", toast(&window));
    assert_eq!(desktop.path_on(1, 0), mine);
    assert!(
        desktop.path_on(0, 0).is_file(),
        "its picture must not be deleted"
    );
    desktop.switch_to(0);
    change(&mut stage, AppWindow::invoke_undo);
    assert_eq!(desktop.path_on(0, 0), mine);

    // No result is explained.
    fake.answers(|_| Ok(Page::default()));
    window.set_detail_open(false);
    window.invoke_search("nothing".into());
    assert!(stage.run_until(WAIT, || window.get_message().contains("Không có ảnh nào")));
    assert!(!window.get_can_retry());
    let _ = fs::remove_dir_all(&data);
}
