//! Runs the real UI with live data but draws into memory, so no window ever appears.
//!
//! ```text
//! cargo run --release --example offscreen -- shots <folder> [width height scale]
//! cargo run --release --example offscreen -- bench <width> <height> <scale> [last-frame.png]
//! ```

#[path = "support/stage.rs"]
mod stage;

use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::Duration,
};

use slint::{ComponentHandle, LogicalPosition, Model, platform::WindowEvent};
use stage::Stage;
use wallpaper_core::Size;
use wallpaper_desktop::{App, AppWindow, Config};
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
    let bench = args.first().is_some_and(|mode| mode == "bench");
    let first = if bench { 1 } else { 2 };
    let (width, height) = (
        number(first, 1280.0) as u32,
        number(first + 1, 820.0) as u32,
    );
    let scale = number(first + 2, 1.0);

    let mut stage = Stage::new(width, height, scale);
    let app = App::new(Config {
        provider: Arc::new(Wallhaven),
        screen: Some(Size::new(2560, 1440)),
        cache_dir: Some(env::temp_dir().join("wallpaper-offscreen-thumbs")),
    })
    .unwrap();
    let window = app.window().clone_strong();
    window.show().unwrap();
    let visible = (height as f32 / scale / 200.0).ceil() as usize;
    let loaded = stage.run_until(Duration::from_secs(60), || ready(&window, visible));
    println!(
        "{} rows, first screen ready: {loaded}",
        window.get_rows().row_count()
    );

    if bench {
        run_bench(&mut stage, &window, scale);
        if let Some(path) = args.get(4) {
            save(&stage, Path::new(path));
        }
    } else {
        let folder = PathBuf::from(args.get(1).expect("a folder for the pictures"));
        std::fs::create_dir_all(&folder).unwrap();
        run_shots(&mut stage, &window, &folder);
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

    window.set_detail_open(false);
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
