mod library;
mod rotation;

use std::{
    cell::{Cell as StdCell, RefCell},
    collections::HashSet,
    path::PathBuf,
    rc::{Rc, Weak},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use slint::{ComponentHandle, ModelRc, PlatformError, SharedString, Timer, TimerMode, VecModel};
use wallpaper_core::{
    BackdropMode, Category, Desktop, Item, ListRequest, Page, Provider, ProviderError, Ratio,
    Screen, Size, Sort, System,
};
use wallpaper_daily::Daily;
use wallpaper_favorites::Favorites;
use wallpaper_host::{DiskCache, Gallery, Pixels, ambient, cover};
use wallpaper_provider_folder::Folder;

use library::Source;

use crate::{
    AppWindow, Detail, Option as UiOption, Shade, Tray,
    apply::{self, ApplyError, Place},
    grid::{self, GridModel},
    loader::Loader,
    text,
};

const THUMB_BYTES: u64 = 200 << 20;
const ORIGINAL_BYTES: u64 = 200 << 20;
const TOAST: Duration = Duration::from_secs(8);
/// Larger originals stay blurry in the preview instead of using hundreds of megabytes.
const MAX_PREVIEW_PIXELS: u64 = 40_000_000;
/// A random listing with few matches repeats itself. Stop after this many pages without news.
const MAX_STALE_PAGES: u32 = 2;

thread_local! {
    static CURRENT: RefCell<Weak<Controller>> = const { RefCell::new(Weak::new()) };
}

pub struct Config {
    /// Sources from the web, at least one. The first one is shown first.
    pub providers: Vec<Arc<dyn Provider>>,
    pub desktop: Arc<dyn Desktop>,
    pub system: Arc<dyn System>,
    /// Where pictures, favorites and the undo history are kept. Without it nothing is written:
    /// the wallpaper cannot be changed, and there are no favorites and no folder source.
    pub data_dir: Option<PathBuf>,
}

pub struct App {
    controller: Rc<Controller>,
}

impl App {
    pub fn new(config: Config) -> Result<Self, PlatformError> {
        Ok(Self {
            controller: Controller::new(config)?,
        })
    }

    pub fn window(&self) -> &AppWindow {
        &self.controller.window
    }

    /// Shows the window, and loads the first page the first time.
    pub fn open(&self) -> Result<(), PlatformError> {
        self.controller.open()
    }

    /// Runs until the user quits. Without a reason to run in the background it returns at once.
    pub fn run(&self, background: bool) -> Result<(), PlatformError> {
        if background && !self.controller.runs_in_background() {
            return Ok(());
        }
        self.controller.start_tray()?;
        let window = self.controller.window.window();
        window.on_close_requested(|| {
            let mut response = slint::CloseRequestResponse::HideWindow;
            with_current(|c| response = c.close());
            response
        });
        if !background {
            self.open()?;
        }
        self.controller.start_ticking();
        slint::run_event_loop_until_quit()
    }

    /// Changes the wallpaper if it is due, and prepares the next ones. Runs every minute.
    pub fn tick(&self) {
        self.controller.tick();
    }

    /// What the close button of the window does.
    pub fn close(&self) {
        self.controller.close();
    }
}

struct Listing {
    request: ListRequest,
    fit_screen: bool,
    seen: HashSet<String>,
    total: Option<u64>,
    loading: bool,
    finished: bool,
    stale_pages: u32,
    failed: bool,
}

struct Controller {
    window: AppWindow,
    sources: Vec<Source>,
    /// Index into `sources`.
    current: StdCell<usize>,
    /// The filters of every source, kept while another one is shown.
    saved: RefCell<Vec<(ListRequest, bool)>>,
    favorites: Option<Arc<Favorites>>,
    folder: Option<Arc<Folder>>,
    picking: StdCell<bool>,
    loader: Loader,
    grid: Rc<GridModel>,
    desktop: Arc<dyn Desktop>,
    gallery: Option<Arc<Gallery>>,
    /// The primary screen when the app started, which the screen filter stands for.
    screen: Option<Size>,
    screens: RefCell<Vec<Screen>>,
    /// Index into `screens`, or none for all of them.
    target: StdCell<Option<usize>>,
    changing: StdCell<bool>,
    listing: RefCell<Listing>,
    /// Answers to an older listing are ignored.
    epoch: StdCell<u64>,
    selected: StdCell<usize>,
    preview: Arc<AtomicU64>,
    resize: Timer,
    toast: Timer,
    system: Arc<dyn System>,
    daily: Option<Arc<Daily>>,
    tray: RefCell<Option<Tray>>,
    ticker: Timer,
    loaded: StdCell<bool>,
    refilling: StdCell<bool>,
    /// Refills in a row that brought nothing.
    failures: StdCell<u32>,
    refill_after: StdCell<Option<Instant>>,
    refill_error: RefCell<Option<ProviderError>>,
    change_after: StdCell<Option<Instant>>,
    /// The user asked for the next picture before one was prepared.
    change_when_ready: StdCell<bool>,
}

impl Controller {
    fn new(config: Config) -> Result<Rc<Self>, PlatformError> {
        let window = AppWindow::new()?;
        let folder = |name| config.data_dir.as_ref().map(|dir| dir.join(name));
        let favorites = folder("favorites")
            .and_then(|dir| Favorites::open(dir).ok())
            .map(Arc::new);
        let local = folder("folder.txt").map(|file| Arc::new(Folder::open(file)));
        let mut sources: Vec<Source> = config
            .providers
            .into_iter()
            .map(|provider| Source::new(provider, false))
            .collect();
        if let Some(favorites) = &favorites {
            sources.push(Source::new(favorites.clone(), true));
        }
        if let Some(local) = &local {
            sources.push(Source::new(local.clone(), true));
        }
        assert!(!sources.is_empty(), "the app needs at least one source");
        let cache = |name, max_bytes| {
            let cache = DiskCache::open(folder(name)?).ok()?;
            let dir = cache.dir().to_owned();
            thread::spawn(move || DiskCache::open(dir).and_then(|c| c.trim(max_bytes)));
            Some(cache)
        };
        let loader = Loader::new(
            sources[0].http.clone(),
            cache("thumbs", THUMB_BYTES),
            cache("originals", ORIGINAL_BYTES),
        );
        let gallery = folder("pictures")
            .and_then(|dir| Gallery::open(dir).ok())
            .map(Arc::new);
        let daily = folder("daily")
            .and_then(|dir| Daily::open(dir).ok())
            .map(Arc::new);
        let screens = config.desktop.screens();
        let screen = screens.first().map(|s| s.size);
        // The model is asked for rows in the middle of a redraw, so load the page afterwards.
        let hearts = favorites.clone();
        let grid = GridModel::new(
            loader.share(),
            || Timer::single_shot(Duration::ZERO, || with_current(Controller::load_more)),
            move |item| hearts.as_ref().is_some_and(|f| f.contains(item)),
        );
        // Pictures from the web are filtered to the screen from the start. The user's own are not.
        let saved: Vec<(ListRequest, bool)> = sources
            .iter()
            .map(|source| {
                let capabilities = source.provider.info().capabilities;
                let can_fit = capabilities.min_size || !capabilities.ratios.is_empty();
                let fit = !source.own && screen.is_some() && can_fit;
                (source.first_request(), fit)
            })
            .collect();
        let (request, fit_screen) = saved[0].clone();
        let controller = Rc::new(Self {
            sources,
            current: StdCell::new(0),
            saved: RefCell::new(saved),
            favorites,
            folder: local,
            picking: StdCell::new(false),
            loader,
            grid,
            desktop: config.desktop,
            gallery,
            screen,
            screens: RefCell::new(screens),
            target: StdCell::new(None),
            changing: StdCell::new(false),
            listing: RefCell::new(Listing {
                request,
                fit_screen,
                seen: HashSet::new(),
                total: None,
                loading: false,
                finished: false,
                stale_pages: 0,
                failed: false,
            }),
            epoch: StdCell::new(0),
            selected: StdCell::new(0),
            preview: Arc::default(),
            resize: Timer::default(),
            toast: Timer::default(),
            system: config.system,
            daily,
            tray: RefCell::default(),
            ticker: Timer::default(),
            loaded: StdCell::new(false),
            refilling: StdCell::new(false),
            failures: StdCell::new(0),
            refill_after: StdCell::new(None),
            refill_error: RefCell::default(),
            change_after: StdCell::new(None),
            change_when_ready: StdCell::new(false),
            window,
        });
        CURRENT.with(|c| *c.borrow_mut() = Rc::downgrade(&controller));
        controller.connect();
        Ok(controller)
    }

    fn connect(self: &Rc<Self>) {
        let window = &self.window;
        window.set_gap(grid::GAP);
        window.set_rows(ModelRc::from(self.grid.clone()));
        window.set_can_apply(self.gallery.is_some());
        self.show_targets();
        self.show_undo();
        self.connect_daily();
        self.connect_library();
        window.on_pick_target(|i| with_current(|c| c.pick_target(i as usize)));
        window.on_apply(|| with_current(Controller::apply));
        window.on_undo(|| with_current(Controller::undo));
        window.on_close_toast(|| with_current(|c| c.window.set_toast(SharedString::new())));
        window.on_search(|text| with_current(|c| c.change(|l| l.request.text = text.into())));
        window.on_pick_sort(|i| {
            with_current(|c| {
                if let Some(sort) = c.sorts().get(i as usize) {
                    c.change(|l| l.request.sort = *sort);
                }
            })
        });
        window.on_pick_category(|i| with_current(|c| c.toggle_category(i as usize)));
        window.on_pick_shade(|i| {
            with_current(|c| {
                let colors = c.provider().info().capabilities.colors;
                let shade = colors.get(i as usize).copied();
                c.change(|l| {
                    l.request.color = if l.request.color == shade {
                        None
                    } else {
                        shade
                    }
                });
            })
        });
        window
            .on_toggle_screen_fit(|| with_current(|c| c.change(|l| l.fit_screen = !l.fit_screen)));
        window.on_retry(|| with_current(|c| c.load_more_after_error()));
        window.on_open(|i| with_current(|c| c.show(i as usize)));
        window.on_apply_at(|i| {
            with_current(|c| {
                c.selected.set(i as usize);
                c.apply();
            })
        });
        window.on_step(|by| {
            with_current(|c| {
                if let Some(next) = c.selected.get().checked_add_signed(by as isize)
                    && next < c.grid.len()
                {
                    c.show(next);
                }
            })
        });
        window.on_open_source(|| {
            with_current(|c| {
                if let Some(item) = c.grid.item(c.selected.get()) {
                    Controller::open_page(&item);
                }
            })
        });
        window.on_layout_changed(|| {
            with_current(|c| {
                // Resizing by hand reports every pixel; decode once the size settles.
                let wait = Duration::from_millis(if c.grid.len() == 0 { 0 } else { 120 });
                c.resize.start(TimerMode::SingleShot, wait, || {
                    with_current(Controller::relayout)
                });
            })
        });
    }

    fn sorts(&self) -> &'static [Sort] {
        self.provider().info().capabilities.sorts
    }

    fn relayout(&self) {
        self.measure();
        self.fill();
    }

    fn measure(&self) {
        let scale = self.window.window().scale_factor();
        let (width, height) = (self.window.get_grid_width(), self.window.get_grid_height());
        self.grid
            .resize(width, height, scale, self.listing.borrow().finished);
    }

    fn change(&self, edit: impl FnOnce(&mut Listing)) {
        let before = self.request();
        edit(&mut self.listing.borrow_mut());
        if self.request() != before {
            self.reload();
        }
        self.show_filters();
    }

    fn toggle_category(&self, index: usize) {
        let all = self.provider().info().capabilities.categories;
        let Some(picked) = all.get(index) else { return };
        self.change(|l| toggle(&mut l.request.categories, *picked, all));
    }

    /// The request without its position, with the screen filter applied.
    fn request(&self) -> ListRequest {
        let listing = self.listing.borrow();
        let mut request = ListRequest {
            cursor: None,
            ..listing.request.clone()
        };
        if let Some(screen) = self.screen.filter(|_| listing.fit_screen) {
            let capabilities = self.provider().info().capabilities;
            request.min_size = capabilities.min_size.then_some(screen);
            request.ratios = Ratio::nearest(screen, capabilities.ratios)
                .into_iter()
                .collect();
        }
        request
    }

    fn reload(&self) {
        self.epoch.set(self.epoch.get() + 1);
        {
            let mut listing = self.listing.borrow_mut();
            listing.request.cursor = None;
            listing.seen.clear();
            listing.total = None;
            listing.loading = false;
            listing.finished = false;
            listing.stale_pages = 0;
            listing.failed = false;
        }
        self.grid.clear();
        self.window.set_scroll(0.0);
        self.show_filters();
        self.load_more();
    }

    fn load_more_after_error(&self) {
        self.listing.borrow_mut().failed = false;
        self.load_more();
    }

    fn load_more(&self) {
        {
            let mut listing = self.listing.borrow_mut();
            if listing.loading || listing.finished || listing.failed {
                return;
            }
            listing.loading = true;
        }
        self.show_status(None);
        let mut request = self.request();
        request.cursor = self.listing.borrow().request.cursor.clone();
        let source = self.source();
        let (provider, http, epoch) = (
            source.provider.clone(),
            source.http.clone(),
            self.epoch.get(),
        );
        thread::spawn(move || {
            let result = provider.list(&request, &*http);
            // Fails only while the app is closing.
            let _ = slint::invoke_from_event_loop(move || {
                with_current(|c| {
                    if c.epoch.get() == epoch {
                        c.receive(result);
                    }
                })
            });
        });
    }

    fn receive(&self, result: Result<Page, ProviderError>) {
        let page = match result {
            Ok(page) => page,
            Err(error) => {
                let mut listing = self.listing.borrow_mut();
                listing.loading = false;
                listing.failed = true;
                drop(listing);
                return self.show_status(Some(&error));
            }
        };
        let (fresh, finished) = {
            let mut listing = self.listing.borrow_mut();
            let fresh: Vec<Item> = page
                .items
                .into_iter()
                .filter(|i| listing.seen.insert(i.id.clone()))
                .collect();
            listing.stale_pages = if fresh.is_empty() {
                listing.stale_pages + 1
            } else {
                0
            };
            listing.loading = false;
            listing.total = page.total;
            listing.finished = page.next.is_none() || listing.stale_pages >= MAX_STALE_PAGES;
            listing.request.cursor = page.next;
            (fresh, listing.finished)
        };
        // The window reports changes of its size, but never the size it starts with.
        self.measure();
        self.grid.append(fresh, finished);
        self.show_status(None);
        self.fill();
    }

    /// Loads another page when the rows so far do not fill the window.
    fn fill(&self) {
        use slint::Model;
        let height: f32 = self.grid.row_count() as f32 * (grid::ROW_HEIGHT * 0.7);
        if height < self.window.get_grid_height() * 1.5 && self.grid.len() > 0 {
            self.load_more();
        }
    }

    fn show_status(&self, error: Option<&ProviderError>) {
        let listing = self.listing.borrow();
        let empty = self.grid.len() == 0;
        let message = match error {
            Some(error) => text::error(error),
            // The window offers to choose a folder instead.
            None if self.needs_folder() => String::new(),
            None if empty && listing.finished => self
                .empty_reason()
                .unwrap_or("Không có ảnh nào khớp bộ lọc.")
                .into(),
            None => String::new(),
        };
        // With pictures on screen an error goes to the summary instead of covering them.
        let summary = match (listing.total, error) {
            (_, Some(_)) if !empty => message.clone(),
            (Some(total), _) => format!("{} ảnh", text::thousands(total)),
            (None, _) => String::new(),
        };
        self.window.set_busy(listing.loading);
        self.window.set_can_retry(error.is_some());
        self.window.set_message(if empty {
            message.into()
        } else {
            SharedString::new()
        });
        self.window.set_summary(summary.into());
    }

    fn show_filters(&self) {
        let capabilities = self.provider().info().capabilities;
        let listing = self.listing.borrow();
        let request = &listing.request;
        let option = |label: &str, on| UiOption {
            label: label.into(),
            on,
        };
        let on = enabled_categories(&request.categories, capabilities.categories);
        self.window.set_sorts(model(
            capabilities
                .sorts
                .iter()
                .map(|s| option(text::sort(*s), *s == request.sort)),
        ));
        self.window.set_categories(model(
            capabilities
                .categories
                .iter()
                .map(|c| option(text::category(*c), on.contains(c))),
        ));
        self.window
            .set_shades(model(capabilities.colors.iter().map(|c| Shade {
                color: grid::color(*c),
                name: text::shade(*c).into(),
                on: request.color == Some(*c),
            })));
        let screen = self
            .screen
            .filter(|_| capabilities.min_size || !capabilities.ratios.is_empty());
        let label = screen.map(|s| format!("Vừa màn hình {}", grid::size_label(s)));
        self.window
            .set_screen_fit(option(&label.unwrap_or_default(), listing.fit_screen));
    }

    fn show(&self, index: usize) {
        let Some(item) = self.grid.item(index) else {
            return;
        };
        self.selected.set(index);
        self.refresh_screens();
        let fit = text::fit_text(item.size, self.preview_screen().unwrap_or(item.size));
        self.window.set_detail(Detail {
            resolution: grid::size_label(item.size).into(),
            fit_title: fit.title.into(),
            fit_note: fit.note.into(),
            fit_good: fit.good,
            file: text::file(&item.mime, item.file_bytes).into(),
            source: format!("{} · {}", self.source_name(&item), Self::short_id(&item)).into(),
            favorite: self.is_favorite(&item),
            page_label: Self::page_label(&item).into(),
            tint: grid::tint(&item),
            palette: model(item.colors.iter().map(|c| grid::color(*c))),
        });
        self.window.set_has_picture(false);
        self.window.set_picture_sharp(false);
        self.window.set_detail_open(true);
        if self.grid.len() - index < 6 {
            self.load_more();
        }
        self.load_preview(item);
    }

    fn load_preview(&self, item: Item) {
        let ticket = self.preview.fetch_add(1, Ordering::Relaxed) + 1;
        let scale = self.window.window().scale_factor();
        let px = |v: f32| (v * scale).round().max(1.0) as u32;
        let target = Size::new(
            px(self.window.get_picture_width()),
            px(self.window.get_picture_height()),
        );
        // The backdrop is soft anyway, so a third of the window is enough.
        let size = self.window.window().size();
        let backdrop = Size::new(size.width / 3, size.height / 3);
        let (current, loader) = (self.preview.clone(), self.loader.share());
        thread::spawn(move || {
            let wanted = || current.load(Ordering::Relaxed) == ticket;
            let decode = |bytes: Vec<u8>| cover(&bytes, target, 0.0, grid::BACKGROUND).ok();
            let thumb = item.best_thumb(target).and_then(|t| loader.thumb(&t.url));
            if let Some(bytes) = thumb {
                if let Ok(glow) = ambient(&bytes, backdrop, grid::BACKGROUND) {
                    let _ = slint::invoke_from_event_loop(move || {
                        with_current(|c| c.receive_ambient(ticket, glow))
                    });
                }
                if let Some(pixels) = decode(bytes) {
                    deliver_preview(ticket, Some(pixels), false);
                }
            }
            // Holding an arrow key must not download every original on the way.
            thread::sleep(Duration::from_millis(250));
            let pixels = item.size.width as u64 * item.size.height as u64;
            if !wanted() {
                return;
            }
            let full = (pixels <= MAX_PREVIEW_PIXELS)
                .then(|| loader.original(&item).ok())
                .flatten()
                .filter(|_| wanted())
                .and_then(decode);
            deliver_preview(ticket, full, true);
        });
    }

    /// Screens come and go while the app runs.
    fn refresh_screens(&self) {
        self.update_screens();
        let desktop = self.desktop.clone();
        thread::spawn(move || {
            let mode = desktop.read().map_or(BackdropMode::Picture, |b| b.mode);
            let _ = slint::invoke_from_event_loop(move || {
                with_current(|c| {
                    c.window.set_mode_warning(text::mode_warning(mode).into());
                    c.show_undo();
                })
            });
        });
    }

    fn update_screens(&self) {
        let now = self.desktop.screens();
        if *self.screens.borrow() != now {
            *self.screens.borrow_mut() = now;
            self.target.set(None);
            self.show_targets();
        }
    }

    /// The screen the preview imitates.
    fn preview_screen(&self) -> Option<Size> {
        let screens = self.screens.borrow();
        screens.get(self.target.get().unwrap_or(0)).map(|s| s.size)
    }

    /// Chips for "all" and every screen, or none with a single screen.
    fn show_targets(&self) {
        let screens = self.screens.borrow();
        let target = self.target.get();
        let mut options = vec![];
        if screens.len() > 1 {
            options.push(UiOption {
                label: "Tất cả màn hình".into(),
                on: target.is_none(),
            });
            options.extend(screens.iter().enumerate().map(|(i, screen)| UiOption {
                label: text::target(i, screen.size).into(),
                on: target == Some(i),
            }));
        }
        self.window.set_targets(model(options.into_iter()));
        if let Some(screen) = screens.get(target.unwrap_or(0)) {
            self.window.set_screen_aspect(screen.size.aspect());
            self.window
                .set_screen_name(grid::size_label(screen.size).into());
        }
    }

    fn pick_target(&self, option: usize) {
        self.target.set(option.checked_sub(1));
        self.show_targets();
        self.show(self.selected.get());
    }

    fn apply(&self) {
        let Some(item) = self.grid.item(self.selected.get()) else {
            return;
        };
        let screens: Vec<Screen> = match self.target.get() {
            Some(index) => self
                .screens
                .borrow()
                .get(index)
                .cloned()
                .into_iter()
                .collect(),
            None => self.screens.borrow().clone(),
        };
        let loader = self.loader.share();
        self.change_desktop(
            "Đã đặt làm hình nền.",
            self.desktop.workspace(),
            true,
            move |place| apply::apply(&item, &screens, &loader, place),
        );
    }

    fn undo(&self) {
        let workspace = self.desktop.workspace();
        self.change_desktop(
            "Đã trả lại hình nền trước đó.",
            workspace,
            true,
            apply::undo,
        );
    }

    /// `workspace` is the one the user is looking at while asking: the one they mean.
    fn change_desktop(
        &self,
        success: &'static str,
        workspace: Option<String>,
        asked: bool,
        work: impl FnOnce(&Place) -> Result<(), ApplyError> + Send + 'static,
    ) {
        let Some(gallery) = self.gallery.clone() else {
            return;
        };
        // Two changes at once would mix up the undo history.
        if self.changing.replace(true) {
            return;
        }
        self.window.set_changing(true);
        self.show_daily();
        let desktop = self.desktop.clone();
        thread::spawn(move || {
            let result = work(&Place {
                gallery: &gallery,
                desktop: &*desktop,
                workspace: workspace.as_deref(),
            });
            let _ = slint::invoke_from_event_loop(move || {
                with_current(|c| {
                    c.changing.set(false);
                    c.window.set_changing(false);
                    c.show_undo();
                    c.after_change(asked, &result);
                    c.show_toast(match &result {
                        Ok(()) => success.into(),
                        Err(error) => text::apply_error(error),
                    });
                })
            });
        });
    }

    fn show_undo(&self) {
        let workspace = self.desktop.workspace();
        let can_undo = self
            .gallery
            .as_ref()
            .is_some_and(|g| g.previous(workspace.as_deref()).is_some());
        self.window.set_can_undo(can_undo);
    }

    fn show_toast(&self, message: String) {
        self.window.set_toast(message.into());
        self.toast.start(TimerMode::SingleShot, TOAST, || {
            with_current(|c| c.window.set_toast(SharedString::new()))
        });
    }

    fn receive_ambient(&self, ticket: u64, glow: Pixels) {
        if self.preview.load(Ordering::Relaxed) == ticket {
            self.window.set_ambient(grid::to_image(&glow));
            self.window.set_has_ambient(true);
        }
    }

    fn receive_preview(&self, ticket: u64, pixels: Option<Pixels>, last: bool) {
        if self.preview.load(Ordering::Relaxed) != ticket {
            return;
        }
        if let Some(pixels) = pixels {
            self.window.set_picture(grid::to_image(&pixels));
            self.window.set_has_picture(true);
        }
        self.window.set_picture_sharp(last);
    }
}

fn deliver_preview(ticket: u64, pixels: Option<Pixels>, last: bool) {
    let _ = slint::invoke_from_event_loop(move || {
        with_current(|c| c.receive_preview(ticket, pixels, last))
    });
}

fn enabled_categories(chosen: &[Category], all: &[Category]) -> Vec<Category> {
    if chosen.is_empty() { all } else { chosen }.to_vec()
}

/// Turns a category on or off, where an empty list means all of them.
fn toggle(chosen: &mut Vec<Category>, picked: Category, all: &[Category]) {
    let mut on = enabled_categories(chosen, all);
    match on.iter().position(|c| *c == picked) {
        // At least one category has to stay on.
        Some(_) if on.len() == 1 => {}
        Some(at) => drop(on.remove(at)),
        None => on.push(picked),
    }
    *chosen = if on.len() == all.len() {
        vec![]
    } else {
        all.iter().copied().filter(|c| on.contains(c)).collect()
    };
}

fn model<T: Clone + 'static>(values: impl Iterator<Item = T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(values.collect::<Vec<_>>()))
}

/// Shows the window of the running app. Call it on the event loop.
pub fn open_window() {
    with_current(|c| drop(c.open()));
}

fn with_current(run: impl FnOnce(&Controller)) {
    let current = CURRENT.with(|c| c.borrow().upgrade());
    if let Some(controller) = current {
        run(&controller);
    }
}
