#![windows_subsystem = "windows"]
slint::include_modules!();
use slint::{ComponentHandle, Image, Model, ModelNotify, ModelRc, ModelTracker, SharedPixelBuffer, Rgba8Pixel, VecModel, Timer};
use std::{cell::RefCell, collections::VecDeque, rc::Rc, sync::{mpsc, Arc, Mutex}, time::{Duration, Instant, SystemTime}};
fn cols() -> usize { std::env::var("BENCH_COLS").ok().and_then(|x| x.parse().ok()).unwrap_or(4) }
const ROWS: usize = 125; const KEEP: usize = 24;
thread_local! { static CACHE: RefCell<VecDeque<(usize, Rc<VecModel<Image>>)>> = RefCell::new(VecDeque::new()); }
struct Lazy { tx: mpsc::Sender<usize>, n: ModelNotify }
impl Model for Lazy { type Data = ModelRc<Image>;
    fn row_count(&self) -> usize { ROWS }
    fn row_data(&self, r: usize) -> Option<ModelRc<Image>> { if r >= ROWS { return None; }
        CACHE.with(|c| { let mut c = c.borrow_mut();
            if let Some((_, m)) = c.iter().find(|x| x.0 == r) { return Some(ModelRc::from(m.clone())); }
            let m = Rc::new(VecModel::from(vec![Image::default(); cols()])); c.push_back((r, m.clone())); if c.len() > KEEP { c.pop_front(); }
            for i in 0..cols() { self.tx.send(r * cols() + i).unwrap(); } Some(ModelRc::from(m)) }) }
    fn model_tracker(&self) -> &dyn ModelTracker { &self.n } }
fn spawn_decoders(dir: String) -> mpsc::Sender<usize> { let (tx, rx) = mpsc::channel::<usize>(); let rx = Arc::new(Mutex::new(rx));
    for _ in 0..4 { let (rx, dir) = (rx.clone(), dir.clone()); std::thread::spawn(move || loop {
        let i = match rx.lock().unwrap().recv() { Ok(i) => i, Err(_) => return };
        let im = image::open(format!("{dir}/t{i}.jpg")).unwrap().to_rgba8();
        let b = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(im.as_raw(), im.width(), im.height());
        let _ = slint::invoke_from_event_loop(move || CACHE.with(|c| { let m = c.borrow().iter().find(|x| x.0 == i / cols()).map(|x| x.1.clone()); if let Some(m) = m { m.set_row_data(i % cols(), Image::from_rgba8(b)); } })); }); } tx }
fn log(m: String) { use std::io::Write; let mut f = std::fs::OpenOptions::new().create(true).append(true).open(std::env::args().nth(3).unwrap()).unwrap();
    writeln!(f, "{} {m}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros()).unwrap(); }
fn make(dir: &str) -> App { let app = App::new().unwrap(); app.window().set_position(slint::PhysicalPosition::new(200, 100));
    app.set_rows(ModelRc::new(Lazy { tx: spawn_decoders(dir.into()), n: Default::default() })); app.show().unwrap(); if cols() != 4 { let wk = app.as_weak(); Timer::single_shot(Duration::from_millis(150), move || wk.unwrap().window().set_maximized(true)); } app }
fn close(a: App) { a.hide().unwrap(); drop(a); CACHE.with(|c| c.borrow_mut().clear()); }
fn cycle(dir: String, left: u32) { if left == 0 { Timer::single_shot(Duration::from_secs(16), || slint::quit_event_loop().unwrap()); log("idle_after_cycles".into()); return; }
    log("reopen_start".into()); let w = make(&dir);
    Timer::single_shot(Duration::from_secs(3), move || { close(w); Timer::single_shot(Duration::from_secs(3), move || cycle(dir, left - 1)); }); }
fn main() { let a: Vec<String> = std::env::args().collect(); let (mode, dir) = (a[1].clone(), a[2].clone());
    match mode.as_str() {
        "min" => { Timer::single_shot(Duration::from_secs(20), || slint::quit_event_loop().unwrap()); slint::run_event_loop_until_quit().unwrap(); }
        "tray" => { let w = make(&dir); Timer::single_shot(Duration::from_secs(4), move || { close(w); log("destroyed".into());
                Timer::single_shot(Duration::from_secs(15), move || cycle(dir, 8)); }); slint::run_event_loop_until_quit().unwrap(); }
        _ => { let w = make(&dir); let fr = Rc::new(RefCell::new(Vec::<Instant>::new()));
            if mode == "snap" { let wk = w.as_weak(); w.set_label("Hình nền ngẫu nhiên mỗi ngày — ửữộặệ ĐƯỢC ỷỵ".into()); Timer::single_shot(Duration::from_secs(2), move || { let b = wk.unwrap().window().take_snapshot().unwrap(); let p = std::env::args().nth(3).unwrap().replace(".log", ".png"); image::save_buffer(p, b.as_bytes(), b.width(), b.height(), image::ColorType::Rgba8).unwrap(); slint::quit_event_loop().unwrap(); }); }
            if mode == "scroll" { let f2 = fr.clone(); w.on_frame(move || f2.borrow_mut().push(Instant::now())); let wk = w.as_weak();
                Timer::single_shot(Duration::from_secs(3), move || { log("scroll_start".into()); wk.unwrap().set_scrolling(true); });
                let wk = w.as_weak(); Timer::single_shot(Duration::from_secs(13), move || { wk.unwrap().set_scrolling(false); log("scroll_end".into());
                    let d: Vec<String> = fr.borrow().windows(2).map(|p| format!("{:.2}", (p[1] - p[0]).as_secs_f64() * 1e3)).collect(); log(format!("frames {}", d.join(","))); }); }
            Timer::single_shot(Duration::from_secs(18), || slint::quit_event_loop().unwrap()); slint::run_event_loop_until_quit().unwrap(); drop(w); } } }
