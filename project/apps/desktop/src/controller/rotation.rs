//! The part of the controller that changes the wallpaper every day, and the tray icon.

use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use slint::{CloseRequestResponse, ComponentHandle, Image, PlatformError, TimerMode};
use wallpaper_core::{Autostart, DesktopError, ProviderError, Size};
use wallpaper_daily::{Daily, READY, Ready, Settings, Source, current_day};

use super::{Controller, enabled_categories, model, with_current};
use crate::{
    DailyState, Option as UiOption, Tray,
    apply::{self, ApplyError},
    icon::icon,
    text,
};

/// Often enough for a change on time, and after waking up, without costing anything.
const TICK: Duration = Duration::from_secs(60);
/// Lets Windows settle after logging in.
const FIRST_TICK: Duration = Duration::from_secs(3);
const RETRY_CHANGE: Duration = Duration::from_secs(15 * 60);

impl Controller {
    pub(super) fn connect_daily(&self) {
        let window = &self.window;
        window.set_app_icon(icon());
        window.set_has_daily(self.daily.is_some() && self.gallery.is_some());
        window.on_open_daily(|| {
            with_current(|c| {
                c.show_daily();
                c.window.set_daily_open(true);
            })
        });
        window.on_toggle_daily(|| with_current(|c| c.configure(|s| s.enabled = !s.enabled)));
        window.on_toggle_paused(|| with_current(|c| c.configure(|s| s.paused = !s.paused)));
        window.on_shift_hour(|by| {
            with_current(|c| {
                c.configure(|s| {
                    let hour = (s.change_at / 60) as i32 + by;
                    s.change_at = hour.rem_euclid(24) as u32 * 60;
                })
            })
        });
        window.on_pick_daily_sort(|i| {
            with_current(|c| {
                let sorts = c.daily_source().provider.info().capabilities.sorts;
                if let Some(sort) = sorts.get(i as usize) {
                    c.configure(|s| s.sort = *sort);
                }
            })
        });
        window.on_pick_daily_category(|i| {
            with_current(|c| {
                let all = c.daily_source().provider.info().capabilities.categories;
                if let Some(picked) = all.get(i as usize) {
                    c.configure(|s| super::toggle(&mut s.categories, *picked, all));
                }
            })
        });
        window.on_pick_daily_source(|i| {
            with_current(|c| {
                if let Some(source) = c.sources.get(i as usize) {
                    let id = source.provider.info().id;
                    c.configure(|s| s.source = id.into());
                }
            })
        });
        window.on_toggle_autostart(|| with_current(Controller::toggle_autostart));
        window.on_next_daily(|| with_current(|c| c.rotate(true)));
        window.on_block_daily(|| with_current(Controller::block_shown));
        self.show_daily();
    }

    pub(super) fn start_tray(&self) -> Result<(), PlatformError> {
        let tray = Tray::new()?;
        tray.set_picture(icon());
        tray.on_open(|| with_current(|c| drop(c.open())));
        tray.on_next(|| with_current(|c| c.rotate(true)));
        tray.on_previous(|| with_current(Controller::undo));
        tray.on_block(|| with_current(Controller::block_shown));
        tray.on_toggle_paused(|| with_current(|c| c.configure(|s| s.paused = !s.paused)));
        tray.on_quit(|| drop(slint::quit_event_loop()));
        *self.tray.borrow_mut() = Some(tray);
        self.show_daily();
        Ok(())
    }

    pub(super) fn start_ticking(&self) {
        slint::Timer::single_shot(FIRST_TICK, || with_current(Controller::tick));
        self.ticker
            .start(TimerMode::Repeated, TICK, || with_current(Controller::tick));
    }

    pub(super) fn runs_in_background(&self) -> bool {
        self.daily.as_ref().is_some_and(|d| d.settings().enabled)
    }

    pub(super) fn open(&self) -> Result<(), PlatformError> {
        self.window
            .set_rows(slint::ModelRc::from(self.grid.clone()));
        self.window.show()?;
        if !self.loaded.replace(true) {
            self.reload();
        }
        self.show_undo();
        self.show_daily();
        // A window that is open already may be behind others or minimized. The handle of a new
        // one is only there once the event loop has run.
        slint::Timer::single_shot(Duration::from_millis(50), || {
            with_current(|c| c.raise_window())
        });
        Ok(())
    }

    fn raise_window(&self) {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let window = self.window.window().window_handle();
        let handle = window.window_handle().map(|h| h.as_raw());
        if let Ok(RawWindowHandle::Win32(handle)) = handle {
            wallpaper_platform_windows::raise(handle.hwnd.get());
        }
    }

    /// The close button. Below the tray nothing of the window is kept in memory.
    pub(super) fn close(&self) -> CloseRequestResponse {
        if self.runs_in_background() {
            self.window.set_detail_open(false);
            self.window.set_daily_open(false);
            self.window.set_rows(Default::default());
            self.window.set_picture(Image::default());
            self.window.set_has_picture(false);
            self.grid.release();
        } else {
            let _ = slint::quit_event_loop();
        }
        CloseRequestResponse::HideWindow
    }

    /// Called every minute, and whenever something may have become possible.
    pub(super) fn tick(&self) {
        let Some(daily) = &self.daily else { return };
        let settings = daily.settings();
        if !settings.enabled || settings.paused {
            return;
        }
        let later = |at: Option<Instant>| at.is_some_and(|at| Instant::now() < at);
        if daily.is_due(self.system.now())
            && !later(self.change_after.get())
            && !self.system.do_not_disturb()
        {
            self.rotate(false);
        }
        if !later(self.refill_after.get()) {
            self.refill();
        }
        self.show_daily();
    }

    /// Shows the next prepared picture on every screen that has one.
    pub(super) fn rotate(&self, asked: bool) {
        let Some(daily) = self.daily.clone() else {
            return;
        };
        self.update_screens();
        let wanted: Vec<(String, Ready)> = self
            .screens
            .borrow()
            .iter()
            .filter_map(|screen| Some((screen.id.clone(), daily.ready(screen.size)?)))
            .collect();
        if wanted.is_empty() {
            // On its own the next minute tries again; asked for, it should not wait.
            if asked {
                self.change_when_ready.set(true);
                self.refill();
            }
            return self.show_daily();
        }
        // Without knowing the workspace the change could not be undone on the right one.
        let Some(workspace) = self.desktop.workspace() else {
            if asked {
                self.report("Chưa biết bạn đang ở desktop nào. Mở một cửa sổ rồi thử lại.".into());
            }
            return;
        };
        self.change_when_ready.set(false);
        let day = current_day(self.system.now(), daily.settings().change_at);
        let message = if asked {
            "Đã đổi hình nền."
        } else {
            "Hình nền hôm nay đã đổi."
        };
        self.change_desktop(message, Some(workspace), asked, move |place| {
            let mut pictures = vec![];
            let mut used: Vec<(Ready, PathBuf)> = vec![];
            for (screen, ready) in wanted {
                let path = place
                    .gallery
                    .adopt(&ready.path)
                    .map_err(|_| ApplyError::Disk)?;
                pictures.push((screen, path.clone()));
                used.push((ready, path));
            }
            let result = apply::show(pictures, place);
            let shown = matches!(
                &result,
                Ok(()) | Err(ApplyError::Desktop(DesktopError::WorkspaceSwitched))
            );
            if shown {
                // Not being able to save this would at worst change the wallpaper again.
                let _ = daily.used(&used, day);
            }
            result
        });
    }

    pub(super) fn block_shown(&self) {
        let Some(daily) = &self.daily else { return };
        let shown = self.desktop.read().map_or(vec![], |backdrop| {
            backdrop.pictures.into_iter().map(|p| p.path).collect()
        });
        if daily.block(&shown) {
            self.rotate(true);
        } else {
            self.report("Ảnh đang hiện không phải do tự đổi mỗi ngày.".into());
        }
    }

    fn refill(&self) {
        let Some(daily) = self.daily.clone() else {
            return;
        };
        if self.refilling.get() {
            return;
        }
        let missing: Vec<Size> = self
            .sizes()
            .into_iter()
            .filter(|size| daily.count(*size) < READY)
            .collect();
        if missing.is_empty() {
            return;
        }
        self.refilling.set(true);
        let from = self.daily_source();
        let (provider, http, loader) = (
            from.provider.clone(),
            from.http.clone(),
            self.loader.share(),
        );
        thread::spawn(move || {
            let source = Source {
                provider: &*provider,
                http: &*http,
                download: &|item| loader.fresh_original(item),
            };
            let mut random = fastrand::Rng::new();
            let mut added = Ok(0);
            for size in missing {
                match daily.refill(size, &source, &mut random) {
                    Ok(more) => added = added.map(|sum| sum + more),
                    Err(error) => {
                        added = Err(error);
                        break;
                    }
                }
            }
            let _ = slint::invoke_from_event_loop(move || with_current(|c| c.refilled(added)));
        });
    }

    fn refilled(&self, added: Result<usize, ProviderError>) {
        self.refilling.set(false);
        if matches!(added, Ok(1..)) {
            self.failures.set(0);
            self.refill_after.set(None);
        } else {
            // 2, 4, 8, 16, then every 30 minutes.
            self.failures.set((self.failures.get() + 1).min(5));
            let minutes = (1_u64 << self.failures.get()).min(30);
            self.refill_after
                .set(Some(Instant::now() + Duration::from_secs(minutes * 60)));
        }
        *self.refill_error.borrow_mut() = added.err();
        if self.change_when_ready.get() && self.has_ready() {
            self.rotate(true);
        } else if self.has_ready() {
            self.tick();
        }
        self.show_daily();
    }

    /// Where the daily pictures come from: the source in the settings, or the first one.
    pub(super) fn daily_source(&self) -> &super::library::Source {
        let wanted = self.daily.as_ref().map(|d| d.settings().source);
        let chosen = |s: &&super::library::Source| Some(s.provider.info().id) == wanted.as_deref();
        self.sources.iter().find(chosen).unwrap_or(&self.sources[0])
    }

    /// The pictures of a source changed, which matters if the daily change takes them from it.
    pub(super) fn source_changed(&self, id: &str) {
        let Some(daily) = &self.daily else { return };
        if self.daily_source().provider.info().id != id {
            return;
        }
        if id == wallpaper_provider_folder::ID {
            // Another folder: what was prepared belongs to the old one.
            let _ = daily.restart();
        } else if let Some(favorites) = &self.favorites {
            // A picture that is no favorite any more must not be shown tomorrow.
            for size in self.sizes() {
                while let Some(ready) = daily.ready(size) {
                    if favorites.contains_key(&ready.key) {
                        break;
                    }
                    let _ = daily.drop_ready(&ready.key);
                }
            }
        }
        self.refill_after.set(None);
        *self.refill_error.borrow_mut() = None;
        self.tick();
        self.show_daily();
    }

    fn has_ready(&self) -> bool {
        let daily = self.daily.as_ref();
        daily.is_some_and(|d| self.sizes().into_iter().any(|s| d.ready(s).is_some()))
    }

    fn sizes(&self) -> Vec<Size> {
        let mut sizes: Vec<Size> = vec![];
        for screen in self.screens.borrow().iter() {
            if !sizes.contains(&screen.size) {
                sizes.push(screen.size);
            }
        }
        sizes
    }

    fn configure(&self, change: impl FnOnce(&mut Settings)) {
        let Some(daily) = &self.daily else { return };
        if daily.configure(change).is_err() {
            self.report("Không lưu được cài đặt. Kiểm tra dung lượng trống.".into());
        }
        // What went wrong before may work with the new settings.
        self.refill_after.set(None);
        self.change_after.set(None);
        *self.refill_error.borrow_mut() = None;
        self.update_screens();
        self.tick();
        self.show_daily();
    }

    fn toggle_autostart(&self) {
        let on = self.system.autostart() == Autostart::Off;
        if let Err(reason) = self.system.set_autostart(on) {
            self.report(format!("Windows không cho đổi: {reason}"));
        }
        self.show_daily();
    }

    /// After every change of the wallpaper, whoever asked for it.
    pub(super) fn after_change(&self, asked: bool, result: &Result<(), ApplyError>) {
        if !asked {
            // Being on another workspace sorts itself out; the rest is not worth a try a minute.
            let retry = match result {
                Err(ApplyError::Desktop(DesktopError::OtherWorkspace)) | Ok(()) => None,
                Err(_) => Some(Instant::now() + RETRY_CHANGE),
            };
            self.change_after.set(retry);
        }
        if self.runs_in_background() {
            self.refill();
        }
        self.show_daily();
    }

    fn report(&self, message: String) {
        self.show_toast(message.clone());
        if let Some(tray) = &*self.tray.borrow() {
            tray.set_status(message.into());
        }
    }

    pub(super) fn show_daily(&self) {
        let Some(daily) = &self.daily else { return };
        let settings = daily.settings();
        let (status, problem) = self.daily_status(daily, &settings);
        let option = |label: &str, on| UiOption {
            label: label.into(),
            on,
        };
        let from = self.daily_source().provider.info();
        let all = from.capabilities.categories;
        let on = enabled_categories(&settings.categories, all);
        let sorts = from.capabilities.sorts;
        // A sort the source does not have falls back to its first one.
        let sort = Some(settings.sort).filter(|s| sorts.contains(s));
        let sort = sort.or(sorts.first().copied());
        let autostart = self.system.autostart();
        self.window.set_daily(DailyState {
            enabled: settings.enabled,
            paused: settings.paused,
            hour: text::clock(settings.change_at).into(),
            status: status.clone().into(),
            problem,
            autostart: autostart != Autostart::Off,
            autostart_note: text::autostart(autostart, settings.enabled).into(),
            sources: model(
                self.sources
                    .iter()
                    .map(|s| option(s.provider.info().name, s.provider.info().id == from.id)),
            ),
            sorts: model(
                sorts
                    .iter()
                    .map(|s| option(text::sort(*s), Some(*s) == sort)),
            ),
            categories: model(
                all.iter()
                    .map(|c| option(text::category(*c), on.contains(c))),
            ),
        });
        if let Some(tray) = &*self.tray.borrow() {
            tray.set_shown(settings.enabled);
            tray.set_paused(settings.paused);
            tray.set_busy(self.changing.get());
            tray.set_can_undo(self.window.get_can_undo());
            tray.set_status(format!("Wallpaper\n{status}").into());
        }
    }

    fn daily_status(&self, daily: &Arc<Daily>, settings: &Settings) -> (String, bool) {
        if settings.paused {
            return ("Đang tạm dừng.".into(), false);
        }
        let ready = self.sizes().iter().map(|s| daily.count(*s)).min();
        let ready = ready.unwrap_or(0);
        if let Some(error) = &*self.refill_error.borrow() {
            // With pictures waiting, being offline is not a problem yet.
            if ready == 0 {
                return (text::error(error), true);
            }
        }
        if self.refilling.get() && ready == 0 {
            return ("Đang chuẩn bị ảnh…".into(), false);
        }
        if ready == 0 {
            return ("Không có ảnh nào khớp bộ lọc và màn hình.".into(), true);
        }
        let today = current_day(self.system.now(), settings.change_at);
        let due = daily.last_day().is_none_or(|last| last < today);
        let when = if due {
            "Sẽ đổi ngay khi có thể"
        } else {
            "Đã đổi hôm nay"
        };
        (format!("{when}. Có sẵn {ready} ảnh."), false)
    }
}
