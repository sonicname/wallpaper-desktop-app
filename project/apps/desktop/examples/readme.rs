//! Takes the pictures in the README. The UI is real; the wallpapers are drawn here, because the
//! ones from the sources belong to their authors. Touches neither the network nor the desktop.
//!
//! ```text
//! cargo run --release --example readme -- <folder> [folder for more states] [width height scale]
//! ```

#[path = "support/stage.rs"]
mod stage;

use std::{
    env, fs,
    io::Cursor as Bytes,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use image::{ImageFormat, RgbImage};
use slint::{ComponentHandle, Model};
use stage::Stage;
use wallpaper_core::{
    Autostart, Backdrop, BackdropMode, Desktop, DesktopError, Http, Item, ListRequest, LocalTime,
    Page, Picture, Provider, ProviderError, ProviderInfo, Rgb, Screen, Size, System, Thumb,
    ThumbCrop,
};
use wallpaper_desktop::{App, Config};
use wallpaper_host::DiskCache;
use wallpaper_provider_wallhaven::Wallhaven;

const COUNT: u32 = 24;
const FULL: Size = Size::new(2560, 1440);
const SMALL: Size = Size::new(640, 360);
const WAIT: Duration = Duration::from_secs(60);

type Color = [f32; 3];

/// Sky at the top, sky at the horizon, sun, far hills, near hills.
const PALETTES: [[Color; 5]; 8] = [
    [
        [44., 62., 110.],
        [246., 178., 125.],
        [255., 232., 190.],
        [150., 104., 116.],
        [38., 30., 52.],
    ],
    [
        [16., 22., 48.],
        [58., 82., 132.],
        [232., 238., 250.],
        [44., 62., 100.],
        [8., 12., 26.],
    ],
    [
        [98., 170., 214.],
        [222., 240., 240.],
        [255., 250., 222.],
        [120., 172., 160.],
        [24., 66., 62.],
    ],
    [
        [92., 40., 92.],
        [250., 120., 102.],
        [255., 214., 140.],
        [140., 60., 96.],
        [40., 16., 46.],
    ],
    [
        [24., 70., 82.],
        [128., 196., 178.],
        [246., 252., 226.],
        [62., 128., 120.],
        [10., 36., 42.],
    ],
    [
        [120., 110., 160.],
        [246., 204., 214.],
        [255., 246., 236.],
        [160., 130., 168.],
        [62., 48., 86.],
    ],
    [
        [30., 20., 60.],
        [120., 60., 150.],
        [250., 190., 240.],
        [76., 40., 112.],
        [18., 10., 36.],
    ],
    [
        [206., 140., 70.],
        [252., 222., 160.],
        [255., 250., 226.],
        [190., 130., 80.],
        [86., 50., 30.],
    ],
];

fn mix(a: Color, b: Color, share: f32) -> Color {
    let share = share.clamp(0.0, 1.0);
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * share)
}

/// A repeatable number from 0 to 1.
fn noise(seed: u32) -> f32 {
    let mut value = seed.wrapping_mul(0x9e37_79b9) ^ 0x5bd1_e995;
    value = (value ^ (value >> 15)).wrapping_mul(0x85eb_ca6b);
    (value ^ (value >> 13)) as f32 / u32::MAX as f32
}

fn palette(id: u32) -> &'static [Color; 5] {
    &PALETTES[(id as usize * 3 + id as usize / 8) % PALETTES.len()]
}

/// Hills under a sky, different for every `id`.
fn scene(id: u32, size: Size) -> Vec<u8> {
    let [top, horizon, sun, far, near] = *palette(id);
    let night = top.iter().sum::<f32>() < 140.0;
    let vary = |n: u32| noise(id * 64 + n);
    let layers = 4 + (id % 3) as usize;
    let sun_at = (0.18 + 0.64 * vary(1), 0.2 + 0.3 * vary(2));
    let sun_radius = 0.05 + 0.06 * vary(3);
    let ridge = |layer: usize, x: f32| {
        let n = layer as u32 * 8 + 8;
        let base = 0.5 + 0.48 * layer as f32 / layers as f32;
        let wave = |f: f32, p: f32| (x * f * std::f32::consts::TAU + p * 7.0).sin();
        let height = 0.085 * wave(0.7 + vary(n), vary(n + 1))
            + 0.045 * wave(1.9 + vary(n + 2), vary(n + 3))
            + 0.014 * wave(5.0 + 3.0 * vary(n + 4), vary(n + 5));
        base + height * (1.0 - 0.35 * layer as f32 / layers as f32)
    };
    let picture = RgbImage::from_fn(size.width, size.height, |px, py| {
        let aspect = size.aspect();
        let (x, y) = (
            px as f32 / size.width as f32,
            py as f32 / size.height as f32,
        );
        let mut color = mix(top, horizon, (y / 0.62).powf(1.2));
        let distance = ((x - sun_at.0) * aspect).hypot(y - sun_at.1);
        color = mix(color, sun, 0.5 * (-distance * 5.0).exp());
        let edge = 1.5 / size.height as f32;
        color = mix(color, sun, (sun_radius - distance) / edge + 0.5);
        if night {
            // One star in some cells of a fixed grid, so that every size shows the same sky.
            let (cx, cy) = ((x * 64.0) as u32, (y * 36.0) as u32);
            let star = cy * 64 + cx + id * 4096;
            if noise(star) > 0.93 {
                let at = (
                    (cx as f32 + 0.2 + 0.6 * noise(star ^ 0xaaaa)) / 64.0,
                    (cy as f32 + 0.2 + 0.6 * noise(star ^ 0x5555)) / 36.0,
                );
                let off = ((x - at.0) * aspect).hypot(y - at.1);
                color = mix(color, [255.0; 3], (0.0016 - off) / edge + 0.4);
            }
        }
        for layer in 0..layers {
            let depth = layer as f32 / (layers - 1) as f32;
            let top = ridge(layer, x);
            // Mist at the foot of every ridge.
            let misty = mix(
                mix(far, near, depth),
                horizon,
                0.3 * (1.0 - depth) + (y - top) * 0.5,
            );
            color = mix(color, misty, (y - top) / edge + 0.5);
        }
        image::Rgb(color.map(|c| c.round().clamp(0.0, 255.0) as u8))
    });
    let mut bytes = Bytes::new(vec![]);
    picture.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

struct Drawn {
    info: ProviderInfo,
    items: Vec<Item>,
}

impl Provider for Drawn {
    fn info(&self) -> &ProviderInfo {
        &self.info
    }

    fn list(&self, _: &ListRequest, _: &dyn Http) -> Result<Page, ProviderError> {
        Ok(Page {
            items: self.items.clone(),
            next: None,
            total: Some(COUNT.into()),
        })
    }
}

fn url(kind: &str, id: u32) -> String {
    format!("https://drawn.invalid/{kind}/{id}")
}

struct MemoryDesktop(Mutex<Backdrop>);

impl Desktop for MemoryDesktop {
    fn screens(&self) -> Vec<Screen> {
        vec![Screen {
            id: "screen".into(),
            size: FULL,
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

struct Morning;

impl System for Morning {
    fn now(&self) -> LocalTime {
        LocalTime::new(2026, 11, 25, 9, 0)
    }

    fn do_not_disturb(&self) -> bool {
        false
    }

    fn autostart(&self) -> Autostart {
        Autostart::On
    }

    fn pick_folder(&self) -> Option<PathBuf> {
        None
    }

    fn set_autostart(&self, _: bool) -> Result<Autostart, String> {
        Ok(Autostart::On)
    }
}

fn save(stage: &Stage, path: &Path) {
    let rgb: Vec<u8> = stage
        .buffer
        .iter()
        .flat_map(|p| [p.red, p.green, p.blue])
        .collect();
    let (width, height) = (stage.width, stage.height);
    image::save_buffer(path, &rgb, width, height, image::ColorType::Rgb8).unwrap();
    println!("saved {}", path.display());
}

fn main() {
    let folder = PathBuf::from(env::args().nth(1).expect("a folder for the pictures"));
    fs::create_dir_all(&folder).unwrap();
    let data = env::temp_dir().join(format!("wallpaper-readme-{}", std::process::id()));
    let thumbs = DiskCache::open(data.join("thumbs")).unwrap();
    let originals = DiskCache::open(data.join("originals")).unwrap();
    let items = (0..COUNT)
        .map(|id| {
            let full = scene(id, FULL);
            thumbs.put(&url("small", id), &scene(id, SMALL)).unwrap();
            originals.put(&url("full", id), &full).unwrap();
            Item {
                // The id of the provider below, which borrows the filters of Wallhaven.
                provider: Wallhaven.info().id.into(),
                id: format!("{id:03}"),
                size: FULL,
                image_url: url("full", id),
                page_url: String::new(),
                file_bytes: full.len() as u64,
                mime: "image/png".into(),
                colors: palette(id)
                    .map(|c| c.map(|v| v as u8))
                    .map(|[r, g, b]| Rgb(r, g, b))
                    .into(),
                thumbs: vec![Thumb {
                    url: url("small", id),
                    size: SMALL,
                    crop: ThumbCrop::Whole,
                }],
            }
        })
        .collect();
    let info = ProviderInfo {
        name: "Minh họa",
        ..*Wallhaven.info()
    };

    let more = env::args().nth(2).map(PathBuf::from);
    let number = |at: usize, default: f32| {
        let given = env::args().nth(at);
        given.map_or(default, |text| text.parse().expect("a number"))
    };
    let mut stage = Stage::new(
        number(3, 1920.0) as u32,
        number(4, 1200.0) as u32,
        number(5, 1.5),
    );
    let app = App::new(Config {
        providers: vec![Arc::new(Drawn { info, items })],
        desktop: Arc::new(MemoryDesktop(Mutex::new(Backdrop {
            mode: BackdropMode::Picture,
            placement: 4,
            workspace: Some("one".into()),
            pictures: vec![Picture {
                screen: "screen".into(),
                path: PathBuf::new(),
                kept: None,
            }],
        }))),
        system: Arc::new(Morning),
        data_dir: Some(data.clone()),
    })
    .unwrap();
    let window = app.window().clone_strong();
    app.open().unwrap();
    let loaded = || {
        let rows = window.get_rows();
        rows.row_count() >= 4
            && (0..4).all(|r| rows.row_data(r).unwrap().cells.iter().all(|c| c.ready))
    };
    assert!(stage.run_until(WAIT, loaded));
    for favorite in [1, 5, 6] {
        window.invoke_open(favorite);
        window.invoke_toggle_favorite();
        assert!(stage.run_until(WAIT, || window.get_detail().favorite));
    }
    window.set_detail_open(false);
    stage.run_for(Duration::from_millis(500));
    save(&stage, &folder.join("browse.png"));

    window.invoke_open(1);
    assert!(stage.run_until(WAIT, || window.get_picture_sharp()));
    stage.run_for(Duration::from_millis(500));
    save(&stage, &folder.join("preview.png"));
    window.set_detail_open(false);

    window.invoke_toggle_daily();
    let prepared = || {
        window
            .get_daily()
            .status
            .contains("Đã đổi hôm nay. Có sẵn 3 ảnh")
    };
    assert!(
        stage.run_until(WAIT, prepared),
        "{}",
        window.get_daily().status
    );
    window.invoke_close_toast();
    window.invoke_open_daily();
    stage.run_for(Duration::from_millis(500));
    save(&stage, &folder.join("daily.png"));

    if let Some(more) = more {
        fs::create_dir_all(&more).unwrap();
        window.set_daily_open(false);
        // The pointer over a picture, then its button to set it.
        let position = slint::LogicalPosition::new(700.0, 420.0);
        stage
            .window
            .dispatch_event(slint::platform::WindowEvent::PointerMoved { position });
        stage.run_for(Duration::from_millis(400));
        save(&stage, &more.join("hover.png"));
        window.invoke_apply_at(4);
        assert!(stage.run_until(WAIT, || !window.get_changing() && window.get_toast() != ""));
        stage.run_for(Duration::from_millis(300));
        save(&stage, &more.join("toast.png"));
        stage
            .window
            .dispatch_event(slint::platform::WindowEvent::PointerExited);
        window.invoke_close_toast();
        window.invoke_pick_source(1);
        assert!(stage.run_until(WAIT, || !window.get_busy()
            && window.get_rows().row_count() > 0));
        stage.run_for(Duration::from_millis(800));
        save(&stage, &more.join("favorites.png"));
        window.invoke_pick_source(2);
        assert!(stage.run_until(WAIT, || window.get_needs_folder() && !window.get_busy()));
        stage.run_for(Duration::from_millis(300));
        save(&stage, &more.join("folder.png"));
    }

    drop(app);
    let _ = fs::remove_dir_all(&data);
}
