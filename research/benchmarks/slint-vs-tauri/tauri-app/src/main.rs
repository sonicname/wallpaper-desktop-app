#![windows_subsystem = "windows"]
use std::{thread::sleep, time::{Duration, SystemTime}}; use tauri::{http::Response, AppHandle, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder};
fn arg(i: usize) -> String { std::env::args().nth(i).unwrap() }
fn log(m: &str) { use std::io::Write; let mut f = std::fs::OpenOptions::new().create(true).append(true).open(arg(3)).unwrap(); writeln!(f, "{} {m}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros()).unwrap(); }
fn make(h: &AppHandle) { let w = WebviewWindowBuilder::new(h, "main", WebviewUrl::App("index.html".into())).title("bench-window").inner_size(1200.0, 800.0).always_on_top(true)
    .data_directory(arg(4).into()).visible(false).build().unwrap(); w.set_position(tauri::PhysicalPosition::new(200, 100)).unwrap(); w.show().unwrap(); if std::env::var("BENCH_COLS").is_ok() { w.maximize().unwrap(); } }
fn s(n: u64) { sleep(Duration::from_secs(n)) }
fn main() { let mode = arg(1);
    tauri::Builder::default().register_asynchronous_uri_scheme_protocol("thumb", |_c, req, resp| { std::thread::spawn(move || {
            let p = req.uri().path().trim_start_matches('/').to_string(); let mut ct = "text/plain";
            let body = if p == "cols" { std::env::var("BENCH_COLS").unwrap_or("4".into()).into_bytes() } else if p == "mode" { arg(1).into_bytes() } else if let Some(m) = p.strip_prefix("log/") { log(&m.replace("%20", " ")); vec![] } else { ct = "image/jpeg"; std::fs::read(format!("{}/{p}.jpg", arg(2))).unwrap() };
            resp.respond(Response::builder().header("content-type", ct).header("access-control-allow-origin", "*").header("cache-control", "max-age=3600").body(body).unwrap()); }); })
        .setup(move |app| { let h = app.handle().clone(); if mode != "min" && mode != "tray" { make(&h); }
            std::thread::spawn(move || { match mode.as_str() { "min" => s(20),
                "tray" => { make(&h); s(4); h.get_webview_window("main").unwrap().destroy().unwrap(); log("destroyed"); s(15);
                    for _ in 0..8 { log("reopen_start"); make(&h); s(3); h.get_webview_window("main").unwrap().destroy().unwrap(); s(3); } log("idle_after_cycles"); s(16); }
                _ => s(18) } h.exit(0); }); Ok(()) })
        .build(tauri::generate_context!()).unwrap().run(|_, e| if let RunEvent::ExitRequested { api, code: None, .. } = e { api.prevent_exit(); }); }
