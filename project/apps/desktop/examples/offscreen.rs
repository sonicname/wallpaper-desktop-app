//! Runs the real UI with live data but draws into memory, so no window ever appears.
//! Only the `wallpaper` mode changes the desktop, and it undoes the change after some seconds.
//!
//! ```text
//! cargo run --release --example offscreen -- shots <folder> [width height scale]
//! cargo run --release --example offscreen -- bench <width> <height> <scale> [last-frame.png]
//! cargo run --release --example offscreen -- wallpaper <seconds>
//! ```

#[path = "support/stage.rs"]
mod stage;

use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use slint::{ComponentHandle, LogicalPosition, Model, platform::WindowEvent};
use stage::Stage;
use wallpaper_desktop::{App, AppWindow, Config};
use wallpaper_platform_windows::{WindowsDesktop, WindowsSystem};
use wallpaper_provider_wallhaven::Wallhaven;

fn save(stage: &Stage, path: &Path) {
    let rgb: Vec<u8> = stage
        .buffer
        .iter()
        .flat_map(|p| [p.red, p.green, p.blue])
        .collect();
    image::save_buffer(
        path,
        &rgb,
        stage.width,
        stage.height,
        image::ColorType::Rgb8,
    )
    .unwrap();
    println!("saved {}", path.display());
}

/// Visible rows whose pictures have all arrived.
fn ready(window: &AppWindow, rows: usize) -> bool {
    let all = window.get_rows();
    all.row_count() >= rows
        && (0..rows).all(|r| all.row_data(r).unwrap().cells.iter().all(|c| c.ready))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let number = |at: usize, default: f32| {
        args.get(at)
            .map_or(default, |text| text.parse().expect("a number"))
    };
    let mode = args.first().map_or("", String::as_str);
    // Where the window size starts. The wallpaper mode keeps the default one.
    let first = match mode {
        "bench" => 1,
        "shots" => 2,
        "wallpaper" => args.len(),
        _ => panic!("the mode must be shots, bench or wallpaper"),
    };
    let (width, height) = (
        number(first, 1280.0) as u32,
        number(first + 1, 820.0) as u32,
    );
    let scale = number(first + 2, 1.0);

    let mut stage = Stage::new(width, height, scale);
    let app = App::new(Config {
        provider: Arc::new(Wallhaven),
        desktop: Arc::new(WindowsDesktop),
        // Never added to the startup list, because nothing here switches it on.
        system: Arc::new(WindowsSystem::new("WallpaperOffscreen", String::new())),
        data_dir: Some(env::temp_dir().join("wallpaper-offscreen")),
    })
    .unwrap();
    let window = app.window().clone_strong();
    app.open().unwrap();
    let visible = (height as f32 / scale / 200.0).ceil() as usize;
    let loaded = stage.run_until(Duration::from_secs(60), || ready(&window, visible));
    println!(
        "{} rows, first screen ready: {loaded}",
        window.get_rows().row_count()
    );

    match mode {
        "bench" => {
            run_bench(&mut stage, &window, scale);
            if let Some(path) = args.get(4) {
                save(&stage, Path::new(path));
            }
        }
        "wallpaper" => run_wallpaper(&mut stage, &window, number(1, 10.0)),
        _ => {
            let folder = PathBuf::from(args.get(1).expect("a folder for the pictures"));
            std::fs::create_dir_all(&folder).unwrap();
            run_shots(&mut stage, &window, &folder);
        }
    }
}

fn run_wallpaper(stage: &mut Stage, window: &AppWindow, seconds: f32) {
    window.invoke_open(1);
    stage.run_until(Duration::from_secs(40), || window.get_picture_sharp());
    for (name, change) in [
        ("set", AppWindow::invoke_apply as fn(&AppWindow)),
        ("undone", AppWindow::invoke_undo),
    ] {
        let start = Instant::now();
        change(window);
        stage.run_until(Duration::from_secs(60), || !window.get_changing());
        println!("{name} after {:?}: {}", start.elapsed(), window.get_toast());
        stage.run_for(Duration::from_secs_f32(seconds));
    }
}

fn run_shots(stage: &mut Stage, window: &AppWindow, folder: &Path) {
    stage.run_for(Duration::from_millis(600));
    save(stage, &folder.join("grid.png"));

    let position = LogicalPosition::new(120.0, 260.0);
    stage
        .window
        .dispatch_event(WindowEvent::PointerMoved { position });
    stage.run_for(Duration::from_millis(400));
    save(stage, &folder.join("hover.png"));

    window.invoke_open(1);
    let sharp = stage.run_until(Duration::from_secs(40), || window.get_picture_sharp());
    println!("preview ready: {sharp}");
    stage.run_for(Duration::from_millis(300));
    save(stage, &folder.join("detail.png"));

    // States that depend on the machine, filled in by hand.
    let option = |label: &str, on| wallpaper_desktop::Option {
        label: label.into(),
        on,
    };
    let targets = [
        option("Tất cả màn hình", true),
        option("Màn 1 (2560 × 1440)", false),
        option("Màn 2 (1440 × 2560)", false),
    ];
    window.set_targets(slint::ModelRc::from(targets));
    window.set_toast("Đã đặt làm hình nền.".into());
    window.set_can_undo(true);
    stage.run_for(Duration::from_millis(300));
    save(stage, &folder.join("notice.png"));
    window.set_detail_open(false);
    stage.run_for(Duration::from_millis(300));
    save(stage, &folder.join("undo.png"));
    window.set_toast("".into());

    // Filled in by hand: switching it on for real would change the wallpaper.
    let options = |labels: &[&str], on: usize| {
        let all: Vec<_> = labels
            .iter()
            .enumerate()
            .map(|(i, l)| option(l, i == on))
            .collect();
        slint::ModelRc::new(slint::VecModel::from(all))
    };
    window.set_daily(wallpaper_desktop::DailyState {
        enabled: true,
        paused: false,
        hour: "08:00".into(),
        status: "Đã đổi hôm nay. Có sẵn 3 ảnh.".into(),
        problem: false,
        autostart: true,
        autostart_note: "App chạy ẩn dưới khay sau khi đăng nhập.".into(),
        sorts: options(&["Ngẫu nhiên", "Nổi bật", "Mới nhất", "Yêu thích nhất"], 1),
        categories: options(&["Tổng hợp", "Anime", "Con người"], 0),
    });
    window.set_daily_open(true);
    stage.run_for(Duration::from_millis(300));
    save(stage, &folder.join("daily.png"));
    let mut off = window.get_daily();
    off.enabled = false;
    off.autostart = false;
    off.autostart_note = "Cần bật tự đổi mỗi ngày trước.".into();
    window.set_daily(off);
    stage.run_for(Duration::from_millis(300));
    save(stage, &folder.join("daily-off.png"));
    window.set_daily_open(false);

    window.invoke_search("zzqqxxyyzz no such picture".into());
    stage.run_until(Duration::from_secs(40), || !window.get_busy());
    stage.run_for(Duration::from_millis(300));
    save(stage, &folder.join("empty.png"));
}

fn run_bench(stage: &mut Stage, window: &AppWindow, scale: f32) {
    // Scroll once to load pages and fill the disk cache, so that the network is not measured.
    let distance = 6000.0;
    for step in 0..300 {
        window.set_scroll(-distance * step as f32 / 300.0);
        stage.frame();
        thread::sleep(Duration::from_millis(16));
    }
    stage.run_for(Duration::from_secs(3));
    window.set_scroll(0.0);
    stage.run_for(Duration::from_secs(2));

    // 10 physical pixels per frame, like the Slint and Tauri prototypes at 144 Hz.
    let mut times = vec![];
    for step in 0..600 {
        window.set_scroll(-(step as f32) * 10.0 / scale);
        if let Some(time) = stage.frame() {
            times.push(time.as_secs_f64() * 1e3);
        }
        thread::sleep(Duration::from_millis(4));
    }
    times.sort_by(f64::total_cmp);
    let at = |share: f64| times[((times.len() - 1) as f64 * share) as usize];
    println!(
        "{}x{} scale {scale}: {} frames, median {:.2} ms, p95 {:.2} ms, p99 {:.2} ms, max {:.2} ms",
        stage.width,
        stage.height,
        times.len(),
        at(0.5),
        at(0.95),
        at(0.99),
        at(1.0)
    );
}
